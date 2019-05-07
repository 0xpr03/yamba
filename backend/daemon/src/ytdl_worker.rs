/*
 *  This file is part of yamba.
 *
 *  yamba is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  yamba is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with yamba.  If not, see <https://www.gnu.org/licenses/>.
 */

//! Worker for ytdl tasks
//!
//! Adds layer on top of ytdl to allow a fair scheduling of jobs with multi-part resolution of playlists.

use failure::Fallible;
use futures::{Future, Stream};
use mpmc_scheduler as scheduler;
use tokio::runtime::Runtime;
use tokio::timer::Interval;
use tokio_threadpool::blocking;

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::api::callback::send_resolve;
use crate::daemon::instance::{SongCache, ID};
use crate::daemon::Instances;
use crate::ytdl::YtDL;
use crate::SETTINGS;

use yamba_types::models::callback::*;
use yamba_types::models::*;
use yamba_types::track::{GetId, TrackResponse};

#[derive(Fail, Debug)]
pub enum YtDLWorkerErr {
    #[fail(display = "Expected playlist for multipart, got song! For {}", _0)]
    SongOnMultipart(String),
    #[fail(display = "No song for URL {}", _0)]
    NoSongFound(String),
}

pub type R = (Request, ReqSongs);
pub type ReqSongs = Fallible<ResolveType>;
pub type PartSongs = Fallible<Playlist>;
pub type Controller = scheduler::Controller<ID, Request, R>;
pub type YTSender = scheduler::Sender<Request>;

/// Struct for resolve jobs
pub struct ResolveDispatcher {
    url: String,
    ticket: Ticket,
    resolve_limit: usize,
    instance: ID,
}

impl ResolveDispatcher {
    pub fn new(req: ResolveRequest, ticket: Ticket) -> ResolveDispatcher {
        ResolveDispatcher {
            ticket,
            url: req.url,
            resolve_limit: req.limit,
            instance: req.instance,
        }
    }

    /// Returns ResolveState for request
    ///
    /// On Playlist check if retrieved < limit => finished
    fn state(&self, data: &ResolveType) -> ResolveState {
        match data {
            ResolveType::Playlist(p) => {
                if p.songs.len() < self.resolve_limit {
                    ResolveState::Finished
                } else {
                    ResolveState::Part
                }
            }
            _ => ResolveState::Finished,
        }
    }

    /// Run followup
    fn run_followup(self, instances: &Instances) {
        let inst = instances.read().expect("Can't read instances!");
        if let Some(inst) = inst.get(&self.instance) {
            let followup = ResolveFollowup {
                url: self.url,
                ticket: self.ticket,
                start: self.resolve_limit,
                end: self.resolve_limit * 2,
                instance: self.instance,
            };
            if let Err(e) = inst.dispatch_resolve(followup.wrap()) {
                error!("Unable to dispatch followup resolve: {}", e);
                send_resolve(&ResolveResponse {
                    ticket: self.ticket,
                    data: ResolveResponseData::Error(ResolveErrorResponse {
                        details: ErrorCodes::NONE,
                        msg: Some(String::from("Unable to re-queue part request")),
                    }),
                });
            }
        } else {
            debug!("Instance {} shut down, aborting resolve..", self.instance);
            send_resolve(&ResolveResponse {
                ticket: self.ticket,
                data: ResolveResponseData::Error(ResolveErrorResponse {
                    details: ErrorCodes::RESOLVE_CANCELLED,
                    msg: Some(String::from("Instance shut down")),
                }),
            });
        }
    }

    /// Url to resolve
    fn url(&self) -> &str {
        &self.url
    }

    /// Max amount of songs to retrieve
    fn bounds(&self) -> usize {
        self.resolve_limit
    }

    pub fn wrap(self) -> Request {
        Request::Info(self)
    }

    /// Callback, called after resolving of requested url with return value
    fn callback(self, songs: ReqSongs, instances: Instances) {
        let mut followup = false;
        let response = match songs {
            Ok(s) => {
                let state = self.state(&s);
                followup = state == ResolveState::Part;
                ResolveResponse {
                    ticket: self.ticket,
                    data: ResolveResponseData::Initial(ResolveInitialResponse {
                        state: state,
                        data: s,
                    }),
                }
            }
            Err(e) => ResolveResponse {
                ticket: self.ticket,
                data: ResolveResponseData::Error(ResolveErrorResponse {
                    details: ErrorCodes::NONE,
                    msg: Some(format!("{}", e)),
                }),
            },
        };
        send_resolve(&response);
        if followup {
            self.run_followup(&instances);
        }
    }
}

/// Struct for followup resolves
pub struct ResolveFollowup {
    url: String,
    ticket: Ticket,
    start: usize,
    end: usize,
    instance: ID,
}

impl ResolveFollowup {
    fn state(&self, data: &Playlist) -> ResolveState {
        if data.songs.len() < self.end - self.start {
            ResolveState::Finished
        } else {
            ResolveState::Part
        }
    }

    fn wrap(self) -> Request {
        Request::Chunked(self)
    }

    /// Run followup part resolve
    fn run_followup(mut self, instances: &Instances) {
        let inst = instances.read().expect("Can't read instances!");
        if let Some(inst) = inst.get(&self.instance) {
            let start_new = self.end + 1;
            let end_new = self.end + (self.end - self.start);
            self.start = start_new;
            self.end = end_new;
            let ticket = self.ticket;
            if let Err(e) = inst.dispatch_resolve(self.wrap()) {
                error!("Unable to dispatch followup resolve: {}", e);
                send_resolve(&ResolveResponse {
                    ticket: ticket,
                    data: ResolveResponseData::Error(ResolveErrorResponse {
                        details: ErrorCodes::NONE,
                        msg: Some(String::from("Unable to re-queue part request")),
                    }),
                });
            }
        } else {
            debug!("Instance {} shut down, aborting resolve..", self.instance);
            send_resolve(&ResolveResponse {
                ticket: self.ticket,
                data: ResolveResponseData::Error(ResolveErrorResponse {
                    details: ErrorCodes::RESOLVE_CANCELLED,
                    msg: Some(String::from("Instance shut down")),
                }),
            });
        }
    }

    /// Url to resolve
    fn url(&self) -> &str {
        &self.url
    }

    /// Start,end of retrieval
    fn bounds(&self) -> (usize, Option<usize>) {
        (self.start, Some(self.end))
    }

    /// Callback, called after resolving of requested url with return value
    fn callback(self, resolved: PartSongs, instances: Instances) {
        let mut followup = false;
        let response = match resolved {
            Ok(playlist) => {
                let state = self.state(&playlist);
                followup = state == ResolveState::Part;
                ResolveResponse {
                    ticket: self.ticket,
                    data: ResolveResponseData::Part(ResolvePartResponse {
                        state: self.state(&playlist),
                        data: playlist,
                        position: self.start,
                    }),
                }
            }
            Err(e) => ResolveResponse {
                ticket: self.ticket,
                data: ResolveResponseData::Error(ResolveErrorResponse {
                    details: ErrorCodes::NONE,
                    msg: Some(format!("{}", e)),
                }),
            },
        };

        send_resolve(&response);
        if followup {
            self.run_followup(&instances);
        }
    }
}

/// Request
pub enum Request {
    /// Chunked retrieval, start-end
    Chunked(ResolveFollowup),
    /// Startup resolve, limit for first chunk
    Info(ResolveDispatcher),
}

/// Part request following up YTStart when resolving a bigger playlist
pub trait YTPart {
    /// Url to resolve
    fn url(&self) -> &str;
    /// Start,end of retrieval
    fn bounds(&self) -> (usize, Option<usize>);
    /// Callback, called after resolving of requested url with return value
    /// instance calls should be done via the instance map passed
    fn callback(&mut self, songs: PartSongs, instances: Instances);
}

/// Info request for initial resolve  
/// Can be extended by YTPart when resolving a playlist
pub trait YTInfo {
    /// Url to resolve
    fn url(&self) -> &str;
    /// Max amount of songs to retrieve
    fn bounds(&self) -> usize;
    /// Callback, called after resolving of requested url with return value
    /// instance calls should be done via the instance map passed
    /// can't take self due to https://github.com/rust-lang/rust/issues/28796
    fn callback(&mut self, songs: ReqSongs, instances: Instances);
}

/// Update scheduler for ytdl
pub fn crate_yt_updater(runtime: &mut Runtime, ytdl: Arc<YtDL>) {
    let ytdl = ytdl.clone();
    let updater = Interval::new_interval(Duration::from_secs(
        u64::from(SETTINGS.ytdl.update_intervall) * 3600,
    ))
    .for_each(move |_| {
        let _ = blocking(|| match ytdl.update_downloader() {
            Ok(_) => (),
            Err(e) => warn!("Error when updating ytdl: {}", e),
        });
        Ok(())
    })
    .map_err(|_| {});
    runtime.spawn(updater);
}

pub fn crate_ytdl_scheduler(
    runtime: &mut Runtime,
    ytdl: Arc<YtDL>,
    cache: SongCache,
    instances: Instances,
) -> Controller {
    let (controller, scheduler) = scheduler::Scheduler::new(
        SETTINGS.ytdl.workers as usize,
        move |req: Request| {
            let ytdl_c = ytdl.clone();
            let start = Instant::now();
            let result = scheduler_retrieve(cache.clone(), &ytdl_c, &req);
            let end = start.elapsed();
            debug!(
                "Request took {}{:03}ms to process",
                end.as_secs(),
                end.subsec_millis()
            );
            (req, result)
        },
        Some(move |(req, resolved): R| {
            let instances_c = instances.clone();
            match req {
                Request::Chunked(r) => match resolved {
                    // need to un-pack after passing to this section
                    Ok(ResolveType::Song(_)) => unreachable!(),
                    Ok(ResolveType::Playlist(p)) => r.callback(Ok(p), instances_c),
                    Err(e) => r.callback(Err(e), instances_c),
                },
                Request::Info(r) => r.callback(resolved, instances_c),
            }
        }),
        false,
    );

    runtime.spawn(scheduler);
    controller
}

/// Retrieve function for scheduler
/// query ytdl, update cache
/// returns all song IDs
fn scheduler_retrieve(cache: SongCache, ytdl: &YtDL, req: &Request) -> ReqSongs {
    let track_response = match req {
        Request::Chunked(req) => {
            let (start, end) = req.bounds();
            TrackResponse::TrackList(ytdl.get_tracks_multipart(req.url(), start, end)?)
        }
        Request::Info(req) => ytdl.get_url_info(req.url(), req.bounds())?,
    };
    match track_response {
        TrackResponse::TrackList(list) => {
            let pl = Playlist {
                id: list.get_id(),
                author: list.uploader,
                name: list.title,
                source: list.webpage_url,
                songs: list
                    .entries
                    .into_iter()
                    .filter_map(|t| {
                        let min_song = match t.best_audio_format(SETTINGS.ytdl.min_audio_bitrate) {
                            Some(v) => v.url.clone(),
                            None => {
                                warn!("No audio track for {}", t.webpage_url);
                                return None;
                            }
                        };

                        let song: Song = t.into();
                        cache.upsert(song.id.clone(), min_song);
                        Some(song)
                    })
                    .collect(),
            };
            Ok(ResolveType::Playlist(pl))
        }
        TrackResponse::Track(track) => {
            if let Request::Chunked(_) = req {
                // multipart request, got track, not tracklist
                return Err(YtDLWorkerErr::SongOnMultipart(track.webpage_url).into());
            }
            let min_song = match track.best_audio_format(SETTINGS.ytdl.min_audio_bitrate) {
                Some(v) => v.url.clone(),
                None => {
                    warn!("No audio track for {}", track.webpage_url);
                    return Err(YtDLWorkerErr::NoSongFound(track.webpage_url).into());
                }
            };

            let song: Song = track.into();
            cache.upsert(song.id.clone(), min_song);

            Ok(ResolveType::Song(song))
        }
    }
}

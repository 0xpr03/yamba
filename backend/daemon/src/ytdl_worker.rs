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

use failure::Fallible;
use futures::{Future, Stream};
use mpmc_scheduler as scheduler;
use mysql::Pool;
use tokio::runtime::Runtime;
use tokio::timer::Interval;
use tokio_threadpool::blocking;

use std::boxed::Box;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ytdl::YtDL;

use db;
use instance::{SongCache, ID};
use models::SongMin;
use SETTINGS;

/// Worker for ytdl tasks

pub type R = (YTReqWrapped, RSongs);
pub type YTReqWrapped = Box<dyn YTRequest + 'static + Send + Sync>;
pub type RSongs = Fallible<Vec<SongMin>>;
pub type Controller = scheduler::Controller<ID, YTReqWrapped, R>;
pub type YTSender = scheduler::Sender<YTReqWrapped>;

pub trait YTRequest {
    /// Force track over of playlist if possible
    fn is_force_track(&self) -> bool;
    /// Url to resolve
    fn url(&self) -> &str;
    /// Callback, called after resolving of requested url with return value
    fn callback(&mut self, RSongs);
    /// Turn YTRequest into wrapped to send to scheduler
    fn wrap(self) -> YTReqWrapped
    where
        Self: std::marker::Sized + Send + Sync + 'static,
    {
        Box::new(self)
    }
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
    pool: Pool,
    cache: SongCache,
) -> Controller {
    let (controller, scheduler) = scheduler::Scheduler::new(
        SETTINGS.ytdl.workers as usize,
        move |req: YTReqWrapped| {
            let ytdl_c = ytdl.clone();
            let start = Instant::now();
            let result = scheduler_retrieve(
                cache.clone(),
                &ytdl_c,
                &pool,
                req.is_force_track(),
                req.url(),
            );
            let end = start.elapsed();
            debug!(
                "Request {} took {}{:03}ms to process",
                req.url(),
                end.as_secs(),
                end.subsec_millis()
            );
            (req, result)
        },
        Some(|(mut req, tracks): R| {
            req.callback(tracks);
        }),
        false,
    );

    runtime.spawn(scheduler);
    controller
}

/// Retrieve function for scheduler
/// query ytdl, insert into db & update cache
/// returns all song IDs
fn scheduler_retrieve(
    cache: SongCache,
    ytdl: &YtDL,
    pool: &Pool,
    playlist: bool,
    url: &str,
) -> RSongs {
    let tracks = if playlist {
        ytdl.get_playlist_info(url)?
    } else {
        vec![ytdl.get_url_info(url)?]
    };
    let tracks = db::insert_tracks(Some(cache), tracks, &pool)?;

    Ok(tracks)
}

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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Instant;

use audio::NullSink;
use cache::Cache;
use daemon::WInstances;
use models::{CacheSong, SongID, SongMin};
use playback::Player;
use ts::TSInstance;
use ytdl::YtDL;
use ytdl_worker::{YTReqWrapped, YTSender};

/// module containing a single instance

const MAX_BACK_TITLES: usize = 30;

#[derive(Fail, Debug, PartialEq)]
pub enum InstanceErr {
    #[fail(display = "No Audio track for URL {}", _0)]
    NoAudioTrack(String),
    #[fail(display = "No YTDL result for URL {}", _0)]
    InvalidSource(String),
}

pub type ID = i32;
/// Cache for resolved media URIs
pub type SongCache = Cache<SongID, CacheSong>;
#[allow(non_camel_case_types)]
type CURRENT_SONG = Arc<RwLock<Option<CurrentSong>>>;

/// Type holding the current song
pub type CurrentSong = SongMin;

/// Base for each instance
pub struct Instance {
    pub id: ID,
    pub voip: InstanceType,
    pub player: Player,
    pub ytdl: Arc<YtDL>,
    pub current_song: CURRENT_SONG,
    pub cache: SongCache,
    pub instances: WInstances,
    pub url_resolve: YTSender,
}

impl Drop for Instance {
    fn drop(&mut self) {
        // don't store on clone drop
        println!("Storing instance {}", self.id);
        self.player.stop();
    }
}

impl Instance {
    /// Resolve URL under this instances queue
    pub fn dispatch_resolve(&self, request: YTReqWrapped) -> Fallible<()> {
        Ok(self.url_resolve.try_send(request)?)
    }

    /// Stop playback
    pub fn stop_playback(&self) {
        let mut lock = self.current_song.write().expect("Can't lock current song!");
        *lock = None;
        // don't store to history, still in queue, no end-of-stream triggered
        self.player.stop();
    }

    /// Handle end of stream event
    pub fn end_of_stream(&self) {
        // !stop-flag && no current song (avoid feedback loop)
        let has_current_song = self
            .current_song
            .read()
            .expect("Can't lock current song!")
            .is_some();

        if has_current_song {
            //TODO: send end of song event
        } else {
            trace!("Ignoring end of stream");
        }
    }

    /// Resume playback
    pub fn resume_playback(&self) -> Fallible<()> {
        if self
            .current_song
            .read()
            .expect("can't lock current song")
            .is_some()
        {
            self.player.play();
        }
        Ok(())
    }

    /// Get current playback info
    /// Contains track name, artist, current playback position, total length
    pub fn playback_info(&self) -> String {
        let song_guard = self.current_song.read().expect("Can't lock current track!");
        match song_guard.as_ref() {
            Some(cur_song) => {
                let position = self.player.get_position();
                let length = format_time(cur_song.length);
                let artist = match cur_song.artist.as_ref() {
                    Some(v) => format!(" - {}", v),
                    None => String::new(),
                };
                format!(
                    "{}{} {:02}:{:02} / {} {}",
                    cur_song.name.as_str(),
                    artist,
                    position.minutes + (position.hours * 60),
                    position.seconds,
                    length,
                    match self.player.is_paused() {
                        true => "-paused-",
                        false => "",
                    }
                )
            }
            None => String::from("Playback ended"),
        }
    }

    /// Play song
    pub fn play_track(&self, song: SongMin) -> Fallible<()> {
        let mut c_song_w = self.current_song.write().expect("Can't lock current song!");

        let source = song.source.clone();
        let songid = song.id.clone();
        *c_song_w = Some(song);
        let instances = self.instances.clone();
        let cache = self.cache.clone();
        let id = self.id.clone();
        let ytdl = self.ytdl.clone();
        thread::spawn(move || {
            if let Err(e) = Instance::play_track_inner(instances, cache, id, ytdl, source, songid) {
                warn!("Error while resolving next track! {}", e);
            }
        });

        Ok(())
    }

    /// Inner function, blocking
    /// Resolves the playback URI
    fn play_track_inner(
        instances: WInstances,
        cache: SongCache,
        id: ID,
        ytdl: Arc<YtDL>,
        source: String,
        song_id: SongID,
    ) -> Fallible<()> {
        let audio_url: String = if let Some(v) = cache.get(&song_id) {
            v
        } else {
            debug!("No cache entry for {}", song_id);
            let track = ytdl.get_url_info(source.as_str())?;
            let track = match track.get(0) {
                Some(t) => t,
                None => return Err(InstanceErr::InvalidSource(source).into()),
            };

            let url_temp = match track.best_audio_format() {
                Some(v) => v.url.clone(),
                None => return Err(InstanceErr::NoAudioTrack(source).into()),
            };
            cache.upsert(song_id, url_temp.clone());
            url_temp
        };

        let instances = match instances.upgrade() {
            Some(v) => v,
            None => return Ok(()),
        };
        let lock = instances.read().expect("Can't read instances!");
        if let Some(inst) = lock.get(&id) {
            inst.player.set_uri(audio_url.as_str());
        } else {
            warn!("Instance gone, ignoring playback resolver..");
        }

        Ok(())
    }
}

/// Format time by
#[inline(always)]
fn format_time(length: Option<u32>) -> String {
    match length {
        Some(v) => format!("{:02}:{:02}", v / 60, v % 60),
        None => String::from("--:--"),
    }
}

/// Instance type for different VoIP systems
pub enum InstanceType {
    Teamspeak(Teamspeak),
}

/// Teamspeak specific VoIP instance
pub struct Teamspeak {
    pub ts: TSInstance,
    pub sink: NullSink,
    pub mute_sink: Arc<NullSink>,
    pub updated: RwLock<Instant>,
}

impl Teamspeak {
    /// Setup call on successfull connection
    /// process_id is the real ts id, as the xvfb wrapper doesn't count
    pub fn on_connected(&self, process_id: u32) -> Fallible<()> {
        trace!("Setting monitor for ts");
        self.sink.set_monitor_for_process(process_id)?;
        trace!("Setting sink for ts");
        self.mute_sink.set_sink_for_process(process_id)?;
        Ok(())
    }
}

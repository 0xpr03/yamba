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

use arraydeque::{ArrayDeque, Wrapping};
use failure::Fallible;
use mysql::Pool;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Instant;

use audio::NullSink;
use cache::Cache;
use daemon::WInstances;
use db;
use models::SongID;
use models::{InstanceStorage, QueueID, SongMin};
use playback::Player;
use ts::TSInstance;
use ytdl::YtDL;
use ytdl_worker::YTSender;

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
pub type SongCache = Cache<SongID, String>;
#[allow(non_camel_case_types)]
type CURRENT_SONG = Arc<RwLock<Option<CurrentSong>>>;

/// Struct holding the current song
pub struct CurrentSong {
    pub song: SongMin,
    pub queue_id: QueueID,
}

/// Base for each instance
pub struct Instance {
    pub id: ID,
    pub voip: InstanceType,
    pub store: RwLock<InstanceStorage>,
    pub playback_history: Mutex<ArrayDeque<[CurrentSong; MAX_BACK_TITLES], Wrapping>>,
    pub player: Player,
    pub stop_flag: AtomicBool,
    pub pool: Pool,
    pub ytdl: Arc<YtDL>,
    pub current_song: CURRENT_SONG,
    pub cache: SongCache,
    pub ytdl_tx: YTSender,
    pub instances: WInstances,
}

impl Drop for Instance {
    fn drop(&mut self) {
        // don't store on clone drop
        println!("Storing instance {}", self.id);
        self.player.pause();
        if let Ok(mut lock) = self.store.write() {
            lock.volume = self.player.get_volume();

            match db::upsert_instance_storage(&*lock, &self.pool) {
                Ok(_) => (),
                Err(e) => error!("Unable to store instance {}", e),
            }
        }
    }
}

impl Instance {
    /// Play previous track, does nothing if no song in history
    pub fn play_previous_track(&self) -> Fallible<()> {
        let mut lock = self.playback_history.lock().expect("Can't lock history!");
        if let Some(song) = lock.pop_front() {
            db::add_previous_song_to_queue(&self.pool, &self.id, &song.song.id, &song.queue_id)?;
            self.player.stop();
            let mut c_song_w = self.current_song.write().expect("Can't lock current song!");
            *c_song_w = None;
            drop(c_song_w);
            drop(lock);
            // drop locks first, then call play_track with specified qID
            // allows for rewind of non-linear playback
            self.play_track(Some(song.queue_id))?;
        } else {
            info!("No previous song to play!");
        }
        Ok(())
    }

    /// Stop playback
    pub fn stop_playback(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
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
        let stop_flag = self.stop_flag.load(Ordering::Relaxed);
        trace!(
            "Stop flag: {} has_current_song_ {}",
            stop_flag,
            has_current_song
        );

        if !stop_flag && has_current_song {
            if let Err(e) = self.play_next_track() {
                warn!("Couldn't play next track in queue. {}", e);
            }
        } else {
            trace!("Ignoring end of stream");
        }
    }

    /// Resume playback
    pub fn resume_playback(&self) -> Fallible<()> {
        self.stop_flag.store(false, Ordering::Relaxed);
        if self
            .current_song
            .read()
            .expect("can't lock current song")
            .is_some()
        {
            self.player.play();
        } else {
            self.play_next_track()?;
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
                let length = format_time(cur_song.song.length);
                let artist = match cur_song.song.artist.as_ref() {
                    Some(v) => format!(" - {}", v),
                    None => String::new(),
                };
                format!(
                    "{}{} {:02}:{:02} / {} {}",
                    cur_song.song.name.as_str(),
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
            None => {
                if self.stop_flag.load(Ordering::Relaxed) {
                    String::from("Playback stopped")
                } else {
                    String::from("Playback ended")
                }
            }
        }
    }

    /// Clear queue
    pub fn clear_queue(&self) -> Fallible<()> {
        {
            let mut c_song_w = self.current_song.write().expect("Can't lock current song!");
            db::clear_queue(&self.pool, &self.id)?;
            *c_song_w = None;
        }
        self.player.stop();
        Ok(())
    }

    /// Play next track if queue_id is None, otherwise the specified track
    fn play_track(&self, queue_id: Option<QueueID>) -> Fallible<()> {
        let mut c_song_w = self.current_song.write().expect("Can't lock current song!");
        if let Some(song) = c_song_w.take() {
            trace!("Removing old song from queue {}", song.queue_id);
            db::remove_from_queue(&self.pool, &song.queue_id)?;
            let mut lock = self.playback_history.lock().expect("Can't lock history!");
            lock.push_front(song);
            drop(lock);
        }

        let song_data = match queue_id {
            Some(queue_id) => Some((queue_id, db::get_track_by_queue_id(&self.pool, &queue_id)?)),
            None => db::get_next_in_queue(&self.pool, &self.id)?,
        };

        if let Some((queue_id, song)) = song_data {
            trace!("Found new song, queue_id: {}", queue_id);
            let source = song.source.clone();
            let songid = song.id.clone();
            *c_song_w = Some(CurrentSong { queue_id, song });
            let instances = self.instances.clone();
            let cache = self.cache.clone();
            let id = self.id.clone();
            let ytdl = self.ytdl.clone();
            thread::spawn(move || {
                if let Err(e) =
                    Instance::play_track_inner(instances, cache, id, ytdl, source, songid)
                {
                    warn!("Error while resolving next track! {}", e);
                }
            });
        } else {
            trace!("play_track can't find any song");
            *c_song_w = None;
            self.player.stop();
        }

        Ok(())
    }

    /// Play next track in queue
    pub fn play_next_track(&self) -> Fallible<()> {
        self.play_track(None)
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
            inst.stop_flag.store(false, Ordering::Relaxed);
            inst.player.set_uri(audio_url.as_str());
        } else {
            warn!("Instance gone, ignoring playback resolver..");
        }

        Ok(())
    }

    /// Enqueue a set of SongMin
    pub fn enqueue_songs(&self, songs: Vec<SongMin>, url: &str) -> Fallible<()> {
        if songs.len() == 0 {
            return Err(InstanceErr::InvalidSource(url.to_string()).into());
        }

        let _ = songs
            .iter()
            .map(|s| db::add_song_to_queue(&self.pool, &self.id, &s.id))
            .collect::<Result<Vec<_>, _>>()?;

        let r_guard = self.current_song.read().expect("Can't lock current-song");

        let no_song = r_guard.is_none();
        drop(r_guard); // required for next step
        if no_song {
            self.play_next_track()?;
        }
        Ok(())
    }

    /// Get next n tracks in queue
    pub fn get_next_tracks_queue(&self, amount: &i32) -> Fallible<Vec<String>> {
        db::lookahead_queue_tracks(&self.pool, &self.id, amount).map(|val_ok| {
            val_ok
                .iter()
                .map(|x| {
                    format!(
                        "{}{} {}",
                        x.name,
                        x.artist
                            .as_ref()
                            .map(|art| format!(" - {}", art))
                            .unwrap_or(String::new()),
                        format_time(x.length)
                    )
                })
                .collect()
        })
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

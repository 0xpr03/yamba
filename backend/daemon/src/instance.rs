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
use mysql::Pool;

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Instant;

use audio::NullSink;
use db;
use models::{InstanceStorage, QueueID, SongMin};
use playback::Player;
use ts::TSInstance;
use ytdl::YtDL;

/// module containing a single instance

#[derive(Fail, Debug)]
enum InstanceErr {
    #[fail(display = "No Audio track for URL {}", _0)]
    NoAudioTrack(String),
    #[fail(display = "Unused {}", _0)]
    SomeErr(String),
}

pub type ID = Arc<i32>;
type CURRENT_SONG = Arc<RwLock<Option<CurrentSong>>>;

/// Struct holding the current song
pub struct CurrentSong {
    pub song: SongMin,
    pub queue_id: QueueID,
}

/// Base for each instance
#[derive(Clone)]
pub struct Instance {
    pub id: ID,
    pub voip: Arc<InstanceType>,
    pub store: Arc<RwLock<InstanceStorage>>,
    pub player: Arc<Player>,
    pub pool: Pool,
    pub ytdl: Arc<YtDL>,
    pub current_song: CURRENT_SONG,
}

impl Drop for Instance {
    fn drop(&mut self) {
        // don't store on clone drop
        if Arc::strong_count(&self.voip) <= 1 {
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
}

impl Instance {
    /// Enqueue track
    pub fn enqueue_by_url(&self, url: String) {
        let instance = self.clone();
        thread::spawn(
            move || match Instance::enqueue_by_url_inner(url, instance) {
                Err(e) => warn!("Couldn't enqueue song: {}\n{}", e, e.backtrace()),
                Ok(_) => (),
            },
        );
    }

    /// Inner function, blocking
    fn enqueue_by_url_inner(url: String, inst: Instance) -> Fallible<()> {
        let entry = db::get_track_by_url(&url, &inst.pool)?;

        let track = inst.ytdl.get_url_info(&url)?;
        let audio_url = match track.best_audio_format() {
            Some(v) => v.url.clone(),
            None => return Err(InstanceErr::NoAudioTrack(url).into()),
        };

        let entry = match entry {
            Some(e) => e,
            None => db::insert_track(track, &inst.pool)?,
        };

        let queue_id = db::add_song_to_queue(&inst.pool, &inst.id, &entry.id)?;

        let mut w_guard = inst.current_song.write().expect("Can't lock current-song");

        if w_guard.is_none() {
            debug!(
                "No current song, starting playback for {} qid {}",
                entry.id, queue_id
            );
            *w_guard = Some(CurrentSong {
                song: entry,
                queue_id,
            });

            drop(w_guard);

            inst.player.set_uri(&audio_url);
        }

        Ok(())
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

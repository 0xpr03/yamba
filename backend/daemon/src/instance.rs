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
use models::{InstanceStorage, SongMin};
use playback::Player;
use ts::TSInstance;

/// module containing a single instance

/// Base for each instance
#[derive(Clone)]
pub struct Instance {
    pub id: ID,
    pub voip: Arc<InstanceType>,
    pub store: Arc<RwLock<InstanceStorage>>,
    pub player: Arc<Player>,
    pub pool: Pool,
}

impl Drop for Instance {
    fn drop(&mut self) {
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

pub type ID = Arc<i32>;

/*
thread::spawn(move || {
                let entry = match db::get_track_by_url(&url, pool) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Couldn't search track in db: {}", e);
                        None
                    }
                };

                let tracks = match ytdl.get_url_info(&url) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Can't resolve URL {}: {}", url, e);
                        return;
                    }
                };

                if entry.is_none() {
                    //db::insert_tracks()
                }
            });
            instance.player.set_uri(v);

*/

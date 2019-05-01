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

use concurrent_hashmap::ConcHashMap;
use futures::{Future, Stream};
use tokio::runtime::Runtime;
use tokio::timer::Interval;

use super::{Instances, ID};

use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

const CHECK_INTERVAL: Duration = Duration::from_secs(3);

/// A guard for an instance that removes the heartbeat entry on drop
pub struct HeartBeatInstance {
    storage: Weak<ConcHashMap<ID, Instant>>,
    id: ID,
}

impl Drop for HeartBeatInstance {
    fn drop(&mut self) {
        if let Some(v) = self.storage.upgrade() {
            v.remove(&self.id);
        }
    }
}

#[derive(Clone)]
pub struct HeartbeatMap {
    storage: Arc<ConcHashMap<ID, Instant>>,
}

impl HeartbeatMap {
    /// Create new heartbeatmap & start handler
    pub fn new(instances: Instances, runtime: &mut Runtime) -> HeartbeatMap {
        let hbm = HeartbeatMap {
            storage: Arc::new(ConcHashMap::<ID, Instant>::new()),
        };

        let hbm_c = hbm.clone();
        runtime.spawn(
            Interval::new_interval(CHECK_INTERVAL)
                .for_each(move |_| {
                    let mut inst_rw = instances.write().expect("Can't lock instances!");
                    hbm_c
                        .get_entries_older_than(CHECK_INTERVAL)
                        .iter()
                        .for_each(move |id| {
                            warn!("Killing instance {}, timeout for heartbeat.", id);
                            inst_rw.remove(id);
                        });
                    Ok(())
                })
                .map_err(|e| {
                    error!("Interval errored: {:?}", e);
                    ()
                }),
        );

        hbm
    }

    /// Returns instance guard, removes instance from heartbeats on drop
    pub fn get_instance_guard(&self, id: ID) -> HeartBeatInstance {
        HeartBeatInstance {
            storage: Arc::downgrade(&self.storage),
            id,
        }
    }

    /// Update heartbeat timestamp for instance
    pub fn update(&self, id: ID) {
        trace!("Updating heartbeat for {}", id);
        self.storage.insert(id, Instant::now());
    }
    /// Get entries older than specified duration
    fn get_entries_older_than<'a>(&'a self, limit: Duration) -> Vec<&'a ID> {
        self.storage
            .iter()
            .filter_map(|(key, val)| match val.elapsed() >= limit {
                true => Some(key),
                false => None,
            })
            .collect()
    }
}

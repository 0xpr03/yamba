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

use super::ID;

use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct HeartbeatMap {
    storage: Arc<ConcHashMap<ID, Instant>>,
}

impl HeartbeatMap {
    pub fn new() -> HeartbeatMap {
        HeartbeatMap {
            storage: Arc::new(ConcHashMap::<ID, Instant>::new()),
        }
    }

    pub fn update(&self, id: ID) {
        self.storage.insert(id, Instant::now());
    }

    pub fn get_entries_older_than<'a>(&'a self, limit: Duration) -> Vec<&'a ID> {
        self.storage
            .iter()
            .filter_map(|(key, val)| match val.elapsed() >= limit {
                true => Some(key),
                false => None,
            })
            .collect()
    }
}

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

use concurrent_hashmap::ConcHashMap as HashMap;
use futures::Future;
use futures::Stream;
use tokio::runtime::Runtime;
use tokio::timer::Interval;

use std::clone::Clone;
use std::cmp::Eq;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::{Send, Sync};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Cache with aging entries
pub struct Cache<K, V>
where
    K: Sync + Send + Hash + Eq + Debug,
    V: Sync + Send,
{
    map: Arc<HashMap<K, CacheEntry<V>>>,
}

/// Entry inside the cache
#[derive(Debug)]
struct CacheEntry<V> {
    time: Instant,
    value: V,
}

use SETTINGS;

// https://github.com/rust-lang/rust/issues/26925
impl<K, V> Clone for Cache<K, V>
where
    K: Sync + Send + Hash + Eq + Debug,
    V: Sync + Send,
{
    fn clone(&self) -> Self {
        Cache {
            map: self.map.clone(),
        }
    }
}

impl<K, V> Cache<K, V>
where
    K: 'static + Sync + Send + Hash + Eq + Debug,
    V: 'static + Sync + Send + Clone,
{
    pub fn new(runtime: &mut Runtime) -> Cache<K, V> {
        let cache = Cache {
            map: Arc::new(HashMap::<K, CacheEntry<V>>::new()),
        };

        let c_cache = cache.clone();
        runtime.spawn(
            Interval::new_interval(Duration::from_secs(SETTINGS.main.cache_lifetime_secs / 2))
                .for_each(move |_| {
                    let c_cache = c_cache.clone();
                    // as iterating is documented as writer blocking we collect first, then remove
                    let outdated: Vec<&K> = c_cache
                        .map
                        .iter()
                        .filter_map(|(key, value)| {
                            if value.time.elapsed().as_secs() >= SETTINGS.main.cache_lifetime_secs {
                                Some(key)
                            } else {
                                None
                            }
                        })
                        .collect();
                    trace!("Found {} expired entries.", outdated.len());
                    outdated.into_iter().for_each(|key| {
                        c_cache.map.remove(key);
                    });
                    Ok(())
                })
                .map_err(|e| error!("cache cleanup error: {}", e)),
        );
        cache
    }

    /// Insert or update value for key
    pub fn upsert(&self, key: K, val: V) {
        trace!("Inserting cache entry for {:?}", key);
        self.map.insert(
            key,
            CacheEntry {
                value: val.clone(),
                time: Instant::now(),
            },
        );
    }

    /// Get entry in cache, checking it's age, copying the value
    pub fn get(&self, key: &K) -> Option<V> {
        match self.map.find(key) {
            Some(v) => {
                if v.get().time.elapsed().as_secs() < SETTINGS.main.cache_lifetime_secs {
                    trace!("Found cache entry for {:?}", key);
                    Some(v.get().value.clone())
                } else {
                    trace!("Cache entry for {:?} expired", key);
                    None
                }
            }
            None => {
                trace!("No cache entry for {:?}", key);
                None
            }
        }
    }
}

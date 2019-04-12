/*
 *  YAMBA manager
 *  Copyright (C) 2019 Aron Heinecke
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

use owning_ref::OwningRef;
use rand::seq::SliceRandom;
use rand::thread_rng;

use std::cmp::PartialEq;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::vec::Vec;

/// Playlist for generic type of title
pub struct Playlist<T> {
    list: RwLock<Vec<Item<T>>>,
    current_pos: RwLock<Option<usize>>,
    last_item_id: AtomicUsize,
}

/// Item in playlist, wrapper for position
/// Derefs to inner value
pub struct Item<T> {
    id: usize,
    val: T,
}

impl<T> Deref for Item<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.val
    }
}

impl<T> PartialEq for Item<T> {
    fn eq(&self, other: &Item<T>) -> bool {
        self.id == other.id
    }
}

pub type ItemReturn<'a, T> = Option<OwningRef<RwLockReadGuard<'a, Vec<Item<T>>>, Item<T>>>;

#[allow(unused)]
impl<T> Playlist<T>
where
    T: Sync, // rayon iter
{
    /// Create a new playlist
    pub fn new() -> Playlist<T> {
        Playlist {
            list: RwLock::new(Vec::new()),
            current_pos: RwLock::new(Option::None),
            last_item_id: AtomicUsize::new(0),
        }
    }

    /// Returns size of playlist
    pub fn size(&self) -> usize {
        let lst_r = self.list.read().expect("Can't lock list");
        lst_r.len()
    }

    fn get_pos<'a>(&'a self) -> OwningRef<RwLockReadGuard<'a, Option<usize>>, usize> {
        OwningRef::new(self.current_pos.read().expect("Can't lock position!")).map(|o| match o {
            Some(v) => v,
            None => &0,
        })
    }

    /// Get exact position
    fn get_pos_exact<'a>(&'a self) -> RwLockReadGuard<'a, Option<usize>> {
        self.current_pos.read().expect("Can't lock position!")
    }

    /// Get position, if no current positione exists, returns 0
    fn get_pos_mut<'a>(&'a self) -> RwLockWriteGuard<'a, Option<usize>> {
        self.current_pos.write().expect("Can't lock position!")
    }

    /// Returns amount of upcoming tracks
    pub fn amount_upcoming(&self) -> usize {
        let lst_r = self.list.read().expect("Can't lock list");
        lst_r.len().wrapping_sub(*self.get_pos())
    }

    /// (Re)Shuffle the playlist
    pub fn shuffle(&self) {
        let mut lst_w = self.list.write().expect("Can't lock list!");
        let mut pos = *self.get_pos();
        let length = lst_w.len();

        // non-playing playlist
        if pos > length {
            pos = 0;
        } else {
            // don't randomize current playback position
            pos += 1;
        }
        let mut upcoming = &mut lst_w[pos..length];
        upcoming.shuffle(&mut thread_rng());
    }

    /// Returns next n tracks
    pub fn get_next_tracks<'a>(
        &'a self,
        amount: usize,
    ) -> OwningRef<RwLockReadGuard<'a, Vec<Item<T>>>, [Item<T>]> {
        let lst_r = self.list.read().expect("Can't lock list");
        OwningRef::new(lst_r).map(|v| {
            let mut pos: usize = self.get_pos().clone();
            let mut end = pos.wrapping_add(amount);

            if v.len() == 0 {
                return &v[0..0];
            }

            if end >= v.len() {
                end = v.len() - 1;
            }

            // workaround, catch usize::MAX case
            if pos > end {
                pos = 0;
            }
            &v[pos..end]
        })
    }

    /// Get current position
    pub fn get_position(&self) -> usize {
        self.get_pos().clone()
    }

    /// Push track to back
    pub fn push(&self, values: Vec<T>) {
        let mut lst = self.list.write().expect("Can't lock list!'");
        values.into_iter().for_each(|v| {
            lst.push(Item {
                val: v,
                id: self.last_item_id.fetch_add(1, Ordering::SeqCst),
            })
        });
    }

    /// Insert track into playlist
    pub fn insert(&self, i: usize, v: T) {
        let mut pos_opt = self.get_pos_mut();
        if let Some(mut pos) = *pos_opt {
            if pos <= i {
                pos += 1;
            }
        }

        let mut lst = self.list.write().expect("Can't lock list!'");
        lst.insert(
            i,
            Item {
                val: v,
                id: self.last_item_id.fetch_add(1, Ordering::SeqCst),
            },
        );
    }

    /// Get current track
    pub fn get_current<'a>(&'a self) -> ItemReturn<T> {
        self.get_item(*self.get_pos())
    }

    /// Get next track, updating current position
    pub fn get_next(&self, repeat: bool) -> ItemReturn<T> {
        let lst = self.list.read().expect("Can't lock list!");
        let mut pos_mut = self.get_pos_mut();
        if lst.len() == 0 {
            return None;
        }

        let pos = match *pos_mut {
            Some(v) => {
                if !repeat && v >= lst.len() {
                    *pos_mut = None;
                    return None;
                }
                let pos_new = v.wrapping_add(1);
                *pos_mut = Some(pos_new);
                pos_new
            }
            None => {
                *pos_mut = Some(0);
                0
            }
        };

        drop(pos_mut);
        let lst_r = self.list.read().expect("Can't lock list");
        OwningRef::new(lst_r)
            .try_map(|v| match v.get(pos) {
                Some(v) => Ok(v),
                None => Err(()),
            })
            .ok()
    }

    /// Get item at position
    fn get_item<'a>(&'a self, pos: usize) -> ItemReturn<T> {
        let lst_r = self.list.read().expect("Can't lock list");
        OwningRef::new(lst_r)
            .try_map(|v| match v.get(pos) {
                Some(v) => Ok(v),
                None => Err(()),
            })
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    #[test]
    fn get_next_test() {
        let playlist = Playlist::new();
        let vec: Vec<_> = (0..5).collect();
        playlist.push(vec);
        {
            assert_eq!(0, **playlist.get_next(false).unwrap());
            assert_eq!(1, **playlist.get_next(false).unwrap());
            assert_eq!(2, **playlist.get_next(false).unwrap());
            assert_eq!(3, **playlist.get_next(false).unwrap());
            assert_eq!(4, **playlist.get_next(false).unwrap());
            assert!(playlist.get_next(false).is_none());
            assert_eq!(true, playlist.get_next_tracks(1).is_empty());
        }
        let vec: Vec<_> = (5..6).collect();
        playlist.push(vec);
        dbg!(playlist.get_position());
        dbg!(playlist.size());
        assert_eq!(5, **playlist.get_next(false).unwrap());
        assert!(playlist.get_next(false).is_none());
        assert!(playlist.get_next(false).is_none());
    }

    #[test]
    fn shuffle() {
        let playlist = Playlist::new();
        playlist.shuffle();
        // add 5 items
        let vec: Vec<_> = (0..5).collect();
        let mut set = HashSet::new();
        for val in vec.iter() {
            assert!(set.insert(val.clone()));
        }

        playlist.push(vec);
        // get first
        assert_eq!(0, **playlist.get_next(false).unwrap());
        assert!(set.remove(&0));
        playlist.shuffle();
        // current still first..
        assert_eq!(0, **playlist.get_current().unwrap());
        for _ in 0..4 {
            // go forard, shuffle again..
            // make sure the right values are still inside
            let val = playlist.get_next(false).unwrap();
            assert!(set.remove(&**val));
            drop(val); // shuffle will block otherwise
            playlist.shuffle();
        }
        assert!(playlist.get_next(false).is_none());
        assert!(set.is_empty());
    }

    #[test]
    fn init() {
        let playlist = Playlist::new();
        assert!(
            playlist.get_next(false).is_none(),
            "get_next on empty playlist should be none"
        );
        assert!(
            playlist.get_current().is_none(),
            "get_current on empty list should be none"
        );
        playlist.shuffle();
        assert_eq!(true, playlist.get_next_tracks(1).is_empty());
        let vec: Vec<_> = (0..5).collect();
        playlist.push(vec);
        assert_eq!(
            0,
            **playlist.get_next(false).unwrap(),
            "First element after inserting incorrect"
        );
    }
}

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
use rayon::iter::*;

use std::cmp::PartialEq;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{RwLock, RwLockReadGuard};
use std::vec::Vec;

/// Playlist for generic type of title
pub struct Playlist<T> {
    list: RwLock<Vec<Item<T>>>,
    current_pos: AtomicUsize,
    last_id: AtomicUsize,
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

impl<T> Playlist<T>
where
    T: Sync, // rayon iter
{
    /// Create a new playlist
    pub fn new() -> Playlist<T> {
        Playlist {
            list: RwLock::new(Vec::new()),
            current_pos: AtomicUsize::new(0),
            last_id: AtomicUsize::new(0),
        }
    }

    /// (Re)Shuffle the playlist
    pub fn shuffle(&self) {
        let mut lst_w = self.list.write().expect("Can't lock list!");
        let item_id = lst_w
            .get(self.current_pos.load(Ordering::Relaxed))
            .map(|v| v.id);
        (*lst_w).shuffle(&mut thread_rng());

        if let Some(item_id) = item_id {
            let (pos_new, _) = (*lst_w)
                .par_iter()
                .enumerate()
                .find_any(|&(_, v)| v.id == item_id) // expect no overlapping items
                .unwrap();

            self.current_pos.store(pos_new, Ordering::Relaxed);
        }
    }

    /// Push track to back
    pub fn push(&self, values: Vec<T>) {
        let mut lst = self.list.write().expect("Can't lock list!'");
        values.into_iter().for_each(|v| {
            lst.push(Item {
                val: v,
                id: self.last_id.fetch_add(1, Ordering::SeqCst),
            })
        });
    }

    /// Insert track into playlist
    pub fn insert(&self, i: usize, v: T) {
        if self.current_pos.load(Ordering::Relaxed) <= i {
            self.current_pos.fetch_add(1, Ordering::SeqCst);
        }
        let mut lst = self.list.write().expect("Can't lock list!'");
        lst.insert(
            i,
            Item {
                val: v,
                id: self.last_id.fetch_add(1, Ordering::SeqCst),
            },
        );
    }

    /// Get current track
    pub fn get_current<'a>(&'a self) -> ItemReturn<T> {
        let pos = self.current_pos.load(Ordering::Relaxed);
        self.get_item(pos)
    }

    /// Get next track, updating current position
    pub fn get_next(&self) -> ItemReturn<T> {
        let pos = self.current_pos.fetch_add(1, Ordering::SeqCst);
        let lst = self.list.read().expect("Can't lock list!");
        self.get_item(pos)
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

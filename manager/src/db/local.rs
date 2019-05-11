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
use failure::{Error, Fallible};
use sled::{self, *};

use std::sync::Arc;

use super::Database;
use crate::models::*;
use bincode::{deserialize, serialize};
use yamba_types::models::{Song, SongID, TimeStarted, ID};

/// Instance data storage
const TREE_INSTANCES: &'static str = "instances";
/// Startup times for instances
const TREE_STARTUP_TIMES: &'static str = "startup_times";
const TREE_META: &'static str = "meta";
/// Raw Songs
const TREE_SONGS: &'static str = "songs";
/// Match URL <-> Song
const TREE_SONG_URL: &'static str = "songs_url";
/// Playlist storage
const TREE_PLAYLISTS: &'static str = "playlists";
/// Match URL <-> Playlist
const TREE_PLAYLIST_URL: &'static str = "playlists_url";

const KEY_VERSION: &'static str = "DB_VERSION";
const DB_VERSION: &'static str = "0.0.2";
const KEY_INSTANCE_ID: &'static str = "INSTANCE_ID";
const INSTANCE_ID_ZERO: ID = 0;

#[derive(Fail, Debug)]
pub enum LocalDBErr {
    #[fail(display = "Failed to start local DB {}", _0)]
    StartFailed(#[cause] sled::Error),
    #[fail(display = "Failed to open tree {}: {}", _1, _0)]
    TreeOpenFailed(#[cause] sled::Error, &'static str),
    #[fail(display = "Failed to open DB due to invalid version! {}", _0)]
    InvalidVersion(String),
    #[fail(display = "No value found for key!")]
    NoValueFound,
}

#[derive(Clone)]
pub struct DB {
    db: sled::Db,
}

impl Database for DB {
    type DB = DB;
    fn create(path: String) -> Fallible<DB> {
        let config = ConfigBuilder::default().path(path).build();

        let db = DB {
            db: Db::start(config).map_err(|e| LocalDBErr::StartFailed(e))?,
        };
        if let Err(e) = db.check_version() {
            error!("Failed DB version check: {}", e);
            error!("Please delete you Database!");
            Err(e)
        } else {
            Ok(db)
        }
    }
    fn get_instance(&self, id: ID) -> Fallible<Instance> {
        let tree = self.open_tree(TREE_INSTANCES)?;
        match tree.get(id.to_le_bytes()) {
            Ok(Some(v)) => Ok(deserialize::<Instance>(&v)?),
            Err(e) => Err(e.into()),
            Ok(None) => Err(LocalDBErr::NoValueFound.into()),
        }
    }
    fn get_instances(&self, is_autostart: bool) -> Fallible<Vec<Instance>> {
        let tree = self.open_tree(TREE_INSTANCES)?;
        Ok(tree
            .iter()
            .filter_map(|r| {
                let (_, v) = match r {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e.into())),
                };
                match deserialize::<Instance>(&v) {
                    Ok(v) => {
                        if v.autostart || !is_autostart {
                            Some(Ok(v))
                        } else {
                            None
                        }
                    }
                    Err(e) => Some(Err(e.into())),
                }
            })
            .collect::<std::result::Result<Vec<_>, Error>>()?)
    }
    fn create_instance(&self, new_instance: NewInstance) -> Fallible<Instance> {
        let id = self.gen_instance_id()?;
        let instance = Instance::from_new_instance(new_instance, id);
        let tree = self.open_tree(TREE_INSTANCES)?;
        tree.set(serialize(&id).unwrap(), serialize(&instance).unwrap())?;
        Ok(instance)
    }
    fn get_instance_startup(&self, instance: &ID) -> Fallible<Option<TimeStarted>> {
        Ok(self
            .open_tree(TREE_STARTUP_TIMES)?
            .get(serialize(instance).unwrap())?
            .map(|v| deserialize::<TimeStarted>(&v).unwrap()))
    }
    fn set_instance_startup(&self, instance: &ID, time: &Option<TimeStarted>) -> Fallible<()> {
        let tree = self.open_tree(TREE_STARTUP_TIMES)?;
        let serialized = serialize(instance)?;
        match time {
            Some(v) => {
                tree.set(serialized, serialize(v)?)?;
            }
            None => {
                tree.del(serialized)?;
            }
        }
        Ok(())
    }
    fn upsert_song(&self, song: &Song, url: &Option<&str>) -> Fallible<()> {
        let tree = self.open_tree(TREE_SONGS)?;
        let id = serialize(&song.id)?;
        tree.set(serialize(&id)?, serialize(&song)?)?;
        if let Some(url) = url {
            self.open_tree(TREE_SONG_URL)?
                .set(serialize(url)?, serialize(&id)?)?;
        }
        Ok(())
    }
    fn get_song(&self, song: SongID) -> Fallible<Option<Song>> {
        let tree = self.open_tree(TREE_SONGS)?;
        Ok(tree.get(song)?.map(|v| deserialize::<Song>(&v).unwrap()))
    }
    fn get_song_by_url(&self, url: &str) -> Fallible<Option<Song>> {
        let tree_url = self.open_tree(TREE_SONG_URL)?;
        if let Some(id) = tree_url.get(url)? {
            let tree_songs = self.open_tree(TREE_SONGS)?;
            match tree_songs.get(&id)? {
                Some(s) => return Ok(Some(deserialize::<Song>(&s)?)),
                None => {
                    // no song for ID found, delete wrong mapping
                    warn!("Inconsitent DB! No song for stored URL found!");
                    tree_url.del(id)?;
                }
            }
        }
        Ok(None)
    }
    fn upsert_playlist(&self, playlist: &NewPlaylistData) -> Fallible<()> {
        let tree = self.open_tree(TREE_PLAYLISTS)?;
        let id = playlist.id.to_le_bytes();
        tree.set(&id, serialize(playlist)?)?;
        if let Some(url) = playlist.source {
            self.open_tree(TREE_PLAYLIST_URL)?
                .set(serialize(url)?, &id)?;
        }
        Ok(())
    }
    fn get_playlist_by_url(&self, url: &str) -> Fallible<Option<PlaylistData>> {
        let tree_url = self.open_tree(TREE_PLAYLIST_URL)?;
        if let Some(id) = tree_url.get(serialize(url)?)? {
            let tree_pl = self.open_tree(TREE_PLAYLISTS)?;
            match tree_pl.get(&id)? {
                Some(pl) => return Ok(Some(deserialize::<PlaylistData>(&pl)?)),
                None => {
                    // No playlist but mapping URL->ID in DB
                    warn!("Inconsitent DB! No playlist for stored URL found!");
                    tree_pl.del(id)?;
                }
            }
        }
        Ok(None)
    }
}

type WTree = Arc<Tree>;

impl DB {
    /// Generate ID, used for manual ID creation
    /// (playlist creation workaround)
    pub fn generate_id(&self) -> Fallible<u64> {
        Ok(self.db.generate_id()?)
    }

    /// Check version of DB
    fn check_version(&self) -> Fallible<()> {
        let tree = self.open_tree(TREE_META)?;

        if let Some(v) = tree.get(KEY_VERSION)? {
            match deserialize::<String>(&v) {
                Ok(v) => {
                    if v != DB_VERSION {
                        return Err(LocalDBErr::InvalidVersion(format!(
                            "Invalid DB version found {} != {}!",
                            v, DB_VERSION
                        ))
                        .into());
                    }
                }
                Err(e) => {
                    return Err(LocalDBErr::InvalidVersion(format!(
                        "Invalid version type in DB?! {}",
                        e
                    ))
                    .into());
                }
            }
        } else {
            tree.set(KEY_VERSION, serialize(DB_VERSION).unwrap())?;
        }
        Ok(())
    }

    fn gen_instance_id(&self) -> Fallible<ID> {
        self.gen_id(KEY_INSTANCE_ID, INSTANCE_ID_ZERO)
    }

    /// Generate new instance ID
    fn gen_id(&self, key: &'static str, zero: ID) -> Fallible<ID> {
        let old = self
            .open_tree(TREE_META)?
            .fetch_and_update(key, |v| match v {
                // Basically fetch_add in sled style
                Some(v) => Some(serialize(&(deserialize::<ID>(&v).unwrap() + 1)).unwrap()),
                None => Some(serialize(&zero).unwrap()),
            })?;

        Ok(match old {
            Some(v) => deserialize::<ID>(&v).unwrap(),
            None => 0,
        })
    }

    /// Open tree with wrapped error
    fn open_tree(&self, tree: &'static str) -> Fallible<WTree> {
        Ok(self
            .db
            .open_tree(tree)
            .map_err(|e| LocalDBErr::TreeOpenFailed(e, tree))?)
    }
}

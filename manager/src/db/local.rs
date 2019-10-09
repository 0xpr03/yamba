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
const DB_VERSION: &'static str = "0.0.3";
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
    fn create_instance(&self, new_instance: InstanceCore) -> Fallible<Instance> {
        let id = self.gen_instance_id()?;
        let instance = Instance::from_new_instance(new_instance, id);
        let tree = self.open_tree(TREE_INSTANCES)?;
        tree.insert(
            id.to_le_bytes(),
            serialize(&InstanceRef::from_instance(&instance)).unwrap(),
        )?;
        Ok(instance)
    }
    fn update_instance(&self, instance: &InstanceRef) -> Fallible<()> {
        let id = instance.id.clone();
        let tree = self.open_tree(TREE_INSTANCES)?;
        tree.insert(id.to_le_bytes(), serialize(instance).unwrap())?;
        Ok(())
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
                tree.insert(serialized, serialize(v)?)?;
            }
            None => {
                tree.remove(serialized)?;
            }
        }
        Ok(())
    }
    fn upsert_song(&self, song: &Song, url: &Option<&str>) -> Fallible<()> {
        let tree = self.open_tree(TREE_SONGS)?;
        let id = song.id.as_str();
        tree.insert(id, serialize(&song)?)?;
        if let Some(url) = url {
            self.open_tree(TREE_SONG_URL)?.insert(url, id)?;
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
                    tree_url.remove(id)?;
                }
            }
        }
        Ok(None)
    }
    fn upsert_playlist(&self, playlist: &NewPlaylistData) -> Fallible<()> {
        let tree = self.open_tree(TREE_PLAYLISTS)?;
        let id = playlist.id.to_le_bytes();
        tree.insert(&id, serialize(playlist)?)?;
        if let Some(url) = playlist.source {
            self.open_tree(TREE_PLAYLIST_URL)?.insert(url, &id)?;
        }
        Ok(())
    }
    fn get_playlist_by_url(&self, url: &str) -> Fallible<Option<PlaylistData>> {
        let tree_url = self.open_tree(TREE_PLAYLIST_URL)?;
        if let Some(id) = tree_url.get(url)? {
            let tree_pl = self.open_tree(TREE_PLAYLISTS)?;
            match tree_pl.get(&id)? {
                Some(pl) => return Ok(Some(deserialize::<PlaylistData>(&pl)?)),
                None => {
                    // No playlist but mapping URL->ID in DB
                    warn!("Inconsitent DB! No playlist for stored URL found!");
                    tree_pl.remove(id)?;
                }
            }
        }
        Ok(None)
    }
    fn delete_playlist(&self, id: PlaylistID) -> Fallible<()> {
        let tree_pl = self.open_tree(TREE_PLAYLISTS)?;
        match tree_pl.get(id.to_le_bytes())? {
            Some(pl) => {
                let playlist = deserialize::<PlaylistData>(&pl)?;
                if let Some(url) = playlist.source {
                    let tree_url = self.open_tree(TREE_PLAYLIST_URL)?;
                    tree_url.remove(url)?;
                }
                tree_pl.remove(id.to_le_bytes())?;
                Ok(())
            }
            None => Err(LocalDBErr::NoValueFound.into()),
        }
    }
}

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
            tree.insert(KEY_VERSION, serialize(DB_VERSION).unwrap())?;
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
                None => Some(serialize(&(zero + 1)).unwrap()),
            })?;

        Ok(match old {
            Some(v) => deserialize::<ID>(&v).unwrap(),
            None => zero,
        })
    }

    /// Open tree with wrapped error
    fn open_tree(&self, tree: &'static str) -> Fallible<Tree> {
        Ok(self
            .db
            .open_tree(tree)
            .map_err(|e| LocalDBErr::TreeOpenFailed(e, tree))?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::tempdir;
    /// Test header creation
    #[test]
    fn test_init() {
        let tmp_dir = tempdir().unwrap();
        {
            let db = DB::create(format!("{}/db", tmp_dir.path().to_string_lossy())).unwrap();
            drop(db);
            DB::create(format!("{}/db", tmp_dir.path().to_string_lossy())).unwrap();
        }
    }
    #[test]
    fn test_instance_storage() {
        let tmp_dir = tempdir().unwrap();
        {
            let db = DB::create(format!("{}/db", tmp_dir.path().to_string_lossy())).unwrap();
            let instance = Instance {
                id: 0,
                host: String::from("my host"),
                port: Some(1),
                identity: Some(String::from("asd")),
                cid: Some(1),
                name: String::from("asd"),
                password: Some(String::from("asd")),
                autostart: true,
                volume: 1.0,
                nick: String::from("some nick"),
            };
            db.update_instance(&InstanceRef::from_instance(&instance))
                .unwrap();
            let inst_read = db.get_instance(0).unwrap();
            assert_eq!(instance, inst_read);
        }
    }
    #[test]
    fn test_song_insert() {
        let tmp_dir = tempdir().unwrap();
        {
            let db = DB::create(format!("{}/db", tmp_dir.path().to_string_lossy())).unwrap();
            let song = Song {
                id: String::from("asd"),
                name: String::from("test"),
                source: String::from("my_source"),
                artist: Some(String::from("some_artist")),
                length: Some(123),
            };
            let url = Some(song.source.as_str());
            db.upsert_song(&song, &url).unwrap();
            assert!(db.get_song_by_url(song.source.as_str()).unwrap().is_some());
            assert!(db.get_song(song.id).unwrap().is_some());
        }
    }
    #[test]
    fn test_playlist_insert() {
        let tmp_dir = tempdir().unwrap();
        {
            let db = DB::create(format!("{}/db", tmp_dir.path().to_string_lossy())).unwrap();
            let song = Song {
                id: String::from("asd"),
                name: String::from("test"),
                source: String::from("my_source"),
                artist: Some(String::from("some_artist")),
                length: Some(123),
            };
            let mut songs = Vec::new();
            songs.push(song);
            let playlist = PlaylistData::new(
                String::from("my_playlist"),
                Some(String::from("source")),
                songs,
                &db,
            )
            .unwrap();
            let url: &str = playlist.source.as_ref().unwrap().as_str();
            db.upsert_playlist(&NewPlaylistData::from_playlist(&playlist))
                .unwrap();
            assert!(db.get_playlist_by_url(url).unwrap().is_some());
            db.delete_playlist(playlist.id).unwrap();
            assert!(db.get_playlist_by_url(url).unwrap().is_none());
        }
    }
}

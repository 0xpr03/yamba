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

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use super::Database;
use crate::models::*;
use bincode::{deserialize, serialize};
use yamba_types::models::ID;

const TREE_INSTANCES: &'static str = "instances";
const TREE_META: &'static str = "meta";
const TREE_SONGS: &'static str = "songs";
const TREE_PLAYLISTS: &'static str = "playlists";

const KEY_VERSION: &'static str = "DB_VERSION";
const DB_VERSION: &'static str = "0.0.1";
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
}

type WTree = Arc<Tree>;

impl DB {
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
                        "Invalid version type in DB {}",
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

    /// Generate new instance ID
    fn gen_instance_id(&self) -> Fallible<ID> {
        let old = self
            .open_tree(TREE_META)?
            .fetch_and_update(KEY_INSTANCE_ID, |v| match v {
                // Basically fetch_add in sled style
                Some(v) => Some(serialize(&(deserialize::<ID>(&v).unwrap() + 1)).unwrap()),
                None => Some(serialize(&INSTANCE_ID_ZERO).unwrap()),
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

/*
 *  YAMBA types
 *  Copyright (C) 2019 Aron Heinecke
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

/*
 * Manager-RS Files
 */

use crate::db::DB;
use failure::Fallible;
use serde::{Deserialize, Serialize};
use yamba_types::models::{InstanceLoadReq, InstanceType, Song, TSSettings};
use yamba_types::{TimeMS, Volume, ID};

pub type PlaylistID = u64;

/// Used for creating instances in manager-rs
#[derive(Debug, Deserialize)]
pub struct InstanceCore {
    pub host: String,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub identity: Option<String>,
    #[serde(default)]
    pub cid: Option<i32>,
    /// Instance name
    pub name: String,
    #[serde(default)]
    pub password: Option<String>,
    pub autostart: bool,
    /// VoIP identity nick
    pub nick: String,
}

impl InstanceCore {
    /// Turns this instance core into a InstanceLoadReq.  
    /// Returns InstanceLoadReq,name,autostart
    #[allow(non_snake_case)]
    pub fn into_InstanceLoadReq(self, id: ID, volume: Volume) -> (InstanceLoadReq, String, bool) {
        (
            InstanceLoadReq {
                id: id,
                volume,
                data: InstanceType::TS(TSSettings {
                    host: self.host,
                    port: self.port,
                    identity: self.identity,
                    cid: self.cid,
                    name: self.nick,
                    password: self.password,
                }),
            },
            self.name,
            self.autostart,
        )
    }
}

/// Reference version for serialization
#[derive(Debug, Serialize)]
pub struct InstanceCoreRef<'a> {
    pub host: &'a str,
    #[serde(default)]
    pub port: &'a Option<u16>,
    #[serde(default)]
    pub identity: &'a Option<String>,
    #[serde(default)]
    pub cid: &'a Option<i32>,
    /// Instance name
    pub name: &'a str,
    #[serde(default)]
    pub password: &'a Option<String>,
    pub autostart: bool,
    /// VoIP identity nick
    pub nick: &'a str,
}

impl<'a> InstanceCoreRef<'a> {
    /// Used to send instance core settings via frontend API
    pub fn from_load_request(name: &'a str, autostart: bool, model: &'a InstanceLoadReq) -> Self {
        match &model.data {
            InstanceType::TS(ts) => {
                InstanceCoreRef {
                    host: &ts.host,
                    port: &ts.port,
                    identity: &ts.identity,
                    cid: &ts.cid,
                    /// Instance name
                    name,
                    password: &ts.password,
                    autostart,
                    /// VoIP identity nick
                    nick: &ts.name,
                }
            }
        }
    }
}

/// PlaylistData for DB retrieval
#[derive(Debug, Deserialize)]
pub struct PlaylistData {
    pub id: PlaylistID,
    pub name: String,
    pub source: Option<String>,
    /// Data of playlist
    pub data: Vec<Song>,
}

impl PlaylistData {
    pub fn new(name: String, source: Option<String>, data: Vec<Song>, db: &DB) -> Fallible<Self> {
        Ok(PlaylistData {
            id: db.generate_id()?,
            name,
            data,
            source,
        })
    }
}

/// New playlist Data, used for insertion
#[derive(Debug, Serialize)]
pub struct NewPlaylistData<'a> {
    pub id: PlaylistID,
    pub name: &'a str,
    pub source: Option<&'a str>,
    /// Data of playlist
    pub data: &'a [Song],
}

impl<'a> NewPlaylistData<'a> {
    pub fn new(
        name: &'a str,
        source: Option<&'a str>,
        data: &'a [Song],
        db: &DB,
    ) -> Fallible<Self> {
        Ok(NewPlaylistData {
            id: db.generate_id()?,
            name,
            data,
            source,
        })
    }

    pub fn from_playlist(data: &'a PlaylistData) -> NewPlaylistData<'a> {
        NewPlaylistData {
            id: data.id,
            name: data.name.as_str(),
            data: &data.data,
            source: data.source.as_ref().map_or(None, |v| Some(v.as_str())),
        }
    }
}

/// Instance model, contains data for creating an instance
#[derive(Debug, Deserialize, PartialEq)]
pub struct Instance {
    pub id: ID,
    pub host: String,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub identity: Option<String>,
    #[serde(default)]
    pub cid: Option<i32>,
    /// Instance name
    pub name: String,
    #[serde(default)]
    pub password: Option<String>,
    pub autostart: bool,
    pub volume: Volume,
    /// VoIP identity nick
    pub nick: String,
}

/// Reference version for serialization
#[derive(Debug, Serialize)]
pub struct InstanceRef<'a> {
    pub id: ID,
    pub host: &'a str,
    #[serde(default)]
    pub port: &'a Option<u16>,
    #[serde(default)]
    pub identity: &'a Option<String>,
    #[serde(default)]
    pub cid: &'a Option<i32>,
    /// Instance name
    pub name: &'a str,
    #[serde(default)]
    pub password: &'a Option<String>,
    pub autostart: bool,
    pub volume: Volume,
    /// VoIP identity nick
    pub nick: &'a str,
}

impl<'a> InstanceRef<'a> {
    pub fn from_instance(model: &'a Instance) -> Self {
        InstanceRef {
            id: model.id,
            host: model.host.as_str(),
            port: &model.port,
            identity: &model.identity,
            cid: &model.cid,
            name: model.name.as_str(),
            password: &model.password,
            autostart: model.autostart,
            volume: model.volume,
            nick: model.nick.as_str(),
        }
    }
    /// Create InstanceRef from InstanceLoadReq  
    /// Volume is overriden to be more up to date.
    #[allow(non_snake_case)]
    pub fn from_InstanceLoadReq(
        name: &'a str,
        volume: Volume,
        autostart: bool,
        origin: &'a InstanceLoadReq,
    ) -> Self {
        match &origin.data {
            InstanceType::TS(ts) => InstanceRef {
                id: origin.id,
                host: ts.host.as_str(),
                port: &ts.port,
                identity: &ts.identity,
                cid: &ts.cid,
                name,
                password: &ts.password,
                autostart,
                volume,
                nick: &ts.name,
            },
        }
    }
}

impl Instance {
    /// Create Instance from NewInstance + ID
    pub fn from_new_instance(new: InstanceCore, id: ID) -> Self {
        Instance {
            id,
            host: new.host,
            port: new.port,
            identity: new.identity,
            cid: new.cid,
            name: new.name,
            password: new.password,
            autostart: new.autostart,
            volume: 0.05,
            nick: new.nick,
        }
    }

    /// Turn Model into InstanceLoadReq
    /// Returns also the Name & autostart
    #[allow(non_snake_case)]
    pub fn into_InstanceLoadReq(self) -> (InstanceLoadReq, String, bool) {
        (
            InstanceLoadReq {
                id: self.id,
                volume: self.volume,
                data: InstanceType::TS(TSSettings {
                    host: self.host,
                    port: self.port,
                    identity: self.identity,
                    cid: self.cid,
                    name: self.nick,
                    password: self.password,
                }),
            },
            self.name,
            self.autostart,
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenericRequest {
    pub instance: ID,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UseInstance {
    pub id: ID,
}

/// Send to client for overview of instances
pub type Instances = Vec<InstanceMin>;

/// Minimum representation of an Instance
#[derive(Debug, Deserialize, Serialize)]
pub struct InstanceMin {
    pub id: ID,
    pub running: bool,
    pub name: String,
    pub autostart: bool,
}

/// Send to client on initial connect
#[derive(Debug, Deserialize, Serialize)]
pub struct Playback {
    pub playing: bool,
    pub position: TimeMS,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackMin {
    pub title: String,
    pub artist: Option<String>,
    pub length: Option<TimeMS>,
}

impl TrackMin {
    pub fn from_song(song: &Song) -> Self {
        TrackMin {
            title: song.name.clone(),
            artist: song.artist.clone(),
            length: song.length.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VolumeFull {
    pub current: Volume,
    pub max: Volume,
}

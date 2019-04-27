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

#[cfg(any(feature = "maria", feature = "postgres"))]
use crate::db::schema::*;
use serde::{Deserialize, Serialize};
use yamba_types::models::{InstanceLoadReq, InstanceType, Song, TSSettings};
use yamba_types::{TimeMS, Volume, ID};

/// Used for creating instances in manager-rs
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(any(feature = "maria", feature = "postgres"), table_name = "instances")]
#[cfg_attr(any(feature = "maria", feature = "postgres"), derive(Insertable))]
pub struct NewInstance {
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

/// Instance model, contains data for creating an instance
#[derive(Debug, Serialize, Deserialize)]
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

impl Instance {
    /// Create Instance from NewInstance + ID
    pub fn from_new_instance(new: NewInstance, id: ID) -> Self {
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
    /// Returns also the Name
    #[allow(non_snake_case)]
    pub fn into_InstanceLoadReq(self) -> (InstanceLoadReq, String) {
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
#[derive(Debug, Deserialize, Serialize)]
pub struct Instances {
    pub instances: Vec<InstanceMin>,
}

/// Minimum representation of an Instance
#[derive(Debug, Deserialize, Serialize)]
pub struct InstanceMin {
    pub id: ID,
    pub running: bool,
    pub name: String,
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

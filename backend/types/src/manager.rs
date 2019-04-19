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

use serde::{Serialize,Deserialize};
use crate::{Volume,ID,TimeMS};
#[cfg(feature = "rest")]
use crate::models::Song;

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
    pub instances: Vec<InstanceMin>
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

#[cfg(feature = "rest")]
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
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

/// All of the types here are written from the daemons point of view
/// So a PlaybackUrlReq means client -> daemon request (incoming for daemon)
use serde::{Deserialize, Serialize};

#[cfg(feature = "track")]
use crate::track::Track;

/// Song identifier, char(32)
pub type SongID = String;

/// Cache representation
pub type CacheSong = String;

pub type ID = i32;

/// Volume it 0 to 1.0 (you can go above but that's undefined)
pub type Volume = f64;

/// Database models

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Song {
    pub id: SongID,
    pub name: String,
    /// URL (not youtube-dl Format URL)
    pub source: String,
    pub artist: Option<String>,
    /// Length in seconds
    pub length: Option<u32>,
    pub downloaded: bool,
}

#[cfg(feature = "track")]
impl From<Track> for Song {
    fn from(mut track: Track) -> Self {
        Song {
            id: track.get_id(),
            artist: track.take_artist(),
            length: track.duration_as_u32(),
            name: track.title,
            source: track.webpage_url,
            downloaded: false,
        }
    }
}

impl Song {
    /// Convert Song into minimal song model
    #[allow(dead_code)]
    pub fn into_song_min(self) -> SongMin {
        SongMin {
            id: self.id,
            source: self.source,
            artist: self.artist,
            name: self.name,
            length: self.length,
        }
    }
}

/// Playback request data
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Extract))]
pub struct PlaybackUrlReq {
    pub id: ID,
    pub song: SongMin,
}

/// Volume set data
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Extract))]
pub struct VolumeSetReq {
    pub id: ID,
    pub volume: Volume,
}

/// Pause playback request
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Extract))]
pub struct PlaybackPauseReq {
    pub id: ID,
}

/// Generic Request who require an instance ID
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Extract))]
pub struct GenericRequest {
    pub id: ID,
}

pub type VolumeGetReq = GenericRequest;
pub type StateGetReq = GenericRequest;
pub type InstanceStopReq = GenericRequest;
pub type HeartbeatReq = GenericRequest;

/// Instance started request, internal API
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Extract))]
pub struct InstanceStartedReq {
    pub id: ID,
    pub pid: u32,
}

/// Minimal song representation as required for playback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongMin {
    pub id: SongID,
    pub name: String,
    /// URL (not youtube-dl Format URL)
    pub source: String,
    pub artist: Option<String>,
    /// Length in seconds
    pub length: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Response, Extract))]
pub struct DefaultResponse {
    pub success: bool,
    pub msg: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Response, Extract))]
pub struct VolumeResponse {
    pub volume: Option<Volume>,
    pub msg: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Response, Extract))]
pub struct InstanceListResponse {
    pub instances: Vec<ID>,
}

/// URL Resolver response with ticket number
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Response, Extract))]
pub struct ResolveTicketResponse {
    pub ticket: Option<usize>,
    pub msg: Option<String>,
}

/// Request to resolve an URL for given instance queue
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Extract))]
pub struct ResolveRequest {
    pub instance: ID,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Extract))]
pub struct InstanceLoadReq {
    pub id: ID, //TODO:  zugriff ermöglichen, benötigt für plugin um sich zu identifizieren
    pub data: InstanceType,
    pub volume: Volume,
}

#[derive(Debug, Serialize, Deserialize)] // workaround https://github.com/carllerche/tower-web/issues/189 using Deserialize
pub enum InstanceType {
    TS(TSSettings),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TSSettings {
    pub host: String,
    pub port: Option<u16>,
    pub identity: String,
    pub cid: Option<i32>,
    pub name: String,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "tower", derive(Response, Extract))]
pub struct InstanceOverviewResponse {
    pub instances: Vec<InstanceOverview>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanceOverview {
    pub id: ID,
    pub playing: bool,
    pub volume: Volume,
    pub inst_type: String,
    pub playback_info: String,
}

/// Callbacks

pub mod callback {
    use super::*;

    /// Start of the path for all callbacks
    pub const PATH_START_: &'static str = "callback";
    /// Full path for callback
    pub const PATH_INSTANCE: &'static str = "/callback/instance";
    /// Full path for callback
    pub const PATH_RESOLVE: &'static str = "/callback/resolve";
    /// Full path for callback
    pub const PATH_PLAYBACK: &'static str = "/callback/playback";
    /// Full path for callback
    pub const PATH_SONG: &'static str = "/callback/song";
    /// Full path for callback
    pub const PATH_VOLUME: &'static str = "/callback/volume";

    #[derive(Debug, Serialize, Deserialize)]
    pub struct InstanceStateResponse {
        pub state: InstanceState,
        pub id: ID,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum InstanceState {
        Started,
        Running,
        Stopped,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PlaystateResponse {
        pub state: Playstate,
        pub id: ID,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Playstate {
        Playing,
        Paused,
        Stopped,
        EndOfMedia,
    }

    pub type VolumeChange = VolumeSetReq;

    /// Url resolve response for ticket
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ResolveResponse {
        pub success: bool,
        pub msg: Option<String>,
        pub songs: Vec<Song>,
        pub ticket: usize,
    }
}

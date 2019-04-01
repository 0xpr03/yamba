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

pub use crate::ErrorCodes;
pub use crate::{Volume, ID};

/// Song identifier, char(32)
pub type SongID = String;

/// Cache representation
pub type CacheSong = String;

/// Resolver ticket
pub type Ticket = usize;

pub use crate::TimeMS;

#[cfg(feature = "track")]
impl From<Track> for Song {
    fn from(mut track: Track) -> Self {
        Song {
            id: track.get_id(),
            artist: track.take_artist(),
            length: track.duration_as_u32(),
            name: track.title,
            source: track.webpage_url,
        }
    }
}

/// Playback request data
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Extract))]
pub struct PlaybackUrlReq {
    /// Instance
    pub id: ID,
    pub song: Song,
}

/// Volume set data
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Extract))]
#[cfg_attr(feature = "message", derive(Message))]
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
pub struct Song {
    pub id: SongID,
    pub name: String,
    /// URL (not youtube-dl Format URL)
    pub source: String,
    pub artist: Option<String>,
    /// Length in seconds
    pub length: Option<TimeMS>,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Response, Extract))]
pub struct DefaultResponse {
    pub msg: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Response, Extract))]
pub struct ErrorResponse {
    /// Detailed error code
    pub details: ErrorCodes,
    /// Error details as string
    pub msg: String,
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
    pub ticket: Ticket,
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
    /// Full path for callback
    pub const PATH_POSITION: &'static str = "/callback/position";

    #[derive(Debug, Serialize, Deserialize)]
    pub struct InstanceStateResponse {
        pub state: InstanceState,
        pub id: ID,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum InstanceState {
        Started = 1,
        Running = 2,
        Stopped = 0,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[cfg_attr(feature = "message", derive(Message))]
    pub struct TrackPositionUpdate {
        pub position_ms: TimeMS,
        pub id: ID,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[cfg_attr(feature = "message", derive(Message))]
    pub struct PlaystateResponse {
        pub state: Playstate,
        pub id: ID,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Playstate {
        Stopped = 0,
        Playing = 1,
        Paused = 2,
        EndOfMedia = 3,
    }

    pub type VolumeChange = VolumeSetReq;

    /// Url resolve response for ticket
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ResolveResponse {
        pub success: bool,
        pub msg: Option<String>,
        pub songs: Vec<Song>,
        pub ticket: Ticket,
    }
}

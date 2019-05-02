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

//! Yamba-Daemon API types & structures.
//!
//! All of the types here are written from the daemons pov.  
//! For ResolveRequest this means client -> daemon request (incoming for daemon)

use serde::{Deserialize, Serialize};

#[cfg(feature = "track")]
use crate::track::{GetId, Track, TrackList};

pub use crate::ErrorCodes;
pub use crate::{Volume, ID};

/// Song identifier, char(32)
/// Effectively u128, but not supported by json
pub type SongID = String;

/// Playlist identifier, char(32)
/// Effectively u128, but not supported by json
pub type PlaylistID = String;

/// Cache representation
pub type CacheSong = String;

/// Resolver ticket
pub type Ticket = usize;

/// Instance startup time representation, unix timestamp in seconds
pub type TimeStarted = i64;

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

pub type PlaybackStopReq = PlaybackPauseReq;

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

/// Instance started request, __internal API__  
/// Used by voip plugins like ts3plugin
#[doc(hidden)]
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

/// Playlist representation from resolver
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Playlist {
    pub id: PlaylistID,
    /// Playlist Name
    pub name: String,
    pub songs: Vec<Song>,
    /// Author/Uploader/User
    pub author: Option<String>,
    /// URL
    pub source: String,
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
    /// TODO: evaluate requirement, can be ignored, basically legacy ErrorResponse
    pub msg: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Response, Extract))]
pub struct InstanceListResponse {
    pub instances: Vec<InstanceListEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstanceListEntry {
    /// Instance ID
    pub id: ID,
    /// Unix Timestamp of startup time
    pub started: TimeStarted,
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
    /// Instance ID under which to resolve  
    /// Fails if there exists no instance under this ID
    pub instance: ID,
    /// URL to resolve
    pub url: String,
}

/// Response on successfully started instance
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Response))]
pub struct InstanceLoadResponse {
    /// Internal startup time value, can be used for synchronization
    pub startup_time: TimeStarted,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "tower", derive(Extract))]
pub struct InstanceLoadReq {
    /// ID under which to load this instance
    pub id: ID,
    /// Data for instance
    pub data: InstanceType,
    /// Initial volume
    pub volume: Volume,
}

#[derive(Debug, Serialize, Deserialize)] // workaround https://github.com/carllerche/tower-web/issues/189 using Deserialize
pub enum InstanceType {
    TS(TSSettings),
}

/// Teamspeak instance settings
#[derive(Debug, Serialize, Deserialize)]
pub struct TSSettings {
    /// Host IP/domain
    pub host: String,
    /// Optional if tsdns is available
    pub port: Option<u16>,
    /// Identity to use  
    /// __Currently ignored__
    pub identity: Option<String>,
    /// Channel ID to connect to
    pub cid: Option<i32>,
    /// Name on server
    pub name: String,
    /// Password to use for server, if required
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

#[cfg(feature = "track")]
impl From<TrackList> for Playlist {
    fn from(tracklist: TrackList) -> Self {
        Playlist {
            id: tracklist.get_id(),
            author: tracklist.uploader,
            name: tracklist.title,
            source: tracklist.webpage_url,
            songs: tracklist.entries.into_iter().map(|t| t.into()).collect(),
        }
    }
}

/// Callbacks from daemon (some structs shared for manual polling APIs)
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

    #[derive(Debug, Serialize, Deserialize, Clone)]
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
    /// If a playlist is getting resolved first only one track is resolved.
    /// Subsequent callbacks will be done when further songs got resolved
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ResolveResponse {
        /// Original URL for request
        pub source: String,
        /// Whether the call had success
        pub success: bool,
        /// Message for aribtrary errors
        pub msg: Option<String>,
        /// Song list on success (can be empty for an empty playlist!)
        pub data: ResolveType,
        /// TicketID
        pub ticket: Ticket,
    }
}

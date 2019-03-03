/*
 *  This file is part of yamba.
 *
 *  yamba is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  yamba is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with yamba.  If not, see <https://www.gnu.org/licenses/>.
 */

use chrono::naive::NaiveDateTime;
use serde::Serialize;

use std::fmt::Debug;

use instance::ID;
use ytdl::Track;

/// Song identifier, char(32)
pub type SongID = String;

/// Cache representation
pub type CacheSong = String;

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
#[derive(Debug, Extract)]
pub struct PlaybackUrlReq {
    pub id: ID,
    pub song: SongMin,
}

/// Volume set data
#[derive(Debug, Extract)]
pub struct VolumeSetReq {
    pub id: ID,
    pub volume: f64,
}

/// Pause playback request
#[derive(Debug, Extract)]
pub struct PlaybackPauseReq {
    pub id: ID,
}

/// Generic Request who require an instance ID
#[derive(Debug, Extract)]
pub struct GenericRequest {
    pub id: ID,
}

pub type VolumeGetReq = GenericRequest;
pub type StateGetReq = GenericRequest;
pub type InstanceStopReq = GenericRequest;
pub type HeartbeatReq = GenericRequest;

/// Instance started request, internal API
#[derive(Debug, Extract)]
pub struct InstanceStartedReq {
    pub id: ID,
    pub pid: u32,
}

/// Minimal song representation as required for playback
#[derive(Debug, Clone, Deserialize)]
pub struct SongMin {
    pub id: SongID,
    pub name: String,
    /// URL (not youtube-dl Format URL)
    pub source: String,
    pub artist: Option<String>,
    /// Length in seconds
    pub length: Option<u32>,
}

#[derive(Debug, Response)]
pub struct DefaultResponse {
    pub success: bool,
    pub msg: Option<String>,
}

#[derive(Debug, Response)]
pub struct VolumeResponse {
    pub volume: Option<f64>,
    pub msg: Option<String>,
}

#[derive(Debug, Response)]
pub struct InstanceListResponse {
    pub instances: Vec<ID>,
}

/// Url resolve response for ticket
#[derive(Debug, Serialize)]
pub struct ResolveResponse {
    pub success: bool,
    pub msg: Option<String>,
    pub songs: Vec<Song>,
    pub ticket: usize,
}

/// URL Resolver response with ticket number
#[derive(Debug, Response)]
pub struct ResolveTicketResponse {
    pub ticket: Option<usize>,
    pub msg: Option<String>,
}

/// Request to resolve an URL for given instance queue
#[derive(Debug, Extract)]
pub struct ResolveRequest {
    pub instance: ID,
    pub url: String,
    pub callback_address: String,
}

#[derive(Debug, Extract)]
pub struct InstanceLoadReq {
    pub id: ID, //TODO:  zugriff ermöglichen, benötigt für plugin um sich zu identifizieren
    pub data: InstanceType,
    pub volume: f64,
}

#[derive(Debug, Deserialize)] // workaround https://github.com/carllerche/tower-web/issues/189 using Deserialize
pub enum InstanceType {
    TS(TSSettings),
}

#[derive(Debug, Deserialize)]
pub struct TSSettings {
    pub host: String,
    pub port: Option<u16>,
    pub identity: String,
    pub cid: Option<i32>,
    pub name: String,
    pub password: Option<String>,
}

#[derive(Debug, Response)]
pub struct InstanceOverviewResponse {
    pub instances: Vec<InstanceOverview>,
}

#[derive(Debug, Serialize)]
pub struct InstanceOverview {
    pub id: ID,
    pub playing: bool,
    pub volume: f64,
    pub inst_type: String,
    pub playback_info: String,
}

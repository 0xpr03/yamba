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

use instance::ID;

/// Song identifier, char(32)
pub type SongID = String;
pub type QueueID = i32;

/// Database models

#[derive(Debug, Clone, Deserialize)]
pub struct Song {
    pub id: String,
    pub name: String,
    pub source: String,
    pub artist: Option<String>,
    pub length: Option<i32>,
    pub downloaded: bool,
    pub last_used: NaiveDateTime,
}

impl Song {
    /// Convert Song into minimal song model
    pub fn into_song_min(self) -> SongMin {
        SongMin {
            id: self.id,
            source: self.source,
        }
    }
}

/// Minimal song representation as required for playback
#[derive(Debug, Clone, Deserialize)]
pub struct SongMin {
    pub id: String,
    pub source: String,
/// Instance settings storage
#[derive(Debug, Clone, Deserialize)]
pub struct InstanceStorage {
    pub id: i32,
    pub volume: f64,
    pub index: Option<QueueID>,
    pub position: Option<f64>,
    pub random: bool,
    pub repeat: bool,
    pub queue_lock: bool,
    pub volume_lock: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub keep: bool,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub enum DBInstanceType {
    TS(TSSettings),
}

#[derive(Debug, Clone, Deserialize)]
pub struct TSSettings {
    pub id: i32,
    pub host: String,
    pub port: Option<u16>,
    pub identity: String,
    pub cid: Option<i32>,
    pub name: String,
    pub password: Option<String>,
    pub autostart: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Queue {
    pub index: i32,
    pub instance_id: i32,
    pub title_id: String,
}

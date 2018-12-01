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

/// Database models

#[derive(Debug, Clone, Deserialize)]
pub struct Song {
    pub id: String,
    pub name: String,
    pub source: String,
    pub length: Option<i32>,
    pub downloaded: bool,
    pub artist: Option<String>,
    pub last_used: NaiveDateTime,
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

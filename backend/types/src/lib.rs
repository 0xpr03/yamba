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

#[cfg(feature = "tower")]
#[macro_use]
extern crate tower_web;
use serde::{Deserialize, Serialize};

#[cfg(feature = "rest")]
pub mod models;
#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(feature = "track")]
pub mod track;
#[cfg(feature = "message")]
#[macro_use]
extern crate actix;

pub type ID = i32;

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorCodes {
    NONE = 0,
    INVALID_INSTANCE = 401,
    INVALID_VOLUME = 402,
    INSTANCE_RUNNING = 403,
    RESOLVE_QUEUE_OVERLOAD = 404,
}

/// Volume it 0 to 1.0 (you can go above but that's undefined)
pub type Volume = f64;

/// Time unit (playback)
pub type TimeMS = u32;

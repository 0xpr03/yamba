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

//! ### yamba types
//! Contains types used by the daemon api and hared types of yamba-daemon and voip plugin code.
//!
//! #### Generating docs
//! Run `cargo doc --no-deps --open`.  
//! To also get ts3plugin type docs use `cargo doc --no-deps --open --features rpc`
//!
//! #### Modules
//! - `models` Main docs for API related things, contains callback section.
//! - `rpc` RPC things for voip plugins like ts3plugin.
//! - `track` Internal API types from yamba-daemon

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

/// Instance ID
pub type ID = i32;

/// Error codes  
/// Values are not the http codes and are just for comparison
#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorCodes {
    NONE = 0,
    /// Invalid instance ID
    INVALID_INSTANCE = 401,
    /// Invalid volume (out of range)
    INVALID_VOLUME = 402,
    /// Instance already running, can't start
    INSTANCE_RUNNING = 403,
    /// Queue overloaded for instance, can't enqueue resolve job
    RESOLVE_QUEUE_OVERLOAD = 404,
}

/// Volume of 0 to 1.0 (you can go above but that's undefined)
pub type Volume = f64;

/// Time unit (playback)
pub type TimeMS = u32;

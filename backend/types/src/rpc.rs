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

//! VoIP plugin RPC structures  
//! Param stands for parameter, meaning calls from voip -> management

use serde::{Serialize,Deserialize};
pub use crate::{Volume, ID};
pub use crate::ErrorCodes;


/// Get id
pub trait GetId {
	/// Get id from struct
	fn get_id(&self) -> ID;
}

/// Error code for permission error
pub const PERMISSION_ERROR_CODE:i64  = 403;

macro_rules! add_trait {
    ( ( $( $Trait: ident ),+ ) for $Ty: ident ) => {
        $(
            impl $Trait for $Ty {
                #[inline]
                fn get_id(&self) -> ID {
                    self.id
                }
            }
        )+
    }
}

add_trait! {(GetId) for ParamVolume}
add_trait! {(GetId) for ParamQueue}
add_trait! {(GetId) for ParamDefault}
add_trait! {(GetId) for ParamQueueTracks}

/// Default parameters provided
#[derive(Debug, Deserialize, Serialize)]
pub struct ParamDefault {
	/// Instance ID
	pub id: ID,
	/// Name of invoker
	pub invoker_name: String,
	/// Groups of invoker
	pub invoker_groups: String,
}

/// Volume set RPC
#[derive(Debug, Deserialize, Serialize)]
pub struct ParamVolume {
	/// Instance ID
	pub id: ID,
	/// Name of invoker
	pub invoker_name: String,
	/// Groups of invoker
	pub invoker_groups: String,
	/// Volume
	pub volume: Volume,
}

/// Enqueue Parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct ParamQueue {
	/// Instance ID
	pub id: ID,
	/// Name of invoker
	pub invoker_name: String,
	/// Groups of invoker
	pub invoker_groups: String,
	/// URL to enqueue
	pub url: String,
}

/// Display upcoming tracks in queue
#[derive(Debug, Deserialize, Serialize)]
pub struct ParamQueueTracks {
	/// Instance ID
	pub id: ID,
	/// Name of invoker
	pub invoker_name: String,
	/// Groups of invoker
	pub invoker_groups: String,
	/// Amount of tracks to display in queue
	pub n: usize,
}

/// Default response to RPC call
#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultResponse {
	pub message: String,
}

/// Error response data
#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
	/// Whether call was permitted
	pub allowed: bool,
	/// Message to respond
	pub message: String,
	/// Error details
	pub details: ErrorCodes,
}

/// Playback info response
#[derive(Debug, Deserialize, Serialize)]
pub struct TitleResponse {
	pub title: String,
}

/// Volume request response
#[derive(Debug, Deserialize, Serialize)]
pub struct VolumeResponse {
	pub volume: Volume,
}

/// Unused?
#[derive(Debug, Deserialize, Serialize)]
pub struct PlaylistResponse {
	pub allowed: bool,
	pub message: String,
	pub name: String,
}

/// Queue upcoming songs response
#[derive(Debug, Deserialize, Serialize)]
pub struct TitleListResponse {
	pub tracklist: Vec<String>,
}
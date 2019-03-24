
use serde::{Serialize,Deserialize};
pub use crate::{Volume, ID};
pub use crate::ErrorCodes;

//allowed, message, success

/// Get id
pub trait GetId {
	/// Get id from struct
	fn get_id(&self) -> ID;
}

/// Error code for permission error
pub const PermissionErrorCode:i64  = 403;

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

#[derive(Debug, Deserialize, Serialize)]
pub struct ParamDefault {
	pub id: ID,
	pub invoker_name: String,
	pub invoker_groups: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ParamVolume {
	pub id: ID,
	pub invoker_name: String,
	pub invoker_groups: String,
	pub volume: Volume,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ParamQueue {
	pub id: ID,
	pub invoker_name: String,
	pub invoker_groups: String,
	pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultResponse {
	pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
	pub allowed: bool,
	pub message: String,
	pub details: ErrorCodes,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TitleResponse {
	pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VolumeResponse {
	pub volume: Volume,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlaylistResponse {
	pub allowed: bool,
	pub message: String,
	pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TitleListResponse {
	pub allowed: bool,
	pub message: String,
	pub tracklist: Vec<String>,
}
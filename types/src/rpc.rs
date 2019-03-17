
use serde::{Serialize,Deserialize};
pub use crate::{Volume, ID};


//allowed, message, success

/// Get id
pub trait GetId {
	/// Get id from struct
	fn get_id(&self) -> ID;
}


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

add_trait! {
                (GetId) for ParamVolume
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ParamVolume {
	pub id: ID,
	pub invoker_name: String,
	pub invoker_groups: String,
	pub volume: Volume,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultResponse {
	pub allowed: bool,
	pub message: String,
	pub success: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TitleResponse {
	pub allowed: bool,
	pub message: String,
	pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VolumeResponse {
	pub allowed: bool,
	pub message: String,
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
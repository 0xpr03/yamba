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

pub use crate::ErrorCodes;
pub use crate::{Volume, ID};
use serde::{Deserialize, Serialize};

/// HTTP Error code for missing permission of invoking user
pub const PERMISSION_ERROR_CODE: i64 = 403;

/// Default parameters provided
#[derive(Debug, Serialize)]
pub struct DefaultRequest<'a> {
	/// Instance ID
	pub id: ID,
	/// Login Token (if any)
	pub token: &'a Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParamRequest<'a, T: Serialize> {
	/// Instance ID
	pub id: ID,
	/// Login Token (if any)
	pub token: &'a Option<String>,
	/// Parameter
	pub data: T,
}

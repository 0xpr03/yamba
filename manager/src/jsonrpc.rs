/*
 *  YAMBA middleware
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

use failure::Fallible;
use hashbrown::HashMap;
use jsonrpc_core::types::error::Error;
use jsonrpc_core::*;
use jsonrpc_http_server::*;
use owning_ref::OwningRef;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json;
use yamba_types::rpc::*;

use std::net::SocketAddr;
use std::sync::RwLockReadGuard;

use crate::instance::{Instance, Instances};

macro_rules! read_instance {
    ( ( $( $Trait: ident ),+ ) for $Ty: ident ) => {
        $(
            #[$stability]
            impl fmt::$Trait for $Ty {
                #[inline]
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    self.get().fmt(f)
                }
            }
        )+
    }
}

fn parse_input<'a, T, F>(data: Params, foo: F) -> Result<Value>
where
	F: Fn(T) -> Result<Value> + 'static,
	T: DeserializeOwned + 'a + 'static,
{
	match data.parse::<T>() {
		Ok(v) => foo(v),
		Err(e) => Err(e),
	}
}

// fn parse_input_read_inst<'a, T, F>(data: Params, foo: F, instances: Instances) -> Result<Value>
// where
// 	F: Fn(T) -> Result<Value> + 'static,
// 	T: DeserializeOwned + GetId + 'a + 'static,
// {
// 	let parsed = data.parse::<T>()?;
// 	let instance = match get_instance_by_id(&instances, parsed.get_id()) {
// 		None => Ok(response_invalid_instance(&parsed.get_id())),
// 		Some(v) => v,
// 	};
// 	Ok(Value::Null)
// }

#[inline]
fn response_ignore() -> DefaultResponse {
	DefaultResponse {
		success: true,
		allowed: true,
		message: String::from("default"),
	}
}

#[inline]
fn response_invalid_instance(id: &ID) -> DefaultResponse {
	DefaultResponse {
		success: false,
		allowed: true,
		message: format!("invalid instance {}", id),
	}
}

/// Get instance by ID
/// Returns instance & guard
fn get_instance_by_id<'a>(
	instances: &'a Instances,
	instance_id: &ID,
) -> Option<OwningRef<RwLockReadGuard<'a, HashMap<i32, Instance>>, Instance>> {
	let instances_r = instances.read().expect("Can't read instance!");
	OwningRef::new(instances_r)
		.try_map(|i| match i.get(instance_id) {
			Some(v) => Ok(v),
			None => Err(()),
		})
		.ok()
}

/// Create jsonrpc server for handling chat cmds
pub fn create_server(
	bind_addr: &SocketAddr,
	allowed_host: &str,
	inst: Instances,
) -> Fallible<Server> {
	let mut io = IoHandler::new();

	io.add_method("volume_set", |data: Params| {
		match data.parse::<ParamVolume>() {
			Ok(v) => {
				debug!("Received rpc: {:?}", v);
				let value = serde_json::to_value(response_ignore()).unwrap();
				Ok(value)
			}
			Err(e) => Err(e),
		}
	});

	let server = ServerBuilder::new(io)
		.allowed_hosts(DomainsValidation::AllowOnly(vec![allowed_host.into()]))
		.rest_api(RestApi::Secure)
		.start_http(bind_addr)?;

	Ok(server)
}

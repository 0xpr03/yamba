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
use futures::{
	future::{result, Either, IntoFuture},
	Future,
};
use hashbrown::HashMap;
use jsonrpc_core::types::error::{self, Error, ErrorCode};
use jsonrpc_core::*;
use jsonrpc_http_server::*;
use owning_ref::OwningRef;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json;
use yamba_types::rpc::*;

use std::net::SocketAddr;
use std::sync::RwLockReadGuard;

use crate::instance::{Instance, Instances};

// macro_rules! response_ok {
// 	() => {
// 		result(Ok(serde_json::to_value(response_ignore()).unwrap()))
// 	};
// }

/// Parse input and call fn on success
fn parse_input<T, F, D>(data: Params, foo: F) -> impl Future<Item = Value, Error = Error>
where
	F: Fn(T) -> D,
	T: DeserializeOwned + Send,
	D: Future<Item = Value, Error = Error>,
{
	match data.parse::<T>() {
		Ok(v) => Either::A(foo(v)),
		Err(e) => Either::B(result(Err(e))),
	}
	// let v = data.parse::<T>().unwrap();
	// foo(v)
}

/// Parse input, get correct instance, call fn on success
fn parse_input_instance<T, F, D>(
	instances: Instances,
	data: Params,
	foo: F,
) -> impl Future<Item = Value, Error = Error>
where
	F: Fn(T, InstanceRef) -> D,
	T: DeserializeOwned + 'static + GetId + Send,
	D: Future<Item = Value, Error = Error> + Send,
{
	parse_input(data, move |v: T| {
		match get_instance_by_id(&instances, &v.get_id()) {
			Some(i) => Either::A(foo(v, i)),
			None => Either::B(result(Ok(serde_json::to_value(response_invalid_instance(
				&v.get_id(),
			))
			.unwrap()))),
		}
	})
}

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

type InstanceRef<'a> = OwningRef<RwLockReadGuard<'a, HashMap<i32, Instance>>, Instance>;

/// Get instance by ID
/// Returns instance & guard
fn get_instance_by_id<'a>(instances: &'a Instances, instance_id: &ID) -> Option<InstanceRef<'a>> {
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
	instances: Instances,
) -> Fallible<Server> {
	let mut io = IoHandler::new();

	let inst_c = instances.clone();
	io.add_method("volume_set", move |data: Params| {
		parse_input_instance(inst_c.clone(), data, |v: ParamVolume, inst| {
			inst.set_volume(v.volume)
				.unwrap()
				.map_err(|e| {
					warn!("Unable to set volume: {}", e);
					Error {
						data: None,
						message: e.to_string(),
						code: error::ErrorCode::InternalError,
					}
				})
				.map(|_| serde_json::to_value(response_ignore()).unwrap())
			// Ok(v) => v,
			// Err(e) => {
			// 	warn!("Unable to set volume! {}", e);
			// 	result(Err(Error {
			// 		code: ErrorCode::ServerError,
			// 		message: format!(""),
			// 		data: None,
			// 	}))
			// }
			// }
		})
	});
	// io.add_method("queue", move |data: Params| {
	// 	parse_input_instance(
	// 		instances.clone(),
	// 		data,
	// 		|v: ParamQueue, inst| response_ok!(),
	// 	)
	// });

	let server = ServerBuilder::new(io)
		.allowed_hosts(DomainsValidation::AllowOnly(vec![allowed_host.into()]))
		.rest_api(RestApi::Secure)
		.start_http(bind_addr)?;

	Ok(server)
}

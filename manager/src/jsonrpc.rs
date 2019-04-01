/*
 *  YAMBA manager
 *  Copyright (C) 2019 Aron Heinecke
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

use failure::Fallible;
use futures::{
	future::{result, Either},
	Future,
};
use hashbrown::HashMap;
use jsonrpc_core::types::error::{self, Error};
use jsonrpc_core::*;
use jsonrpc_http_server::*;
use owning_ref::OwningRef;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use yamba_types::rpc::*;

use std::net::SocketAddr;
use std::sync::RwLockReadGuard;

use crate::instance::{Instance, Instances};

// macro_rules! rpc {
//     $( $x:expr,$type:ident,$io:expr,$ ) => {
//         {
//             $io.add_method($x, move |data: Params| {
// 			parse_input_instance(inst_c.clone(), data, |v: $type, inst| {
// 				match inst.set_volume(v.volume) {
// 					Err(e) => Either::A(send_internal_server_error(e)),
// 					Ok(val) => Either::B(
// 						val.map_err(|e| {
// 							warn!("Unable to set volume: {}", e);
// 							Error {
// 								data: None,
// 								message: e.to_string(),
// 								code: error::ErrorCode::InternalError,
// 							}
// 						})
// 						.map(|_| serde_json::to_value(response_ignore()).unwrap()),
// 					),
// 				}
// 			})
// 		});
//         }
//     };
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
	parse_input(data, move |v: T| match instances.read(&v.get_id()) {
		Some(i) => Either::A(foo(v, i)),
		None => Either::B(result(Ok(serde_json::to_value(response_invalid_instance(
			&v.get_id(),
		))
		.unwrap()))),
	})
}

/// Helper method to add rpc
// fn add_rpc<T, F, D>(io: &mut IoHandler, instances: Instances, foo: F, method: &str)
// where
// 	F: Fn(T, InstanceRef) -> D + Send + Sync + 'static,
// 	T: DeserializeOwned + 'static + GetId + Send,
// 	D: Future<Item = Value, Error = Error> + Send + 'static,
// {
// 	let inst_c = instances.clone();
// 	io.add_method(method, move |data: Params| {
// 		let inst_b = inst_c.clone();
// 		parse_input_instance(inst_b, data, |v: T, inst| foo(v, inst))
// 	});
// }

/// Helper to send failure as 500 status
fn send_internal_server_error(err: failure::Error) -> impl Future<Item = Value, Error = Error> {
	result(Ok(serde_json::to_value(Error {
		data: None,
		message: err.to_string(),
		code: error::ErrorCode::InternalError,
	})
	.unwrap()))
}

/// Helper to send ok as 200 status
fn send_ok() -> impl Future<Item = Value, Error = Error> {
	result(Ok(serde_json::to_value(response_ignore()).unwrap()))
}

/// Helper to send ok as 200 status
fn send_ok_custom<T>(val: T) -> impl Future<Item = Value, Error = Error>
where
	T: Serialize,
{
	result(Ok(serde_json::to_value(val).unwrap()))
}

#[inline]
fn response_ignore() -> DefaultResponse {
	DefaultResponse {
		message: String::from(""),
	}
}

#[inline]
fn response_invalid_instance(id: &ID) -> DefaultResponse {
	DefaultResponse {
		message: format!("invalid instance {}", id),
	}
}

type InstanceRef<'a> = OwningRef<RwLockReadGuard<'a, HashMap<i32, Instance>>, Instance>;

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
			match inst.set_volume(v.volume) {
				Err(e) => Either::A(send_internal_server_error(e)),
				Ok(val) => Either::B(
					val.map_err(|e| {
						warn!("Unable to set volume: {}", e);
						Error {
							data: None,
							message: e.to_string(),
							code: error::ErrorCode::InternalError,
						}
					})
					.map(|_| serde_json::to_value(response_ignore()).unwrap()),
				),
			}
		})
	});
	let inst_c = instances.clone();
	io.add_method("queue", move |data: Params| {
		parse_input_instance(inst_c.clone(), data, |v: ParamQueue, inst| {
			match inst.queue(v.url) {
				Err(e) => Either::A(send_internal_server_error(e)),
				Ok(val) => Either::B(
					val.map_err(|e| {
						warn!("Unable to queue url: {}", e);
						Error {
							data: None,
							message: e.to_string(),
							code: error::ErrorCode::InternalError,
						}
					})
					.map(|_v| serde_json::to_value(response_ignore()).unwrap()),
				),
			}
		})
	});
	let inst_c = instances.clone();
	io.add_method("track_next", move |data: Params| {
		parse_input_instance(inst_c.clone(), data, |_v: ParamDefault, inst| {
			match inst.play_next() {
				Err(e) => Either::A(send_internal_server_error(e)),
				Ok(_) => Either::B(send_ok()),
			}
		})
	});
	let inst_c = instances.clone();
	io.add_method("volume_get", move |data: Params| {
		parse_input_instance(inst_c.clone(), data, |_v: ParamDefault, inst| {
			match inst.get_volume() {
				Err(e) => Either::A(send_internal_server_error(e)),
				Ok(v) => Either::B(send_ok_custom(VolumeResponse { volume: v })),
			}
		})
	});
	let inst_c = instances.clone();
	io.add_method("track_get", move |data: Params| {
		parse_input_instance(inst_c.clone(), data, |_v: ParamDefault, inst| {
			match inst.get_formated_title() {
				Err(e) => Either::A(send_internal_server_error(e)),
				Ok(v) => {
					debug!("{}", v);
					Either::B(send_ok_custom(TitleResponse { title: v }))
				}
			}
		})
	});
	let inst_c = instances.clone();
	io.add_method("queue_tracks", move |data: Params| {
		parse_input_instance(inst_c.clone(), data, |v: ParamQueueTracks, inst| {
			send_ok_custom(TitleListResponse {
				tracklist: inst.get_upcoming_tracks(v.n),
			})
		})
	});

	let server = ServerBuilder::new(io)
		//.allowed_hosts(DomainsValidation::AllowOnly(vec![allowed_host.into()]))
		.rest_api(RestApi::Secure)
		.start_http(bind_addr)?;

	Ok(server)
}

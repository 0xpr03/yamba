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

use failure::Fallible;
use hyper;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server};
use jsonrpc_lite::{Error, Id, JsonRpc, Params};
use owning_ref::OwningRef;
use serde_json::{self, to_value, Value};
use tokio::runtime;

use hashbrown::HashMap;
use std::sync::{atomic::Ordering, RwLock, RwLockReadGuard};

use daemon::{self, BoxFut, Instances};
use instance::{Instance, InstanceType};
use SETTINGS;

/// RPC server for client callbacks

#[derive(Fail, Debug)]
pub enum RPCErr {
    #[fail(display = "RPC bind error {}", _0)]
    BindError(#[cause] hyper::error::Error),
}

fn rpc(req: Request<Body>, instances: Instances) -> BoxFut {
    Box::new(req.into_body().concat2().map(move |b| {
        let response_rpc = match serde_json::from_slice::<JsonRpc>(&b) {
            Ok(rpc) => {
                let mut req_id = rpc.get_id().unwrap_or(Id::None(()));
                match parse_rpc_call(&req_id, &rpc) {
                    Ok((instance_id, method, params)) => match method {
                        "heartbeat" => JsonRpc::success(req_id, &json!("true")),
                        "connected" => handle_connected(req_id, params, instances, instance_id),
                        "volume_get" => handle_volume_get(req_id, instances, instance_id),
                        "volume_set" => handle_volume_set(req_id, params, instances, instance_id),
                        "playlist_queue" => handle_enqueue(req_id, params, instances, instance_id),
                        "track_stop" => handle_stop(req_id, params, instances, instance_id),
                        "track_resume" => handle_resume(req_id, params, instances, instance_id),
                        "track_next" => handle_next(req_id, params, instances, instance_id),
                        _ => {
                            trace!("Unknown rpc request: {:?}", rpc);
                            JsonRpc::error(req_id, Error::method_not_found())
                        }
                    },
                    Err(e) => {
                        warn!("Can't parse rpc: {:?}", e);
                        e
                    }
                }
            }
            Err(e) => {
                warn!("Invalid rpc request {}", e);
                JsonRpc::error(Id::None(()), Error::parse_error())
            }
        };
        // https://github.com/hyperium/hyper/blob/master/examples/params.rs
        let body = to_value(response_rpc).unwrap().to_string();
        //trace!("Sending response for rpc");
        Response::new(body.into())
    }))
}

/// handle track_next
fn handle_next(req_id: Id, params: Vec<Value>, instances: Instances, instance_id: i32) -> JsonRpc {
    let instance = match get_instance_by_id(&req_id, &*instances, &instance_id) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if let Err(e) = instance.play_next_track() {
        warn!("Can't play next track. {}\n{}", e, e.backtrace());
        return JsonRpc::error(req_id, Error::internal_error());
    }
    JsonRpc::success(req_id, &json!((true, "test", true)))
}

/// handle track_resume
fn handle_resume(
    req_id: Id,
    params: Vec<Value>,
    instances: Instances,
    instance_id: i32,
) -> JsonRpc {
    let instance = match get_instance_by_id(&req_id, &*instances, &instance_id) {
        Ok(v) => v,
        Err(e) => return e,
    };
    instance.stop_flag.store(false, Ordering::Relaxed);
    if instance
        .current_song
        .read()
        .expect("can't lock current song")
        .is_some()
    {
        instance.player.play();
    } else {
        if let Err(e) = instance.play_next_track() {
            warn!("Can't resume. {}\n{}", e, e.backtrace());
            return JsonRpc::error(req_id, Error::internal_error());
        }
    }
    JsonRpc::success(req_id, &json!((true, "test", true)))
}

/// handle track_stop
fn handle_stop(req_id: Id, params: Vec<Value>, instances: Instances, instance_id: i32) -> JsonRpc {
    let instance = match get_instance_by_id(&req_id, &*instances, &instance_id) {
        Ok(v) => v,
        Err(e) => return e,
    };
    instance.stop_flag.store(true, Ordering::Relaxed);
    let mut lock = instance
        .current_song
        .write()
        .expect("Can't lock current song!");
    *lock = None;
    instance.player.stop();
    JsonRpc::success(req_id, &json!((true, "test", true)))
}

/// Handle enqueue
fn handle_enqueue(
    req_id: Id,
    params: Vec<Value>,
    instances: Instances,
    instance_id: i32,
) -> JsonRpc {
    let instance = match get_instance_by_id(&req_id, &*instances, &instance_id) {
        Ok(v) => v,
        Err(e) => return e,
    };
    match parse_string(&req_id, 3, &params) {
        Ok(url) => {
            instance.enqueue_by_url(url.clone());
            JsonRpc::success(req_id, &json!((true, "test", true)))
        }
        Err(e) => e,
    }
}

/// Handle volume_set
fn handle_volume_set(
    req_id: Id,
    params: Vec<Value>,
    instances: Instances,
    instance_id: i32,
) -> JsonRpc {
    let instance = match get_instance_by_id(&req_id, &*instances, &instance_id) {
        Ok(v) => v,
        Err(e) => return e,
    };
    match parse_f64(&req_id, 3, &params) {
        Ok(v) => {
            if v >= 0.0 && v <= 1.0 {
                instance.player.set_volume(v);
                JsonRpc::success(req_id, &json!((true, "test", true)))
            } else {
                warn!("Invalid volume: {}!", v);
                JsonRpc::error(req_id, Error::invalid_params())
            }
        }
        Err(e) => e,
    }
}

/// Handle volume_get
fn handle_volume_get(req_id: Id, instances: Instances, instance_id: i32) -> JsonRpc {
    let instance = match get_instance_by_id(&req_id, &*instances, &instance_id) {
        Ok(v) => v,
        Err(e) => return e,
    };
    JsonRpc::success(req_id, &json!((true, "test", instance.player.get_volume())))
}

/// Handle connect rpc
fn handle_connected(
    req_id: Id,
    params: Vec<Value>,
    instances: Instances,
    instance_id: i32,
) -> JsonRpc {
    trace!("ts connected");
    let process_id = match params.get(1) {
        Some(Value::Number(ref v)) => v.as_u64(),
        v => {
            warn!("Missing process ID, got {:?}", v);
            return JsonRpc::error(req_id, Error::invalid_params());
        }
    };
    let process_id: u32 = match process_id {
        Some(v) => v as u32,
        None => {
            warn!("Invalid process ID type!");
            return JsonRpc::error(req_id, Error::invalid_params());
        }
    };
    let instance_r = instances.read().expect("Can't read instance!");
    if let Some(instance) = instance_r.get(&instance_id) {
        // if let, but irrefutable pattern as of now..
        let InstanceType::Teamspeak(ref ts) = *instance.voip;
        if let Err(e) = ts.on_connected(process_id) {
            warn!("Error on post-connection action: {}\n{}", e, e.backtrace());
            JsonRpc::success(req_id, &json!(false))
        } else {
            JsonRpc::success(req_id, &json!(true))
        }
    } else {
        error!(
            "Received connected event for invalid instance ID {:?}",
            req_id
        );
        JsonRpc::error(req_id, Error::invalid_params())
    }
}

/// Parse String from params
fn parse_string<'a>(
    req_id: &Id,
    position: usize,
    params: &'a Vec<Value>,
) -> Result<&'a String, JsonRpc> {
    match params.get(position) {
        Some(Value::String(v)) => Ok(v),
        e => {
            warn!("Couldn't parse String! {:?}", e);
            return Err(JsonRpc::error(req_id.clone(), Error::invalid_request()));
        }
    }
}

/// Parse i32 from params
fn parse_i32(req_id: &Id, position: usize, params: &Vec<Value>) -> Result<i32, JsonRpc> {
    match params.get(position) {
        Some(Value::Number(id)) => {
            if let Some(id) = id.as_i64() {
                Ok(id as i32)
            } else if let Some(id) = id.as_u64() {
                Ok(id as i32)
            } else {
                return Err(JsonRpc::error(req_id.clone(), Error::invalid_params()));
            }
        }
        v => {
            warn!("Couldn't parse i32! {:?}", v);
            return Err(JsonRpc::error(req_id.clone(), Error::invalid_request()));
        }
    }
}

/// Parse f64 from params
fn parse_f64(req_id: &Id, position: usize, params: &Vec<Value>) -> Result<f64, JsonRpc> {
    match params.get(position) {
        Some(Value::Number(id)) => {
            if let Some(id) = id.as_f64() {
                Ok(id)
            } else if let Some(id) = id.as_i64() {
                Ok(id as f64)
            } else if let Some(id) = id.as_u64() {
                Ok(id as f64)
            } else {
                return Err(JsonRpc::error(req_id.clone(), Error::invalid_params()));
            }
        }
        v => {
            warn!("Couldn't parse f64! {:?}", v);
            return Err(JsonRpc::error(req_id.clone(), Error::invalid_request()));
        }
    }
}

/// Parse input and retrieve relevant data
fn parse_rpc_call<'a>(
    req_id: &Id,
    rpc: &'a JsonRpc,
) -> Result<(i32, &'a str, Vec<Value>), JsonRpc> {
    let params = match rpc.get_params() {
        Some(Params::Array(params)) => params,
        v => {
            warn!("Invalid rpc request params: {:?}", v);
            return Err(JsonRpc::error(Id::None(()), Error::parse_error()));
        }
    };
    let instance_id = match parse_i32(req_id, 0, &params) {
        Ok(v) => v,
        Err(e) => {
            warn!("Missing instance ID for rpc call! {:?}", e);
            return Err(JsonRpc::error(req_id.clone(), Error::invalid_request()));
        }
    };
    let method = match rpc.get_method() {
        Some(method) => method,
        _ => return Err(JsonRpc::error(req_id.clone(), Error::invalid_request())),
    };

    Ok((instance_id as i32, method, params))
}

/// Get instance by ID
/// Returns instance & guard
fn get_instance_by_id<'a>(
    req_id: &Id,
    instances: &'a RwLock<HashMap<i32, Instance>>,
    instance_id: &i32,
) -> Result<OwningRef<RwLockReadGuard<'a, HashMap<i32, Instance>>, Instance>, JsonRpc> {
    let instances_r = instances.read().expect("Can't read instance!");
    OwningRef::new(instances_r).try_map(|i| match i.get(instance_id) {
        Some(v) => Ok(v),
        None => Err(JsonRpc::error(req_id.clone(), Error::invalid_params())),
    })
}

/// Check config for RPC, throws an error if not ok for starting
pub fn check_config() -> Fallible<()> {
    let _ = daemon::parse_socket_address(&SETTINGS.main.rpc_bind_ip, SETTINGS.main.rpc_bind_port)?;
    Ok(())
}

/// Create rpc server, bind it & attach to runtime
pub fn create_rpc_server(runtime: &mut runtime::Runtime, instances: Instances) -> Fallible<()> {
    let addr =
        daemon::parse_socket_address(&SETTINGS.main.rpc_bind_ip, SETTINGS.main.rpc_bind_port)?;
    let server = Server::try_bind(&addr)
        .map_err(|e| RPCErr::BindError(e))?
        .serve(move || {
            let instances = instances.clone();
            service_fn(move |req: Request<Body>| rpc(req, instances.clone()))
        })
        .map_err(|e| error!("server error: {}", e));

    info!("RPC Listening on http://{}", addr);
    runtime.spawn(server);
    Ok(())
}

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
use hashbrown::HashMap;
use http_r::{response::Response, status::StatusCode};
use owning_ref::OwningRef;
use serde::Serialize;
use tokio::{executor, runtime};
use yamba_types::models::{DefaultResponse, ErrorCodes, ErrorResponse};

use std::net::SocketAddr;
use std::sync::RwLockReadGuard;

use daemon::{
    instance::{Instance, ID},
    InstanceBase, Instances,
};

pub mod callback;
mod internal;
mod public;

#[derive(Fail, Debug)]
pub enum APIErr {
    #[fail(display = "API bind error {}", _0)]
    BindError(#[cause] std::io::Error),
    #[fail(display = "Unable to spawn onto executor {}", _0)]
    ExcecutionFailed(#[cause] executor::SpawnError),
}

/// Start api server
pub fn start_server(
    runtime: &mut runtime::Runtime,
    instances: Instances,
    base: InstanceBase,
) -> Fallible<()> {
    internal::start_server(runtime, instances.clone(), base.heartbeat.clone())?;
    public::start_server(runtime, instances, base)?;
    Ok(())
}

/// Check api runtime config
pub fn check_runtime() -> Fallible<()> {
    public::parse_addr()?;
    internal::parse_addr()?;
    Ok(())
}

/// Unify bind addr parsing
fn parse_address(host: &str, port: &u16) -> Fallible<SocketAddr> {
    Ok(format!("{}:{}", host, port).parse()?)
}

/// Response type
type Rsp = Fallible<Response<String>>;

/// Helper returning empty DefaultResponse
fn ok() -> Rsp {
    Ok(Response::builder()
        .body(serde_json::to_string(&DefaultResponse { msg: None }).unwrap())
        .unwrap())
}

fn accepted() -> Rsp {
    let mut builder = Response::builder();
    builder.status(StatusCode::ACCEPTED);
    Ok(builder.body(String::new()).unwrap())
}

/// Helper returning 200 + specified json struct
fn ok_response<T>(val: T) -> Rsp
where
    T: Serialize,
{
    Ok(Response::builder()
        .body(serde_json::to_string(&val).unwrap())
        .unwrap())
}

/// Helper to return invalid instance error
fn invalid_instance() -> Rsp {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(
            serde_json::to_string(&ErrorResponse {
                details: ErrorCodes::INVALID_INSTANCE,
                msg: String::from("Invalid Instance"),
            })
            .unwrap(),
        )
        .unwrap())
}

/// Helper for custom response
fn custom_response<T>(code: StatusCode, data: T) -> Rsp
where
    T: Serialize,
{
    Ok(Response::builder()
        .status(code)
        .body(serde_json::to_string(&data).unwrap())
        .unwrap())
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

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
use owning_ref::OwningRef;
use tokio::runtime;

use std::net::SocketAddr;
use std::sync::RwLockReadGuard;

use daemon::{InstanceBase, Instances};
use instance::{Instance, ID};

mod callback;
mod internal;
mod public;

#[derive(Fail, Debug)]
pub enum APIErr {
    #[fail(display = "API bind error {}", _0)]
    BindError(#[cause] std::io::Error),
    #[fail(display = "Instance incorrect {}", _0)]
    InvalidInstane(ID),
}

pub fn start_server(
    runtime: &mut runtime::Runtime,
    instances: Instances,
    base: InstanceBase,
) -> Fallible<()> {
    internal::start_server(runtime, instances.clone())?;
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

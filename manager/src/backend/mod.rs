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

pub mod callback;

use failure::Fallible;
use futures::future::Future;
use reqwest::{self, header, r#async::*};
use serde::Serialize;
use std::fmt::Debug;
use tokio::{
    executor::{DefaultExecutor, Executor, SpawnError},
    runtime::Runtime,
};
use yamba_types::models::{self, callback as cb};

use std::net::SocketAddr;
use std::sync::Arc;

use crate::instance::Instances;

#[derive(Fail, Debug)]
pub enum BackendErr {
    #[fail(display = "Failed to execute future {}", _0)]
    ExcecutionFailed(#[cause] SpawnError),
}

#[derive(Clone)]
pub struct Backend {
    addr: SocketAddr,
    client: Client,
    instances: Instances,
}

impl Backend {
    /// Create new backend endpoint
    pub fn new(
        addr: SocketAddr,
        instances: Instances,
        api_secret: &str,
    ) -> Fallible<(Backend, callback::ShutdownGuard)> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(&"YAMBA middleware"),
        );
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(api_secret)?,
        );
        let backend = Backend {
            client: ClientBuilder::new().default_headers(headers).build()?,
            instances,
            addr,
        };

        let shutdown_guard = callback::init_callback_server(backend.clone())?;

        Ok((backend, shutdown_guard))
    }

    /// Spawn future on default executor
    pub fn spawn_on_default<T>(fut: T) -> Fallible<()>
    where
        T: Future<Item = (), Error = ()> + Send + 'static,
    {
        DefaultExecutor::current()
            .spawn(Box::new(fut))
            .map_err(|v| BackendErr::ExcecutionFailed(v))?;
        Ok(())
    }

    /// Spawn on default, simply printing the result
    pub fn spawn_ignore<T, V, E>(fut: T) -> Fallible<()>
    where
        T: Future<Item = V, Error = E> + Send + 'static,
        V: Debug,
        E: Debug,
    {
        DefaultExecutor::current()
            .spawn(Box::new(
                fut.map(|x| trace!("Request response: {:?}", x))
                    .map(|_| ())
                    .map_err(|err| warn!("Error sending api request: {:?}", err)),
            ))
            .map_err(|v| BackendErr::ExcecutionFailed(v))?;
        Ok(())
    }

    /// Stop instance
    pub fn stop_instance(
        &self,
        inst: &models::InstanceStopReq,
    ) -> Fallible<impl Future<Item = models::DefaultResponse, Error = reqwest::Error>> {
        let fut = self
            .get_request_base(&format!("http://{}/instance/stop", self.addr), inst)?
            .and_then(|mut x| x.json::<models::DefaultResponse>());
        Ok(fut)
    }

    /// Create instance
    pub fn create_instance(
        &self,
        inst: &models::InstanceLoadReq,
    ) -> Fallible<impl Future<Item = models::DefaultResponse, Error = reqwest::Error>> {
        let fut = self
            .get_request_base(&format!("http://{}/instance/start", self.addr), inst)?
            .and_then(|mut x| x.json::<models::DefaultResponse>());
        Ok(fut)
    }

    /// Perform request, ignoring outcome
    fn request_ignore<T>(
        &self,
        addr: &str,
        data: &T,
    ) -> Fallible<impl Future<Item = (), Error = ()>>
    where
        T: Serialize,
    {
        Ok(self
            .get_request_base(addr, data)?
            .map(|x| trace!("Request response: {:?}", x))
            .map_err(|err| warn!("Error sending api request: {:?}", err)))
    }

    /// Create request base
    fn get_request_base<T>(
        &self,
        addr: &str,
        data: &T,
    ) -> Fallible<impl Future<Item = Response, Error = reqwest::Error>>
    where
        T: Serialize,
    {
        Ok(self.client.post(addr).json(data).send())
    }
}

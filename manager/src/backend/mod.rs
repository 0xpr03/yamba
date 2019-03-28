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

pub mod callback;
pub mod tickets;

use actix::spawn;
use failure::Fallible;
use futures::future::Future;
use reqwest::{self, header, r#async::*};
use serde::Serialize;
use yamba_types::models;

use std::fmt::Debug;
use std::net::SocketAddr;

use self::tickets::TicketHandler;

use crate::instance::Instances;

#[derive(Clone)]
pub struct Backend {
    addr: SocketAddr,
    client: Client,
    tickets: TicketHandler,
}

impl Backend {
    /// Create new backend endpoint
    pub fn new(
        addr: SocketAddr,
        instances: Instances,
        api_secret: &str,
        callback_bind: SocketAddr,
    ) -> Fallible<Backend> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(&"YAMBA middleware"),
        );
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(api_secret)?,
        );
        let tickets = TicketHandler::new();
        let backend = Backend {
            client: ClientBuilder::new().default_headers(headers).build()?,
            addr,
            tickets: tickets.clone(),
        };

        let shutdown_guard =
            callback::init_callback_server(backend.clone(), instances, callback_bind, tickets)?;

        Ok(backend)
    }

    /// Returns ticket handler
    pub fn get_tickets(&self) -> &TicketHandler {
        &self.tickets
    }

    /// Spawn future on default executor
    pub fn spawn_on_default<T>(fut: T) -> Fallible<()>
    where
        T: Future<Item = (), Error = ()> + Send + 'static,
    {
        spawn(fut);
        Ok(())
    }

    /// Spawn on default, simply printing the result
    pub fn spawn_ignore<T, V, E>(fut: T) -> Fallible<()>
    where
        T: Future<Item = V, Error = E> + Send + 'static,
        V: Debug,
        E: Debug,
    {
        spawn(
            fut.map(|x| trace!("Request response: {:?}", x))
                .map(|_| ())
                .map_err(|err| warn!("Error sending api request: {:?}", err)),
        );
        Ok(())
    }

    /// Stop instance
    pub fn stop_instance(
        &self,
        inst: &models::InstanceStopReq,
    ) -> Fallible<impl Future<Item = models::DefaultResponse, Error = reqwest::Error>> {
        let fut = self
            .get_request_base(&format!("http://{}/instance/stop", self.addr), inst, true)?
            .and_then(|mut x| x.json::<models::DefaultResponse>());
        Ok(fut)
    }

    /// Create instance
    pub fn create_instance(
        &self,
        inst: &models::InstanceLoadReq,
    ) -> Fallible<impl Future<Item = models::DefaultResponse, Error = reqwest::Error>> {
        let fut = self
            .get_request_base(&format!("http://{}/instance/start", self.addr), inst, true)?
            .and_then(|mut x| x.json::<models::DefaultResponse>());
        Ok(fut)
    }

    /// Start URL playback
    pub fn play_url(
        &self,
        request: &models::PlaybackUrlReq,
    ) -> Fallible<impl Future<Item = models::DefaultResponse, Error = reqwest::Error>> {
        let fut = self
            .get_request_base(&format!("http://{}/playback/url", self.addr), request, true)?
            .and_then(|mut x| x.json::<models::DefaultResponse>());
        Ok(fut)
    }

    /// Resolve URL request
    pub fn resolve_url(
        &self,
        request: &models::ResolveRequest,
    ) -> Fallible<impl Future<Item = models::ResolveTicketResponse, Error = reqwest::Error>> {
        trace!("Resolving url {}", request.url);
        let fut = self
            .get_request_base(&format!("http://{}/resolve/url", self.addr), request, false)?
            .and_then(|mut x| {
                debug!("Resolve response: {:?} {:?}", x, x.body());
                x.json::<models::ResolveTicketResponse>()
            });
        Ok(fut)
    }

    /// Set Volume request
    pub fn set_volume(
        &self,
        request: &models::VolumeSetReq,
    ) -> Fallible<impl Future<Item = models::DefaultResponse, Error = reqwest::Error>> {
        let fut = self
            .get_request_base(&format!("http://{}/volume", self.addr), request, true)?
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
            .get_request_base(addr, data, true)?
            .map(|x| trace!("Request response: {:?}", x))
            .map_err(|err| warn!("Error sending api request: {:?}", err)))
    }

    /// Create request base
    fn get_request_base<T>(
        &self,
        addr: &str,
        data: &T,
        post: bool,
    ) -> Fallible<impl Future<Item = Response, Error = reqwest::Error>>
    where
        T: Serialize,
    {
        Ok(if post {
            self.client.post(addr).json(data).send()
        } else {
            self.client.get(addr).query(data).send()
        })
    }
}

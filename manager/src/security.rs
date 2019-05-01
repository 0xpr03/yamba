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

use actix_web::{
    error::Result,
    middleware::{Middleware, Started},
    HttpRequest, HttpResponse,
};
use std::net::{IpAddr, SocketAddr};

/// Actix security module to allow only a specific IP
pub struct SecurityModule {
    ip: String,
}

impl SecurityModule {
    pub fn new(addr: IpAddr) -> SecurityModule {
        SecurityModule {
            ip: addr.to_string(),
        }
    }
}

impl<S> Middleware<S> for SecurityModule {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        if let Some(remote) = req.connection_info().remote() {
            if remote
                .parse::<SocketAddr>()
                .map(|v| v.ip().to_string() == self.ip)
                .unwrap_or_else(|e| {
                    warn!("Can't parse remote IP! {}", e);
                    false
                })
            {
                return Ok(Started::Done);
            } else {
                debug!("Remote: {} Own: {}", remote, self.ip);
            }
        }
        Ok(Started::Response(HttpResponse::Unauthorized().finish()))
    }
}

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

use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{http, Error, HttpResponse};
use futures::future::{ok, Either, FutureResult};
use futures::Poll;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

/// Actix security module to allow only a specific IP
pub struct SecurityModule {
    ip: Arc<String>,
}

impl<S, B> Transform<S> for SecurityModule
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityModuleMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SecurityModuleMiddleware {
            service,
            ip: self.ip.clone(),
        })
    }
}

pub struct SecurityModuleMiddleware<S> {
    service: S,
    ip: Arc<String>,
}

impl SecurityModule {
    pub fn new(addr: IpAddr) -> SecurityModule {
        SecurityModule {
            ip: Arc::new(addr.to_string()),
        }
    }
}

impl<S, B> Service for SecurityModuleMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, FutureResult<Self::Response, Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }
    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let mut correct_ip = false;
        if let Some(remote) = req.connection_info().remote() {
            if remote
                .parse::<SocketAddr>()
                .map(|v| v.ip().to_string() == *self.ip)
                .unwrap_or_else(|e| {
                    warn!("Can't parse remote IP! {}", e);
                    false
                })
            {
                correct_ip = true;
            } else {
                debug!("Remote: {} Own: {}", remote, self.ip);
            }
        }

        match correct_ip {
            false => Either::B(ok(
                req.into_response(HttpResponse::Unauthorized().finish().into_body())
            )),
            true => Either::A(self.service.call(req)),
        }
    }
}

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

use atomic::{Atomic, Ordering};
use erased_serde::Serialize as ESerialize;
use failure::Fallible;
use hyper;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use reqwest;
use reqwest::header::USER_AGENT;
use serde::de::Deserialize;
use serde::Serialize;
use serde_json;
use tokio::runtime;

use std::sync::Arc;

use daemon::{self, BoxFut, Instances};
use ytdl_worker::{RSongs, YTRequest, YTSender};
use SETTINGS;
use USERAGENT;

/// API server for frontend requests

#[macro_export]
macro_rules! enum_number {
    ($name:ident { $($variant:ident = $value:expr, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant = $value,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                // Serialize the enum as a u64.
                serializer.serialize_u64(*self as u64)
            }
        }
    }
}

pub type RequestID = u32;

//#[derive(Clone, Eq, PartialEq, Copy, Debug, Serialize)]
#[allow(non_camel_case_types)]
#[repr(i32)]
enum_number! (CallbackErrorType {
    NoError = 0,
    UnknownError = -1,
});

#[derive(Fail, Debug)]
pub enum APIErr {
    #[fail(display = "API bind error {}", _0)]
    BindError(#[cause] hyper::error::Error),
}

/// URL Resolve API call struct
#[derive(Serialize, Deserialize, Debug)]
pub struct UrlResolve {
    pub url: String,
}

/// Playlist API callback structure
#[derive(Debug, Serialize)]
pub struct PlaylistAnswer {
    pub request_id: RequestID,
    pub song_ids: Vec<String>,
    pub error_code: CallbackErrorType,
}

/// Used for returning errors on failure callbacks
#[derive(Debug, Serialize)]
pub struct CallbackError {
    pub request_id: RequestID,
    pub message: String,
    pub error_code: CallbackErrorType,
}

//const COUNTER: Arc<Atomic<u32>> = Arc::new(Atomic::new(0));
/// API main handler for requests
fn api(req: Request<Body>, counter: Arc<Atomic<u32>>, channel: YTSender) -> BoxFut {
    let mut response: Response<Body> = Response::new(Body::empty());
    let (parts, body) = req.into_parts();
    Box::new(body.concat2().map(move |b| {
        match (parts.method, parts.uri.path()) {
            (Method::GET, "/") => {
                response = Response::new(Body::empty());
                *response.body_mut() =
                    Body::from("Hello, this is part of an rest-like API, see docs");
                *response.status_mut() = StatusCode::IM_A_TEAPOT;
            }
            (Method::POST, "/new/titles") => {
                response = handle_request(counter, &b, resolve_url, channel);
            }
            (_, v) => {
                info!("Unknown API request {}", v);
                response = Response::new(Body::empty());
                *response.status_mut() = StatusCode::NOT_FOUND;
            }
        };
        response
    }))
}

/// Request abstraction, parses input json to specified struct, calls handler on the result
fn handle_request<'a, T, F>(
    req_counter: Arc<Atomic<u32>>,
    data: &'a [u8],
    mut handler: F,
    channel: YTSender,
) -> Response<Body>
where
    F: FnMut(T, &mut Response<Body>, u32, YTSender) -> Fallible<()>,
    T: Deserialize<'a>,
{
    let mut response = Response::new(Body::empty());
    match serde_json::from_slice::<T>(data) {
        Ok(v) => {
            debug!("Parsed request");
            let req_id = req_counter.fetch_add(1, Ordering::AcqRel);
            let mut result = handler(v, &mut response, req_id, channel);
            if result.is_ok() {
                trace!("Processed api call");
                *response.status_mut() = StatusCode::ACCEPTED;
            }
            if let Err(e) = result {
                info!("Error while processing request: {}", e);
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                // wipe body ?
            }
        }
        Err(e) => {
            info!("Invalid request, parsing error: {}", e);
            *response.status_mut() = StatusCode::BAD_REQUEST;
        }
    }
    response
}

struct PlaylistReq {
    url: String,
    request_id: RequestID,
}

impl YTRequest for PlaylistReq {
    fn url(&self) -> &str {
        &self.url
    }

    fn callback(&mut self, songs: RSongs, _: Instances) {
        let response = match songs {
            Ok(s) => Ok(PlaylistAnswer {
                request_id: self.request_id,
                song_ids: s.into_iter().map(|song| song.id).collect(),
                error_code: CallbackErrorType::NoError,
            }),
            Err(e) => Err(CallbackError {
                request_id: self.request_id,
                message: format!("{}", e),
                error_code: CallbackErrorType::UnknownError,
            }),
        };

        let response: Box<ESerialize> = match response {
            Ok(v) => Box::new(v),
            Err(e) => Box::new(e),
        };

        match api_send_callback(
            &SETTINGS.main.api_callback_ip,
            SETTINGS.main.api_callback_port,
            "music/addTitles",
            &response,
        ) {
            Ok(_) => info!("Callback successfull"),
            Err(e) => warn!("Callback errored: {}", e),
        }
    }
}

/// Handle url resolve request
fn resolve_url(
    data: UrlResolve,
    response: &mut Response<Body>,
    request_id: RequestID,
    channel: YTSender,
) -> Fallible<()> {
    info!("URL: {}", data.url);
    *response.status_mut() = StatusCode::ACCEPTED;
    *response.body_mut() = serde_json::to_string(&json!({ "request_id": &request_id }))
        .unwrap()
        .into();
    channel.try_send(
        PlaylistReq {
            url: data.url,
            request_id,
        }
        .wrap(),
    )?;
    Ok(())
}

/// Check config for API startup, returns error is not clean for startup
pub fn check_config() -> Fallible<()> {
    let _ = daemon::parse_socket_address(&SETTINGS.main.api_bind_ip, SETTINGS.main.api_bind_port)?;
    Ok(())
}

/// Create api server, bind it & attach to runtime
pub fn create_api_server(runtime: &mut runtime::Runtime, channel: YTSender) -> Fallible<()> {
    let addr =
        daemon::parse_socket_address(&SETTINGS.main.api_bind_ip, SETTINGS.main.api_bind_port)?;

    debug!(
        "{}",
        serde_json::to_string(&UrlResolve { url: "asd".into() }).unwrap()
    );

    if Atomic::<u32>::is_lock_free() {
        debug!("Passed atomic test");
    } else {
        warn!("Atomics test failed for platform!");
    }

    let counter = Arc::new(Atomic::new(0));

    let server = Server::try_bind(&addr)
        .map_err(|e| APIErr::BindError(e))?
        .serve(move || {
            let counter = counter.clone();
            let channel = channel.clone();
            service_fn(move |req: Request<Body>| api(req, counter.clone(), channel.clone()))
        })
        .map_err(|e| error!("server error: {}", e));

    info!("API Listening on http://{}", addr);
    runtime.spawn(server);
    Ok(())
}

/// Perform api callback with specified message
pub fn api_send_callback<T>(host: &str, port: u16, dir: &str, msg: &T) -> Fallible<()>
where
    T: Serialize,
{
    let agent: &str = &USERAGENT;

    let mut result = reqwest::Client::new()
        .post(&format!("http://{}:{}/{}", host, port, dir))
        .header(USER_AGENT, agent)
        .json(msg)
        .send()?;

    let body_text = result.text();

    trace!("Callback response: {:?} body: {:?}", result, body_text);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use SETTINGS;
    #[test]
    #[ignore]
    fn header_test() {
        api_send_callback(
            "localhost",
            SETTINGS.main.api_callback_port,
            "music/addSongs/",
            &CallbackErrorType::NoError,
        )
        .unwrap();
    }
}

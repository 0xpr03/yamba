use atomic::{Atomic, Ordering};
use failure::Fallible;
use hyper;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use reqwest;
use reqwest::header::HeaderMap;
use reqwest::header::{
    ACCEPT, ACCEPT_ENCODING, CONNECTION, CONTENT_ENCODING, LOCATION, USER_AGENT,
};
use reqwest::{Client, Response as RQResponse};
use serde::de::Deserialize;
use serde::Serialize;
use serde_json;
use tokio::runtime;

use std::sync::Arc;

use daemon::{self, APIChannel, BoxFut};
use SETTINGS;
use USERAGENT;

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

/// Playlist API call struct
#[derive(Serialize, Deserialize, Debug)]
pub struct NewPlaylist {
    pub url: String,
}

/// Playlist API callback structure
#[derive(Debug, Serialize)]
pub struct PlaylistAnswer {
    pub request_id: u32,
    pub song_ids: Vec<String>,
    pub error_code: CallbackErrorType,
}

/// Used for returning errors on failure callbacks
#[derive(Debug, Serialize)]
pub struct CallbackError {
    pub request_id: u32,
    pub message: String,
    pub error_code: CallbackErrorType,
}

/// API Request containing its ID and the request type
#[derive(Debug)]
pub struct APIRequest {
    pub request_id: u32,
    /// Callback (web API)
    pub callback: bool,
    pub request_type: RequestType,
}

/// Request type
#[derive(Debug)]
pub enum RequestType {
    Playlist(NewPlaylist),
}

//const COUNTER: Arc<Atomic<u32>> = Arc::new(Atomic::new(0));

fn api(req: Request<Body>, counter: Arc<Atomic<u32>>, channel: APIChannel) -> BoxFut {
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
            (Method::POST, "/new/playlist") => {
                response = handle_request(counter, &b, new_playlist, channel);
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

/// Request abstraction, parses input json to struct, calls handler on the result
fn handle_request<'a, T, F>(
    req_counter: Arc<Atomic<u32>>,
    data: &'a [u8],
    mut handler: F,
    channel: APIChannel,
) -> Response<Body>
where
    F: FnMut(T, &mut Response<Body>, u32, APIChannel) -> Fallible<()>,
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

/// Handle playlist request
fn new_playlist(
    playlist: NewPlaylist,
    response: &mut Response<Body>,
    request_id: u32,
    mut channel: APIChannel,
) -> Fallible<()> {
    info!("URL: {}", playlist.url);
    *response.status_mut() = StatusCode::ACCEPTED;
    *response.body_mut() = serde_json::to_string(&json!({ "request id": &request_id }))
        .unwrap()
        .into();
    let job = APIRequest {
        request_id,
        callback: true,
        request_type: RequestType::Playlist(playlist),
    };
    channel.try_send(job)?;
    Ok(())
}

pub fn check_config() -> Fallible<()> {
    let _ = daemon::parse_socket_address(&SETTINGS.main.api_bind_ip, SETTINGS.main.api_bind_port)?;
    Ok(())
}

/// Create api server, bind it & attach to runtime
pub fn create_api_server(runtime: &mut runtime::Runtime, channel: APIChannel) -> Fallible<()> {
    let addr =
        daemon::parse_socket_address(&SETTINGS.main.api_bind_ip, SETTINGS.main.api_bind_port)?;

    debug!(
        "{}",
        serde_json::to_string(&NewPlaylist { url: "asd".into() }).unwrap()
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
        }).map_err(|e| eprintln!("server error: {}", e));

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

    let result = reqwest::Client::new()
        .post(&format!("http://{}:{}/{}", host, port, dir))
        .header(USER_AGENT, agent)
        .json(msg)
        .send()?;

    trace!("Callback response: {:?}", result);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    /// Test header creation
    #[test]
    fn header_test() {
        api_send_callback("localhost", 9000, &CallbackErrorType::NoError).unwrap();
    }
}

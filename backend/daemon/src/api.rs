use atomic::{Atomic, Ordering};
use failure::Fallible;
use futures::future;
use hyper;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde::de::Deserialize;
use serde_json::{self, to_value};
use tokio::runtime;

use std::sync::Arc;

use daemon::{self, BoxFut};
use SETTINGS;

#[derive(Fail, Debug)]
pub enum APIErr {
    #[fail(display = "API bind error {}", _0)]
    BindError(#[cause] hyper::error::Error),
}

#[derive(Serialize, Deserialize, Debug)]
struct NewPlaylist {
    url: String,
}

//const COUNTER: Arc<Atomic<u32>> = Arc::new(Atomic::new(0));

fn api(req: Request<Body>, counter: Arc<Atomic<u32>>) -> BoxFut {
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
                response = handle_request(counter, &b, new_playlist);
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
) -> Response<Body>
where
    F: FnMut(T, &mut Response<Body>, u32) -> Fallible<()>,
    T: Deserialize<'a>,
{
    let mut response = Response::new(Body::empty());
    match serde_json::from_slice::<T>(data) {
        Ok(v) => {
            debug!("Parsed request");
            let req_id = req_counter.fetch_add(1, Ordering::AcqRel);
            let mut result = handler(v, &mut response, req_id);
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

fn new_playlist(
    playlist: NewPlaylist,
    response: &mut Response<Body>,
    request_id: u32,
) -> Fallible<()> {
    info!("URL: {}", playlist.url);
    *response.status_mut() = StatusCode::ACCEPTED;
    *response.body_mut() = serde_json::to_string(&json!({ "request id": request_id }))
        .unwrap()
        .into();
    Ok(())
}

pub fn check_config() -> Fallible<()> {
    let _ = daemon::parse_socket_address(&SETTINGS.main.api_bind_ip, SETTINGS.main.api_bind_port)?;
    Ok(())
}

/// Create api server, bind it & attach to runtime
pub fn create_api_server(runtime: &mut runtime::Runtime) -> Fallible<()> {
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
            service_fn(move |req: Request<Body>| api(req, counter.clone()))
        }).map_err(|e| eprintln!("server error: {}", e));

    info!("API Listening on http://{}", addr);
    runtime.spawn(server);
    Ok(())
}

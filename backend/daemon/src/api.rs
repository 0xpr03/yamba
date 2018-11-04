use atomic::{Atomic, Ordering};
use failure::Fallible;
use futures::future;
use hyper;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
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

#[derive(Deserialize, Debug)]
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
                let request_id = counter.fetch_add(1, Ordering::AcqRel);
                response = match new_playlist(&b, request_id) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Error processing {}", e);
                        let mut response_ = Response::new(Body::empty());
                        *response_.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        response_
                    }
                };
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

fn new_playlist(data: &[u8], request_id: u32) -> Fallible<Response<Body>> {
    let mut response = Response::new(Body::empty());
    match serde_json::from_slice::<NewPlaylist>(data) {
        Ok(v) => {
            info!("URL: {}", v.url);
            *response.status_mut() = StatusCode::ACCEPTED;
            *response.body_mut() = to_value(json!({ "request id": request_id }))
                .unwrap()
                .to_string()
                .into()
        }
        Err(e) => {
            info!("API invalid request {}", e);
            *response.status_mut() = StatusCode::BAD_REQUEST;
        }
    }
    Ok(response)
}

pub fn check_config() -> Fallible<()> {
    let _ = daemon::parse_socket_address(&SETTINGS.main.api_bind_ip, SETTINGS.main.api_bind_port)?;
    Ok(())
}

/// Create api server, bind it & attach to runtime
pub fn create_api_server(runtime: &mut runtime::Runtime) -> Fallible<()> {
    let addr =
        daemon::parse_socket_address(&SETTINGS.main.api_bind_ip, SETTINGS.main.api_bind_port)?;

    /*let api = API {
        request_counter: Arc::new(Atomic::new(0)),
    };*/
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

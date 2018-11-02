use failure::Fallible;
use futures::future;
use hyper;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde_json::{self, to_value};
use tokio::runtime;

use daemon;
use SETTINGS;

#[derive(Fail, Debug)]
pub enum APIErr {
    #[fail(display = "API bind error {}", _0)]
    BindError(#[cause] hyper::error::Error),
}

pub fn api(req: Request<Body>) -> BoxFut {
    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => {
            *response.body_mut() = Body::from("Hello, this is part of an rest-like API, see docs");
            *response.status_mut() = StatusCode::IM_A_TEAPOT;
        }

        // Simply echo the body back to the client.
        (&Method::POST, "/new/playlist") => {
            *response.body_mut() = req.into_body();
            *response.status_mut() = StatusCode::ACCEPTED;
        }

        // Convert to uppercase before sending back to client.
        (&Method::GET, "/get/state") => {
            let mapping = req.into_body().map(|chunk| {
                chunk
                    .iter()
                    .map(|byte| byte.to_ascii_uppercase())
                    .collect::<Vec<u8>>()
            });

            *response.body_mut() = Body::wrap_stream(mapping);
        }
        // The 404 Not Found route...
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    Box::new(future::ok(response))
}

pub fn check_config() -> Fallible<()> {
    let _ = daemon::parse_socket_address(&SETTINGS.main.api_bind_ip, &SETTINGS.main.api_bind_port)?;
    Ok(())
}

/// Create api server, bind it & attach to runtime
pub fn create_api_server(runtime: &mut runtime::Runtime) -> Fallible<()> {
    let addr =
        daemon::parse_socket_address(&SETTINGS.main.api_bind_ip, &SETTINGS.main.api_bind_port)?;

    let server = Server::try_bind(&addr)
        .map_err(|e| APIErr::BindError(e))?
        .serve(|| service_fn(rpc))
        .map_err(|e| eprintln!("server error: {}", e));

    info!("API Listening on http://{}", addr);
    runtime.spawn(server);
    Ok(())
}

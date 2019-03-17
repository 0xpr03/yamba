use crate::instance::Instances;
use actix::System;
use actix_web::{
    error::Result,
    fs, http,
    middleware::{self, Middleware, Started},
    server, App, AsyncResponder, Error, Form, HttpMessage, HttpRequest, HttpResponse, Json, State,
};
use failure::Fallible;
use futures::{sync::mpsc, Future, Stream};
use std::net::IpAddr;
use std::net::SocketAddr;
use std::thread;
use tokio::{
    executor::{DefaultExecutor, Executor, SpawnError},
    runtime::Runtime,
};

pub fn handle_create_ts(
    (state, params): (State<Instances>, Form<super::form::TSCreate>),
) -> Result<HttpResponse> {
    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/plain")
        .body(format!("Accepted {} {:?}", params.ip, params.cid)))
}

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

#[macro_use]
extern crate failure;
use clap::{App, Arg, ArgMatches};
use failure::Fallible;
#[macro_use]
extern crate log;
#[cfg(any(feature = "mysql", feature = "postgres"))]
#[macro_use]
extern crate diesel;
use actix::System;
use env_logger::{self, Env};
use futures::future::Future;
use futures::stream::Stream;
use tokio_signal;
use yamba_types::models::{InstanceType, TSSettings};

use crate::db::Database;
use std::net::SocketAddr;

mod backend;
mod db;
mod frontend;
mod instance;
mod jsonrpc;
mod models;
mod playlist;
mod security;

#[cfg(any(feature = "maria", feature = "postgres"))]
const DB_DEFAULT_PATH: &'static str = "127.0.0.1:3306";
#[cfg(feature = "local")]
const DB_DEFAULT_PATH: &'static str = "sled_db.db";

#[derive(Fail, Debug)]
pub enum ParamErr {
    #[fail(display = "TS Address is invalid {}", _0)]
    InvalidTSAddress(#[cause] std::num::ParseIntError),
}

fn main() -> Fallible<()> {
    env_logger::from_env(Env::default().default_filter_or("manager=trace")).init();
    trace!("Starting yamba manager");
    let matches = App::new("YAMBA middleware")
        .version("0")
        .author("Aron Heinecke <aron.heinecke@t-online.de>")
        .arg(
            Arg::with_name("daemon")
                .short("d")
                .long("daemon")
                .value_name("IP:PORT")
                .help("Set daemon Address to use")
                .default_value("127.0.0.1:1338")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("callback")
                .short("c")
                .long("callback")
                .value_name("IP:PORT")
                .help("Set daemon callback bind to use")
                .default_value("127.0.0.1:1336")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("api_secret")
                .short("s")
                .long("secret")
                .value_name("Secret")
                .help("Secret to send to daemon")
                .default_value("MySecret")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("jsonrpc")
                .short("j")
                .long("rpc")
                .value_name("IP:PORT")
                .help("Bind address for jsonrpc calls")
                .default_value("127.0.0.1:1337")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("frontend")
                .short("b")
                .long("bind")
                .value_name("IP:PORT")
                .help("Set bind Address to use for frontend")
                .default_value("127.0.0.1:8080")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ts")
                .long("ts")
                .value_name("IP:PORT")
                .help("Create one ts instance")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("channel")
                .long("cid")
                .requires("ts")
                .value_name("Channel ID")
                .help("Specify channel to connect to")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("pw")
                .long("pw")
                .requires("ts")
                .value_name("Connect PW")
                .help("Specify pw for connecting")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("db")
                .long("db")
                .value_name("DB connection URI")
                .help("Specify for DB connection / path depending on the DB compilation type")
                .takes_value(true)
                .default_value(DB_DEFAULT_PATH),
        )
        .get_matches();

    let addr_daemon: SocketAddr = matches.value_of("daemon").unwrap().parse()?;
    let addr_frontend: SocketAddr = matches.value_of("frontend").unwrap().parse()?;
    let addr_jsonrpc: SocketAddr = matches.value_of("jsonrpc").unwrap().parse()?;
    let addr_callback_bind: SocketAddr = matches.value_of("callback").unwrap().parse()?;
    let api_secret = matches.value_of("api_secret").unwrap();
    let db_path = matches.value_of("db").unwrap();

    let mut sys = System::new("manager");

    let db = db::DB::create(db_path.to_owned())?;

    let instances = instance::Instances::new(db.clone());

    let backend = backend::Backend::new(
        addr_daemon,
        instances.clone(),
        api_secret,
        addr_callback_bind,
    )?;

    let _server = jsonrpc::create_server(&addr_jsonrpc, addr_daemon.ip(), instances.clone())?;

    match create_instance_cmd(&backend, &instances, &matches) {
        Err(e) => error!("Error during test-cmd handling: {}", e),
        Ok(_) => (),
    }

    frontend::init_frontend_server(instances.clone(), backend.clone(), addr_frontend)?;

    instances.load_instances(backend.clone())?;

    let ctrl_c = tokio_signal::ctrl_c().flatten_stream().into_future();

    match sys.block_on(ctrl_c) {
        Err(_e) => {
            // first tuple element conains error, but is neither display nor debug..
            error!("Error in signal handler");
            println!("Shutting down daemon..");
        }
        Ok(_) => (),
    };
    Ok(())
}

/// Create cmd ts instance if applicable
fn create_instance_cmd(
    backend: &backend::Backend,
    instances: &instance::Instances,
    args: &ArgMatches,
) -> Fallible<()> {
    if let Some(addr) = args.value_of("ts") {
        let _pw = args.value_of("pw");
        let cid: Option<i32> = match args
            .value_of("channel")
            .map(|v| v.parse::<i32>().map_err(|e| ParamErr::InvalidTSAddress(e)))
        {
            None => None,
            Some(Err(e)) => return Err(e.into()),
            Some(Ok(i)) => Some(i),
        };
        let addr: SocketAddr = addr.parse()?;
        let model = models::NewInstance {
            autostart: true,
            host: addr.ip().to_string(),
            port: Some(addr.port()),
            identity: None,
            cid: cid,
            name: String::from("test_instance"),
            password: None,
            nick: String::from("TestYambaInstance"),
        };

        instances.create_instance(model, backend.clone())?;

        let mut inst_w = instances.write().expect("Can't lock instances!");
        let inst = inst_w.get_mut(&1).expect("Invalid identifier ?");
        inst.start_with_rt()?;
    }

    Ok(())
}

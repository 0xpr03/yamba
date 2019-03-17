/*
 *  YAMBA middleware
 *  Copyright (C) 2019 Aron Heinecke
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

#[macro_use]
extern crate failure;
use clap::{App, Arg, ArgMatches};
use failure::Fallible;
#[macro_use]
extern crate log;
use env_logger::{self, Env};
use futures::future::{self, Future};
use futures::stream::Stream;
use tokio::runtime::Runtime;
use tokio_signal;
use yamba_types::models::{InstanceLoadReq, InstanceType, TSSettings};

use std::net::SocketAddr;

mod backend;
mod frontend;
mod instance;
mod jsonrpc;
mod playlist;

const MUSIC_PATH: &'static str = "/datadisk/Music/";

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
        .get_matches();

    let addr_daemon: SocketAddr = matches.value_of("daemon").unwrap().parse()?;
    let addr_frontend: SocketAddr = matches.value_of("frontend").unwrap().parse()?;
    let addr_jsonrpc: SocketAddr = matches.value_of("jsonrpc").unwrap().parse()?;
    let api_secret = matches.value_of("api_secret").unwrap();

    let instances = instance::create_instances();

    let mut runtime = Runtime::new()?;

    let (backend, _shutdown_guard) =
        backend::Backend::new(addr_daemon, instances.clone(), api_secret)?;

    let _server = jsonrpc::create_server(
        &addr_jsonrpc,
        &addr_daemon.ip().to_string(),
        instances.clone(),
    )?;

    match create_instance_cmd(&backend, &instances, &matches, &mut runtime) {
        Err(e) => error!("Error during test-cmd handling: {}", e),
        Ok(_) => (),
    }

    let _shutdown_guard_frontend =
        frontend::init_frontend_server(instances.clone(), addr_frontend)?;

    let ctrl_c = tokio_signal::ctrl_c().flatten_stream().into_future();

    match runtime.block_on(ctrl_c) {
        Err(e) => {
            // first tuple element conains error, but is neither display nor debug..
            error!("Error in signal handler");
            println!("Shutting down daemon..");
        }
        Ok(_) => (),
    };

    drop(_shutdown_guard);
    Ok(())
}

/// Create cmd ts instance if applicable
fn create_instance_cmd(
    backend: &backend::Backend,
    instances: &instance::Instances,
    args: &ArgMatches,
    rt: &mut Runtime,
) -> Fallible<()> {
    if let Some(addr) = args.value_of("ts") {
        let pw = args.value_of("pw");
        let cid: Option<i32> = match args
            .value_of("channel")
            .map(|v| v.parse::<i32>().map_err(|e| ParamErr::InvalidTSAddress(e)))
        {
            None => None,
            Some(Err(e)) => return Err(e.into()),
            Some(Ok(i)) => Some(i),
        };
        let addr: SocketAddr = addr.parse()?;
        let model = InstanceLoadReq {
            id: 0,
            volume: 0.05,
            data: InstanceType::TS(TSSettings {
                host: addr.ip().to_string(),
                port: Some(addr.port()),
                identity: "".to_string(),
                cid: cid,
                name: String::from("test_instance"),
                password: None,
            }),
        };

        let mut inst_w = instances.write().expect("Can't lock instances!");
        inst_w.insert(1, instance::Instance::new(1, backend.clone(), model));

        let inst = inst_w.get_mut(&1).expect("Invalid identifier ?");
        inst.start_with_rt(rt)?;
    }

    Ok(())
}

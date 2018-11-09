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
#![recursion_limit = "1024"]
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate vlc;
#[macro_use]
extern crate clap;
extern crate log4rs;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate config as config_rs;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate futures;
extern crate hyper;
extern crate jsonrpc_lite;
#[macro_use]
extern crate serde_json;
extern crate atomic;
extern crate rusqlite;
extern crate serde_urlencoded;
extern crate sha2;
extern crate tokio;
extern crate tokio_signal;

use std::alloc::System;

#[global_allocator]
static GLOBAL: System = System;

mod api;
mod config;
mod daemon;
mod http;
mod models;
mod playback;
mod rpc;
mod ts;
mod ytdl;

use clap::{App, Arg, SubCommand};
use failure::Fallible;

use std::fs::{metadata, DirBuilder, File};
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

const DEFAULT_CONFIG_NAME: &'static str = "00-config.toml";
const CONF_DIR: &'static str = "conf";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const LOG_PATH: &'static str = "conf/daemon_log.yml";

lazy_static! {
    static ref SETTINGS: config::ConfigRoot = {
        info!("Loading config..");
        config::init_settings().unwrap()
    };
}

fn main() -> Fallible<()> {
    println!("Starting yamba daemon {}, switching to logger.", VERSION);

    init_log()?;

    info!("Startup");

    let app = App::new("YAMBA")
        .version(VERSION)
        .author(crate_authors!(",\n"))
        .about("yamba backend, VoIP music bot")
        .subcommand(SubCommand::with_name("init").about("Initialize database on first execution"))
        .subcommand(
            SubCommand::with_name("play-audio")
                .about("Test command to play audio")
                .arg(
                    Arg::with_name("file")
                        .short("f")
                        .required(true)
                        .validator(validator_path)
                        .takes_value(true)
                        .help("audio file"),
                ),
        ).subcommand(
            SubCommand::with_name("test-url")
                .about("Test playback on url, for test use")
                .arg(
                    Arg::with_name("url")
                        .short("u")
                        .required(true)
                        .takes_value(true)
                        .help("media url"),
                ),
        ).subcommand(
            SubCommand::with_name("test-ts")
                .about("Test ts instance start, for test use")
                .arg(
                    Arg::with_name("host")
                        .short("h")
                        .required(true)
                        .takes_value(true)
                        .help("host address"),
                ).arg(
                    Arg::with_name("port")
                        .short("p")
                        .required(true)
                        .takes_value(true)
                        .help("port address"),
                ).arg(
                    Arg::with_name("cid")
                        .required(false)
                        .takes_value(true)
                        .help("channel id"),
                ),
        ).get_matches();

    info!(
        "RPC Binding: {}:{}",
        SETTINGS.main.rpc_bind_ip, SETTINGS.main.rpc_bind_port
    );

    match app.subcommand() {
        ("init", Some(_)) => {
            let downloader = ytdl::YtDL::new()?;
            info!(
                "Downloader startup test success: {}",
                downloader.startup_test()
            );
        }
        ("play-audio", Some(sub_m)) => {
            info!("Audio play testing..");
            let instance = playback::Player::create_instance()?;
            let mut player = playback::Player::new(&instance)?;
            let path = get_path_for_existing_file(sub_m.value_of("file").unwrap()).unwrap();
            player.set_file(&path)?;
            player.play()?;

            debug!("File: {:?}", path);
            while !player.ended() {
                trace!("Position: {}", player.get_position());
                thread::sleep(Duration::from_millis(500));
            }
            info!("Finished");
        }
        ("test-url", Some(sub_m)) => {
            info!("Url play testing..");
            let instance = playback::Player::create_instance()?;
            {
                let mut player = playback::Player::new(&instance)?;
                let url = sub_m.value_of("url").unwrap();
                for i in 0..100 {
                    player.set_url(&url)?;
                    player.play()?;

                    debug!("url: {:?}", url);
                    while !player.ended() {
                        trace!("Position: {}", player.get_position());
                        thread::sleep(Duration::from_millis(250));
                        // play around with volume
                        player.set_volume((player.get_position() * 1000.0) as i32 % 100)?;
                    }
                    println!("playthough finished {}", i);
                }
                drop(player);
            }
            drop(instance);
            info!("finished, waiting..");
            thread::sleep(Duration::from_millis(5000));
            info!("Finished");
        }
        ("test-ts", Some(sub_m)) => {
            info!("Testing ts instance start");
            info!(
                "Folder: {} Exec: {}",
                SETTINGS.ts.dir, SETTINGS.ts.start_binary
            );
            let addr = sub_m.value_of("host").unwrap();
            let port = sub_m.value_of("port").unwrap().parse::<u16>().unwrap();
            let cid = sub_m
                .value_of("cid")
                .unwrap_or("-1")
                .parse::<i32>()
                .unwrap();

            let _instance = ts::TSInstance::spawn(
                0,
                addr,
                port,
                "",
                cid,
                "Test Bot Instance",
                &SETTINGS.main.rpc_bind_port,
            )?;

            info!("Started, starting RPC server..");

            //thread::sleep(Duration::from_millis(10000));

            check_runtime()?;
            daemon::start_runtime()?;
            info!("Test ended");
        }
        (_, _) => {
            warn!("No params, entering daemon mode");
            check_runtime()?;
            daemon::start_runtime()?;
        }
    }
    info!("Shutdown of yamba daemon");
    Ok(())
}

fn check_runtime() -> Fallible<()> {
    if let Err(e) = rpc::check_config() {
        error!("Invalid config for rpc daemon, aborting: {}", e);
        return Err(e);
    }

    if let Err(e) = api::check_config() {
        error!("Invalid config for api daemon, aborting: {}", e);
    }
    Ok(())
}

/// validate path input
fn validator_path(input: String) -> Result<(), String> {
    match get_path_for_existing_file(&input) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

/// Init log system
/// Creating a default log file if not existing
fn init_log() -> Fallible<()> {
    let log_path = std::env::current_dir()?.join(LOG_PATH);
    let mut log_dir = log_path.clone();
    log_dir.pop();
    DirBuilder::new().recursive(true).create(log_dir)?;

    if !metadata(&log_path).is_ok() {
        let config = include_str!("../default_log.yml");
        let mut file = File::create(&log_path)?;
        file.write_all(config.as_bytes())?;
        file.flush()?;
    }
    log4rs::init_file(log_path, Default::default())?;
    Ok(())
}

/// Get path for input if possible
fn get_path_for_existing_file(input: &str) -> Result<PathBuf, String> {
    let path_o = PathBuf::from(input);
    let path;
    if path_o.parent().is_some() && path_o.parent().unwrap().is_dir() {
        path = path_o;
    } else {
        let mut path_w = std::env::current_dir().unwrap();
        path_w.push(input);
        path = path_w;
    }

    if path.is_dir() {
        return Err(format!("Specified file is a directory {:?}", path));
    }

    if !path.exists() {
        return Err(format!("Specified file not existing {:?}", path));
    }

    Ok(path)
}

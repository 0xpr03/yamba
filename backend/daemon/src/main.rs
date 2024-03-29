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
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate clap;
extern crate log4rs;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate atomic;
extern crate chrono;
extern crate concurrent_hashmap;
extern crate config as config_rs;
extern crate erased_serde;
extern crate futures;
extern crate glib;
extern crate gstreamer as gst;
extern crate gstreamer_player as gst_player;
extern crate hashbrown;
extern crate libpulse_binding as pulse;
extern crate libpulse_glib_binding as pglib;
extern crate libpulse_sys as pulse_sys;
extern crate metrohash;
extern crate mpmc_scheduler;
extern crate owning_ref;
extern crate reqwest;
extern crate rusqlite;
extern crate serde;
extern crate serde_json;
extern crate serde_urlencoded;
extern crate sha2;
extern crate tokio;
extern crate tokio_signal;
extern crate tokio_threadpool;
#[macro_use]
extern crate tower_web;
extern crate http as http_r;
extern crate yamba_types;

use std::alloc::System;

#[derive(Fail, Debug)]
pub enum MainErr {
    #[fail(display = "License not accepted.")]
    LicenseUnaccepted,
}

#[global_allocator]
static GLOBAL: System = System;

mod api;
mod audio;
mod cache;
mod config;
mod daemon;
mod http;
mod playback;
mod ts;
mod ytdl;
mod ytdl_worker;

use clap::{App, Arg, SubCommand};
use failure::Fallible;
use futures::sync::mpsc;
use futures::Stream;
use tokio::runtime;

use std::fs::{metadata, DirBuilder, File};
use std::io::Write;
use std::path::PathBuf;

use playback::{PlayerEvent, PlayerEventType};

/// Main

const DEFAULT_CONFIG_NAME: &'static str = "00-config.toml";
const CONF_DIR: &'static str = "conf";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const LOG_PATH: &'static str = "conf/daemon_log.yml";

lazy_static! {
    static ref SETTINGS: config::ConfigRoot = {
        info!("Loading config..");
        config::init_settings().unwrap()
    };
    static ref USERAGENT: String = format!("YAMBA v{}", VERSION);
}

fn main() -> Fallible<()> {
    println!("Starting yamba daemon {}, switching to logger.", VERSION);

    init_log()?;

    info!("Startup");

    check_license_agreements()?;

    let app = App::new("YAMBA")
        .version(VERSION)
        .author(crate_authors!(",\n"))
        .about("yamba backend, VoIP music bot")
        .subcommand(SubCommand::with_name("init").about("Initialize database on first execution"))
        .subcommand(
            SubCommand::with_name("play-audio")
                .about("Test command to play audio, requires existing pulse device.")
                .arg(
                    Arg::with_name("file")
                        .short("f")
                        .required(true)
                        .validator(validator_path)
                        .takes_value(true)
                        .help("audio file"),
                ),
        )
        .subcommand(
            SubCommand::with_name("test-url")
                .about("Test command to play audio from url, requires existing pulse device.")
                .arg(
                    Arg::with_name("url")
                        .short("u")
                        .required(true)
                        .takes_value(true)
                        .help("media url"),
                ),
        )
        .get_matches();

    info!(
        "API Internal Binding: {}:{}",
        SETTINGS.main.api_internal_bind_ip, SETTINGS.main.api_internal_bind_port
    );
    info!(
        "API Public Binding: {}:{}",
        SETTINGS.main.api_bind_ip, SETTINGS.main.api_bind_port
    );
    info!(
        "TS RPC callback IP: {}:{}",
        SETTINGS.main.api_jsonrpc_ip, SETTINGS.main.api_jsonrpc_port
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
            info!("Audio playback testing..");
            gst::init()?;
            let (send, recv) = mpsc::channel::<PlayerEvent>(10);
            let mut player = playback::Player::new(send, -1, 0.1)?;
            let path = get_path_for_existing_file(sub_m.value_of("file").unwrap()).unwrap();
            player.set_uri(&path.to_string_lossy());
            player.play();

            debug!("File: {:?}", path);

            tokio::run(recv.for_each(move |event| {
                println!("Event: {:?}", event);
                Ok(())
            }));

            info!("Finished");
        }
        ("test-url", Some(sub_m)) => {
            use playback::PlaybackState;
            use std::sync::{Arc, Mutex};
            use std::thread;
            info!("Url play testing..");
            {
                gst::init()?;
                let glib_loop = glib::MainLoop::new(None, false);
                gstreamer::debug_set_active(true);
                gstreamer::debug_set_default_threshold(gstreamer::DebugLevel::Warning);
                gstreamer::debug_set_threshold_for_name("player", gstreamer::DebugLevel::Debug);
                let (mainloop, context) = audio::init().unwrap();

                let glib_loop_clone = glib_loop.clone();
                thread::spawn(move || {
                    let glib_loop = &glib_loop_clone;
                    glib_loop.run();
                });

                let sink = audio::NullSink::new(mainloop, context, "test1").unwrap();

                let (send, recv) = mpsc::channel::<PlayerEvent>(10);
                let (mut send_s, recv_s) = mpsc::channel::<()>(1);
                let player = Arc::new(Mutex::new(playback::Player::new(send, -1, 0.1)?));

                let url = sub_m.value_of("url").unwrap();

                let mut runtime = runtime::Runtime::new()?;
                {
                    debug!("url: {:?}", url);
                    runtime.spawn(recv.for_each(move |event| {
                        trace!("Event: {:?}", event);
                        match event.event_type {
                            PlayerEventType::PositionUpdated(time) => {
                                debug!("Position: {}", time);
                            }
                            PlayerEventType::EndOfStream
                            | PlayerEventType::StateChanged(PlaybackState::Stopped) => {
                                debug!("end of stream");
                                let _ = send_s.try_send(());
                            }
                            _ => {}
                        }
                        Ok(())
                    }));
                    {
                        // drop player lock, or we'll block the whole event queue
                        let player_l = player.lock().unwrap();
                        player_l.set_pulse_device(sink.get_sink_name()).unwrap();
                        player_l.set_uri(&url);
                    }
                    debug!("Starting playback");
                    runtime.block_on(recv_s.into_future()).unwrap();
                    glib_loop.quit();
                }
                println!("playthough finished");
            }
            info!("Finished");
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

/// Check runtime relevant config values
fn check_runtime() -> Fallible<()> {
    api::check_runtime()?;
    Ok(())
}

/// Check license agreement for 3rd party terms
fn check_license_agreements() -> Fallible<()> {
    let mut accepted = false;
    if let Ok(v) = std::env::var("LICENSE_AGREEMENT") {
        accepted = v == "accepted";
    }
    if std::path::Path::new(".yamba_license_accepted").exists() {
        accepted = true;
    }

    if !accepted {
        error!("License not accepted!");
        error!("To accept add a env value of LICENSE_AGREEMENT with accepted as value or create a file .yamba_license_accepted");
        return Err(MainErr::LicenseUnaccepted.into());
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
        let config = include_str!("../includes/default_log.yml");
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

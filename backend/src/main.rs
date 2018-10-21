/*
 *  This file is part of yamba.
 *
 *  Foobar is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  Foobar is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with Foobar.  If not, see <https://www.gnu.org/licenses/>.
 */
#![recursion_limit = "1024"]
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate vlc;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate slog;
extern crate slog_scope;
extern crate slog_stdlog;
extern crate slog_term;
extern crate slog_async;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate config;
extern crate glob;

mod playback;

use clap::{Arg,App,SubCommand};
use failure::Fallible;
use slog::Drain;
use config::{File,Config};
use glob::glob;

use std::fs::{OpenOptions,DirBuilder};
use std::thread;
use std::ffi::OsStr;
use std::time::Duration;
use std::path::PathBuf;
use std::sync::RwLock;

const DEFAULT_CONFIG_NAME: &'static str = "00-config.toml";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

lazy_static! {
	static ref SETTINGS: RwLock<Config> = RwLock::new(init_settings());
}

fn main() -> Fallible<()> {
    
    let mut log_path = PathBuf::from("log");
    DirBuilder::new()
        .recursive(true)
        .create(&log_path).unwrap();
    log_path.push("backend.log");
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .unwrap();

    let decorator = slog_term::PlainDecorator::new(file);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let _log = slog::Logger::root(drain, o!());
    
    let _guard = slog_scope::set_global_logger(_log);
    
    slog_stdlog::init().unwrap();
    
    info!("Startup");

    let app = App::new("Clantool")
                    .version(VERSION)
                    .author(crate_authors!(",\n"))
                    .about("yamba backend, VoIP music bot")
                    .subcommand(SubCommand::with_name("init")
                        .about("Initialize database on first execution"))
                    .subcommand(SubCommand::with_name("play-audio")
                        .about("Test command to play audio")
                        .arg(Arg::with_name("file")
                            .short("f")
                            .required(true)
                            .validator(validator_path)
                            .takes_value(true)
                            .help("audio file")))
                    .get_matches();
    
    match app.subcommand() {
        ("init", Some(sub_m)) => {
            
        },
        ("play-audio", Some(sub_m)) => {
            info!("Audio play testing..");
            let instance = playback::Player::create_instance()?;
            let mut player = playback::Player::new(&instance)?;
            let path = get_path_for_existing_file(sub_m.value_of("file").unwrap()).unwrap();
            player.set_file(&path)?;
            player.play()?;
            
            debug!("File: {:?}",path);
            while !player.ended() {
                trace!("Position: {}",player.get_position());
                thread::sleep(Duration::from_millis(500));
            }
            info!("Finished");
        },
        (c,_) => {
            warn!("Unknown command: {}",c);
        }
    }
    Ok(())
}

fn init_settings() -> Config {
    let mut settings = Config::default();
    settings.merge(File::with_name(&format!("conf/{}",DEFAULT_CONFIG_NAME))).unwrap();
    settings.merge(glob("conf/*")
                    .unwrap()
                    .map(|path| path.unwrap())
                    .filter(|path| path.file_name() != Some(OsStr::new(DEFAULT_CONFIG_NAME)))
                    .map(|path| File::from(path))
                    .collect::<Vec<_>>()).unwrap();
    //settings.set_default().unwrap();
    settings
}

/// validate path input
fn validator_path(input: String) -> Result<(),String> {
    match get_path_for_existing_file(&input) {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}

/// Get path for input if possible
fn get_path_for_existing_file(input: &str) -> Result<PathBuf,String> {
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
        return Err(format!("Specified file is a directory {:?}",path));
    }
    
    if !path.exists() {
        return Err(format!("Specified file not existing {:?}",path));
    }

    Ok(path)
}


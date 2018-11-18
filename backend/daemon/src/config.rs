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

use std::env::current_dir;
use std::ffi::OsStr;
use std::fs::{read_dir, DirBuilder, OpenOptions};
use std::io::{self, Write};

use config_rs::{Config, ConfigError as ConfigRSError, Environment, File as CFile, FileFormat};
use failure::Fallible;

use {CONF_DIR, DEFAULT_CONFIG_NAME};

#[derive(Fail, Debug)]
pub enum ConfigErr {
    #[fail(display = "Unable to open default config {}", _0)]
    DefaultConfigParseError(#[cause] ConfigRSError),
    #[fail(display = "Unable to open default config {}", _0)]
    DefaultConfigError(#[cause] io::Error),
    #[fail(display = "Unable to write default config {}", _0)]
    DefaultConfigWriteError(#[cause] io::Error),
    #[fail(display = "Can't retrieve config")]
    FolderError,
    #[fail(display = "Unable to open default config {}", _0)]
    FolderCreationError(#[cause] io::Error),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigRoot {
    pub main: ConfigMain,
    pub db: ConfigDB,
    pub ytdl: ConfigYtDL,
    pub ts: ConfigTS,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigTS {
    pub dir: String,
    pub start_binary: String,
    pub additional_args_binary: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigMain {
    pub user_agent: String,
    pub rpc_bind_port: u16,
    pub rpc_bind_ip: String,
    pub api_bind_port: u16,
    pub api_bind_ip: String,
    pub api_callback_port: u16,
    pub api_callback_ip: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigDB {
    pub host: String,
    pub port: i16,
    pub user: String,
    pub use_password: bool,
    pub password: String,
    pub db: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigYtDL {
    pub dir: String,
    pub update_intervall: u8,
    pub version_source: String,
    pub version_key: String,
    pub download_source: String,
    pub timeout_version: u8,
}

/// Init settings
pub fn init_settings() -> Fallible<ConfigRoot> {
    let settings = load_settings()?;
    let parsed = settings.try_into::<ConfigRoot>()?;
    Ok(parsed)
}

/// Load full settings
fn load_settings() -> Fallible<Config> {
    let mut settings = load_default()?;
    let config_folder = current_dir()?.join(CONF_DIR);
    let config_files: Vec<_> = read_dir(config_folder)?
        .filter_map(|x| x.ok())
        .filter(|x| match x.metadata() {
            Ok(metadata) => {
                metadata.is_file()
                    && x.path().file_name() != Some(OsStr::new(DEFAULT_CONFIG_NAME))
                    && x.path().extension() == Some(OsStr::new("toml"))
            }
            Err(e) => {
                warn!("can't handle {:?} during config loading: {}", x, e);
                false
            }
        }).map(|x| CFile::from(x.path()))
        .collect();
    debug!("config_files {:?}", config_files);
    settings.merge(config_files)?;
    settings.merge(Environment::with_prefix("yamba").separator("__"))?;
    trace!("{:?}", println!("{:?}", settings));
    Ok(settings)
}

/// Load default config file
fn load_default() -> Fallible<Config> {
    let mut settings = Config::default();
    let path_config = current_dir()?.join(CONF_DIR).join(DEFAULT_CONFIG_NAME);
    debug!("Config path: {:?}", path_config);
    let existing = path_config.exists();
    let mut config_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path_config)
        .map_err(|e| ConfigErr::DefaultConfigError(e))?;
    if !existing {
        info!("Config not existing, creating default config & dirs");
        DirBuilder::new()
            .recursive(true)
            .create(path_config.parent().ok_or(ConfigErr::FolderError)?)
            .map_err(|e| ConfigErr::FolderCreationError(e))?;
        let default_config = include_str!("../conf/00-config.toml");
        config_file
            .write_all(default_config.as_bytes())
            .map_err(|e| ConfigErr::DefaultConfigWriteError(e))?;
        config_file
            .flush()
            .map_err(|e| ConfigErr::DefaultConfigWriteError(e))?;
    } else {
        info!("Found default config");
    }
    drop(config_file);
    settings
        .merge(CFile::with_name(&path_config.to_string_lossy()))//(&format!("conf/{}", DEFAULT_CONFIG_NAME)))
        .map_err(|e| ConfigErr::DefaultConfigParseError(e))?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let settings = load_settings().unwrap();
        println!("{:?}", settings);
    }

    #[test]
    fn test_default() {
        let _settings = load_default().unwrap();
    }

    #[test]
    fn test_deserialization() {
        let settings = init_settings().unwrap();
        assert_eq!("ytdl", settings.ytdl.dir);
        assert_eq!(false, settings.db.use_password);
        println!("{:?}", settings);
    }
}

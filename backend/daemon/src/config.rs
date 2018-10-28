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
use std::fs::read_dir;

use config_rs::{Config, File};
use failure::Fallible;

use {CONF_DIR, DEFAULT_CONFIG_NAME};

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
    pub start_script: String,
    pub additional_args: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigMain {
    pub user_agent: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigDB {
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
        }).map(|x| File::from(x.path()))
        .collect();
    debug!("config_files {:?}", config_files);
    settings.merge(config_files)?;
    Ok(settings)
}

/// Load default config file
fn load_default() -> Fallible<Config> {
    let mut settings = Config::default();
    settings.merge(File::with_name(&format!("conf/{}", DEFAULT_CONFIG_NAME)))?;
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

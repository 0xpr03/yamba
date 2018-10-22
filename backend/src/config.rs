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
use failure::Fallible;
use config_rs::{File,Config};
use glob::glob;
use std::ffi::OsStr;

use ::DEFAULT_CONFIG_NAME;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigRoot {
    pub main: ConfigMain,
    pub db: ConfigDB,
    pub ytdl: ConfigYtDL
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigMain {
    pub user_agent: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigDB {
    pub port: i16,
    pub user: String,
    pub use_password: bool,
    pub password: String,
    pub db: String
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

pub fn init_settings() -> Fallible<ConfigRoot> {
    let settings = load_settings()?;
    let parsed = settings.try_into::<ConfigRoot>()?;
    Ok(parsed)
}

fn load_settings() -> Fallible<Config> {
    let mut settings = load_default()?;
    settings.merge(glob("conf/*")
                    .unwrap()
                    .map(|path| path.unwrap())
                    .filter(|path| path.file_name() != Some(OsStr::new(DEFAULT_CONFIG_NAME)))
                    .map(|path| File::from(path))
                    .collect::<Vec<_>>())?;
    //settings.set_default().unwrap();
    Ok(settings)
}

fn load_default() -> Fallible<Config> {
    let mut settings = Config::default();
    settings.merge(File::with_name(&format!("conf/{}",DEFAULT_CONFIG_NAME)))?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load() {
        let settings = load_settings().unwrap();
        println!("{:?}",settings);
    }

    #[test]
    fn test_default() {
        let _settings = load_default().unwrap();
    }

    #[test]
    fn test_deserialization() {
        let settings = init_settings().unwrap();
        assert_eq!("ytdl",settings.ytdl.dir);
        assert_eq!(false,settings.db.use_password);
        println!("{:?}",settings);
    }
}
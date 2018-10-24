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

use std::process::{Command, Stdio, Child};
use std::os::unix::fs::PermissionsExt;
use std::fs::{File,rename,remove_file,DirBuilder,set_permissions};
use std::io::{Read,ErrorKind};
use std::sync::{Arc,RwLock};
use std::path::{Path,PathBuf};
use std::env::current_dir;
use std::{io};

use failure::{Fallible,ResultExt};
use json::{self,JsonValue};
use sha2::{Sha256, Digest};

use http;
use ::SETTINGS;

const UPDATE_VERSION_KEY: &'static str = "latest"; // key in the json map
const VERSIONS_KEY: &'static str = "versions"; // key for versions sub group
const VERSION_BIN_KEY: &'static str = "bin"; // key for versions sub group
const VERSION_SHA_INDEX: usize = 1;
const YTDL_NAME: &'static str = "youtube-dl"; // name of the python program file

lazy_static! {
    static ref LOCK: Arc<RwLock<()>> = Arc::new(RwLock::new(()));
}

#[derive(Fail, Debug)]
pub enum YtDLErr {
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
    #[fail(display = "Json invalid input {}",_0)]
    JsonError(&'static str),
    #[fail(display = "Invalid response {}",_0)]
    ResponseError(String),
    #[fail(display = "Incorrect hash for {}", _0)]
    InvalidHash(String)
}

/// Version struct for retrieval of version & sha on update check
pub struct Version {
    version: String,
    sha256: String,
}

pub struct YtDL {
    // base dir from which ytdl is called
    base: PathBuf
}

impl YtDL {
    /// Creates a new YtDL struct, expects SETTINGS
    pub fn new() -> Fallible<YtDL> {
        let start_path = PathBuf::from(&SETTINGS.ytdl.dir);
        let path;
        if start_path.parent().is_some() && start_path.parent().unwrap().is_dir() {
            path = start_path;
        } else {
            let path_w = current_dir().unwrap();
            path = path_w.join(&SETTINGS.ytdl.dir);
        }
        DirBuilder::new().recursive(true).create(&path)?;
        Ok(YtDL {
            base: path
        })
    }

    /// Get url info
    fn get_url_info(&self, url: &str) -> Fallible<JsonValue> {
        let _guard = LOCK.read().unwrap();
        let result = self.cmd_base().arg("-j").arg(url).output()?;
        Ok(json::parse(&String::from_utf8_lossy(&result.stdout))?)
    }

    /// get executable path
    fn get_exec_path(&self) -> PathBuf {
        self.base.join(YTDL_NAME)
    }
    
    /// Run a self-test checking for either yt-dl binaries or update failure
    /// depending on the config
    /// Returns true on success
    pub fn startup_test(&self) -> bool {
        info!("Testing yt-dl settings");
        match self.update_downloader() {
            Ok(_) => true,
            Err(e) => {error!("Failed updating yt-dl {} {} trace:{}",e,e.as_fail(),e.backtrace()); false}
        }
    }

    /// Retrieve latest version
    pub fn latest_version() -> Fallible<Version> {
        let result = http::get_text(&SETTINGS.ytdl.version_source,http::HeaderType::Ajax)?;
        let mut parsed = json::parse(&result)?;
        let version = match &mut parsed[UPDATE_VERSION_KEY] {
            &mut JsonValue::Null => return Err(YtDLErr::JsonError("Version key not found!").into()),
            r_version => r_version.take_string().ok_or(YtDLErr::JsonError("Version value is not a str!"))?,
        };

        let sha256 = match &mut parsed[VERSIONS_KEY][&version][VERSION_BIN_KEY][VERSION_SHA_INDEX] {
            &mut JsonValue::Null => return Err(YtDLErr::JsonError("SHA256 key not found!").into()),
            r_sha256 => {
                debug!("sha: {:?}",r_sha256);
                r_sha256.take_string().ok_or(YtDLErr::JsonError("SHA256 value is not an str!"))?
            },
        };
        
        Ok(Version {
            version,
            sha256
        })
    }

    /// create command base
    fn cmd_base(&self) -> Command {
        let mut cmd = Command::new(self.get_exec_path());
        cmd.current_dir(&self.base);
        cmd
    }

    /// Get current version
    pub fn current_version(&self) -> Fallible<String> {
        let _guard = LOCK.read().unwrap();
        let result = self.cmd_base()
        .arg("--version")
        .output()?;//.context("Could not run yt-dl")?;
        if result.status.success() {
            Ok(String::from_utf8_lossy(&result.stdout).trim().to_string())
        } else {
            Err(YtDLErr::ResponseError("Process errored".into()).into())
        }
    }
    
    /// Update yt-dl, blocks untill complection.
    /// Blocks new jobs untill finish & waits till current jobs are completed.
    pub fn update_downloader(&self) -> Fallible<()> {
        let latest = YtDL::latest_version()?;
        // if the guard is poinsoned, we can't do anything anymore
        let current_file = self.get_exec_path();
        
        let mut force_download = true;

        if current_file.exists() {
            let backup_file = self.base.join("ytdl_backup");
            self.set_permissions().context("Unable to set permissions")?;
            let current_version = self.current_version()?;
            debug!("Version current: {} latest: {}",current_version,latest.version);
            if latest.version != current_version {
                let _guard = LOCK.write().unwrap();
                rename(&current_file,&backup_file)?;
                match self.download_latest(&current_file, &latest.sha256) {
                    Ok(_) => {
                        remove_file(backup_file)?;
                        force_download = false;
                    },
                    Err(e) => {
                        // use backup
                        rename(&backup_file,&current_file)?;
                        return Err(e);
                    }
                }
                drop(_guard);
            } else { // correct version, correct hash?
                debug!("ytdl existing, checking hash");
                if self.check_sha256(&latest.sha256)? {
                    force_download = false;
                } else {
                    warn!("Forcing download, current hash mismatch!");
                }
            }
        }

        if force_download {
            if current_file.exists() {
                remove_file(&current_file)?;
            }
            info!("No ytdl installed, downloading..");
            let _guard = LOCK.write().unwrap();
            self.download_latest(&current_file, &latest.sha256)?;
            drop(_guard);
        }

        self.set_permissions().context("Unable to set permissions")?;
        
        Ok(())
    }

    /// Set permissions for executable
    fn set_permissions(&self) -> Fallible<()> {
        debug!("permission application: {:?}",self.get_exec_path());
        let metadata = self.get_exec_path().metadata()?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o764); // rwxrw-r--
        set_permissions(self.get_exec_path(),permissions)?;
        Ok(())
    }

    /// Check sha256 of current exec
    /// expected is a hexadecimal representation of the expected hash
    fn check_sha256(&self, expected: &str) -> Fallible<bool> {
        let mut file = File::open(self.get_exec_path())?;
        let mut sha2 = Sha256::default();
        let mut buf = [0; 1024];
        loop {
            let len = match file.read(&mut buf) {
                Ok(0) => break,
                Ok(len) => len,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            };
            sha2.input(&buf[..len]);
        }
        let result = format!("{:X}",sha2.result());
        let result = result.to_lowercase();
        let is_matching = result == expected;
        if !is_matching {
            debug!("SHA Expected: {} Result: {}",expected,result);
        }
        Ok(is_matching)
    }

    /// Inner update method, downloads latest version to target
    /// Doesn't perform any lock checks!
    fn download_latest(&self, target: &Path, hash: &str) -> Fallible<()> {
        http::get_file(&SETTINGS.ytdl.download_source,&target)?;
        if self.check_sha256(hash)? {
            Ok(())
        } else {
            Err(YtDLErr::InvalidHash(target.to_string_lossy().into()).into())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    /// Test header creation
    #[test]
    fn test_init() {
        let downloader = YtDL::new().unwrap();
        downloader.startup_test();
    }
}
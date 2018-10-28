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

use std::process::{Command, Stdio, Child};
use std::os::unix::fs::PermissionsExt;
use std::fs::{File,rename,remove_file,DirBuilder,set_permissions};
use std::io::{self,BufRead,BufReader,Read,ErrorKind};
use std::sync::{Arc,RwLock};

use std::path::{Path,PathBuf};
use std::env::current_dir;
use std::{thread};

use failure::{Fallible,ResultExt};
use serde_json::{self};
use serde_json::value::Value as JsonValue;
use sha2::{Sha256, Digest};

use http;
use ::SETTINGS;

const UPDATE_VERSION_KEY: &'static str = "latest"; // key in the json map
const VERSIONS_KEY: &'static str = "versions"; // key for versions sub group
const VERSION_BIN_KEY: &'static str = "bin"; // key for versions sub group
const VERSION_SHA_INDEX: usize = 1;
const YTDL_NAME: &'static str = "youtube-dl"; // name of the python program file

const KEY_LENGTH: &'static str = "duration";
const KEY_TITLE: &'static str = "fulltitle";
const KEY_FOMATS: &'static str = "formats"; // list of available formats
const KEY_FORMAT: &'static str = "format"; // format name of entry
const KEY_FORMAT_URL: &'static str = "url";
const KEY_FORMAT_CODEC_AUDIO: &'static str = "acodec";
const KEY_FORMAT_CODEC_VIDEO: &'static str = "vcodec";
const KEY_ID: &'static str = "id";
const KEY_DECODER: &'static str = "extractor_key";

lazy_static! {
    static ref LOCK: Arc<RwLock<()>> = Arc::new(RwLock::new(()));
}

#[derive(Debug,Deserialize)]
pub struct Track {
    pub title: String,
    pub id: String,
    pub extractor: String,
    pub duration: Option<f64>,
    pub formats: Vec<Format>,
    pub protocol: Option<String>,
}

#[derive(Debug,Deserialize)]
pub struct Format {
    pub filesize: Option<i64>,
    pub format: String,
    pub abr: Option<i64>, // audio bit rate
    pub format_id: String,
    pub url: String,
    pub protocol: Option<String>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>, // audio codec
    pub http_headers: HttpHeaders
}

#[derive(Debug,Deserialize)]
pub struct HttpHeaders {
    #[serde(rename = "Accept-Charset")]
    pub accept_charset: String,
    #[serde(rename = "Accept-Language")]
    pub accept_language: String,
    #[serde(rename = "Accept-Encoding")]
    pub accept_encoding: String,
    #[serde(rename = "Accept")]
    pub accept: String,
    #[serde(rename = "User-Agent")]
    pub user_agent: String
}

#[derive(Fail, Debug)]
pub enum YtDLErr {
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
    #[fail(display = "Pipe error processing ytdl output {}", _0)]
    PipeError(String),
    #[fail(display = "Json invalid input {}",_0)]
    JsonError(&'static str),
    #[fail(display = "Invalid response {}",_0)]
    ResponseError(String),
    #[fail(display = "Incorrect hash for {}", _0)]
    InvalidHash(String),
    #[fail(display = "Json parsing error {}", _0)]
    IncorrectJson(#[cause] serde_json::Error),
    #[fail(display = "Thread panicked at {}", _0)]
    ThreadPanic(String)
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
    pub fn get_url_info(&self, url: &str) -> Fallible<Track> {
        let _guard = LOCK.read().unwrap();
        let result = self.cmd_base().arg("-j").arg("--no-playlist").arg(url).output()?;
        let stderr = String::from_utf8_lossy(&result.stderr);
        let stdout = String::from_utf8_lossy(&result.stdout);
        debug!("stderr: {}",stderr);
        println!("stdout: {}",stdout);
        Ok(serde_json::from_str(&stdout)?)
    }

    /// Get playlist info
    pub fn get_playlist_info(&self, url: &str) -> Fallible<Vec<Track>> {
        let _guard = LOCK.read().unwrap();
        let mut child = self.cmd_base().arg("-j").arg("--yes-playlist").arg(url)
            .stdin(Stdio::null()).stderr(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
        let stdout = BufReader::new(child.stdout.take().ok_or(YtDLErr::PipeError("Couldn't get stdout".into()))?);
        let mut stderr_reader = child.stderr.take().ok_or(YtDLErr::PipeError("Couldn't get stderr".into()))?;
        let stderr_worker_handle = thread::spawn(move || {
            let mut buffer = String::new();
            stderr_reader.read_to_string(&mut buffer)?;
            Ok(buffer)
        });

        let tracks = stdout.lines().map(|res| {
            Ok(serde_json::from_str(&(res?))?)
            }).collect::<Fallible<Vec<Track>>>()?;

        match stderr_worker_handle.join() {
            Ok(Ok(v)) => if v.len() > 0 {
                return Err(YtDLErr::ResponseError(format!("stderr: {}",v)).into())
            },
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(YtDLErr::ThreadPanic(format!("stderr worker {:?}",e)).into())
        }

        Ok(tracks)
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
        let mut parsed: JsonValue = serde_json::from_str(&result)?;
        let version: String = match parsed[UPDATE_VERSION_KEY].take() {
            JsonValue::Null => return Err(YtDLErr::JsonError("Version key not found!").into()),
            JsonValue::String(v) => v,//.ok_or(YtDLErr::JsonError("Version value is not a str!"))?,
            _ => return Err(YtDLErr::JsonError("Version key is not of correct type!").into())
        };

        let sha256: String = match parsed[VERSIONS_KEY][&version][VERSION_BIN_KEY][VERSION_SHA_INDEX].take() {
            JsonValue::Null => return Err(YtDLErr::JsonError("SHA256 key not found!").into()),
            JsonValue::String(r_sha256) => {
                debug!("sha: {:?}",r_sha256);
                r_sha256
            },
            _ => return Err(YtDLErr::JsonError("Sha256 is not of correct type!").into())
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

    lazy_static! {
        // simplify downloader, perform startup_test just once, this also tests it on the fly
        static ref DOWNLOADER: Arc<YtDL> = {
            let downloader = YtDL::new().unwrap();
            //assert!(downloader.startup_test());
            Arc::new(downloader)
        };
    }
    
    /// Test header creation
    #[test]
    fn test_yt_info() {
        let output = DOWNLOADER.get_url_info("https://www.youtube.com/watch?v=Es44QTJmuZ0").unwrap();
        assert_eq!(Some(259.0),output.duration);
        assert_eq!("HD SMPTE Color Bars & Tones 1920x1080 Test Pattern Jazz",output.title);
        assert_eq!(Some("https".into()),output.protocol);
    }

    #[test]
    fn test_stream_info() {
        let output = DOWNLOADER.get_url_info("https://www.youtube.com/watch?v=8XjDmVzqVUc").expect("can't get yt stream");
        assert_eq!(Some(0.0), output.duration, "failed for yt stream duration");
        assert_eq!(Some("m3u8".into()),output.protocol);

        let output = DOWNLOADER.get_url_info("http://yp.shoutcast.com/sbin/tunein-station.m3u?id=1796249").expect("can't get sc stream");
        assert_eq!(None, output.duration, "failed for shoutcast stream duration");
        assert_eq!(Some("m3u8".into()),output.protocol);
    }

    #[test]
    fn test_soundcloud_info() {
        let output = DOWNLOADER.get_url_info("https://soundcloud.com/djsusumu/alan-walker-faded-susumu-melbourne-bounce-edit").unwrap();
        assert_eq!(Some(144.0),output.duration);
        assert_eq!(Some("https".into()),output.protocol);
    }

    #[test]
    fn test_yt_playlist_info() {
        let output = DOWNLOADER.get_playlist_info("https://www.youtube.com/watch?v=kYdrd4Kspxg&list=PLfU2RMxoOiSCH8R5pzOtGiq2cn5vJPjP6&index=2&t=0s").unwrap();
        println!("{:?}",output);
        assert_eq!(8,output.len());
    }
}
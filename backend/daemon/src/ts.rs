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

use failure::Fallible;
use serde_urlencoded;

use std::env;
use std::fs::{remove_dir_all, DirBuilder};
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;
use std::vec::Vec;

use std::env::current_dir;

use rusqlite::{self, Connection};

use instance::ID;
use yamba_types::models::TSSettings;
use SETTINGS;

/// TS Instance

const TS_ENV_CALLBACK_INTERNAL: &'static str = "CALLBACK_YAMBA_INTERNAL";
const TS_ENV_ID: &'static str = "ID_YAMBA";
const TS_SETTINGS_FILE: &'static str = "settings.db";
const TS_PLUGINS_DIR: &'static str = "plugins";

#[derive(Fail, Debug)]
pub enum TSInstanceErr {
    #[fail(display = "Database Error on configuring instance {}", _0)]
    Database(#[cause] rusqlite::Error),
    #[fail(display = "TS Instance spawn error {}", _0)]
    SpawnError(#[cause] io::Error),
    #[fail(display = "TS Instance config creation error for IO {}", _0)]
    ConfigCreationIOError(#[cause] io::Error),
    #[fail(display = "TS Instance config creation error {}", _0)]
    ConfigCreationError(String),
    #[fail(display = "Couldn't kill instance, timeout")]
    InstanceExitError,
}

impl Drop for TSInstance {
    fn drop(&mut self) {
        match self.is_running() {
            Ok(true) | Err(_) => {
                // ignore error, otherwise run only if alive
                if let Err(e) = TSInstance::kill_by_ppid(&self.process.id()) {
                    warn!("Couldn't kill instance by ppid: {}", e);
                }

                match self.kill() {
                    Ok(()) => (),
                    Err(e) => warn!("Couldn't kill instance on cleanup {}", e),
                }
            }
            _ => (),
        }
    }
}

/// TS Instance, kills itself on drop
#[derive(Debug)]
pub struct TSInstance {
    process: Child,
}

impl TSInstance {
    /// Create a new instance controller
    /// Created from TSSettings model
    /// rpc port is for callbacks used by the yamba plugin
    pub fn spawn(
        settings: &TSSettings,
        id: &ID,
        callback_host: &str,
        callback_port: &u16,
    ) -> Fallible<TSInstance> {
        let mut params = Vec::new();
        if let Some(v) = settings.port {
            params.push(("port".to_owned(), v.to_string()));
        }

        if let Some(v) = settings.cid {
            params.push(("cid".to_owned(), v.to_string()));
        }

        if let Some(ref v) = settings.password {
            params.push(("password".to_owned(), v.to_string()));
        }

        params.push(("nickname".to_owned(), settings.name.to_string()));

        let ts_url = serde_urlencoded::to_string(params)?;
        let path_binary = PathBuf::from(&SETTINGS.ts.dir);
        let path_binary = path_binary.join(&SETTINGS.ts.start_binary);
        let library_path = format!(
            ".:{}",
            env::var("LD_LIBRARY_PATH").unwrap_or("".to_string())
        );

        let path_config = current_dir()?.join("ts").join(format!("{}", id));

        if path_config.exists() && SETTINGS.ts.clear_config {
            remove_dir_all(&path_config).map_err(|e| TSInstanceErr::ConfigCreationIOError(e))?;
        }

        DirBuilder::new()
            .recursive(true)
            .create(&path_config)
            .map_err(|e| TSInstanceErr::ConfigCreationIOError(e))?;
        debug!("TS Instance config path: {:?}", path_config);

        let mut cmd = Command::new("xvfb-run");
        cmd.current_dir(&SETTINGS.ts.dir)
            .env("QT_PLUGIN_PATH", &SETTINGS.ts.dir)
            .env("QTDIR", &SETTINGS.ts.dir)
            .env("LD_LIBRARY_PATH", library_path)
            .env("KDEDIR", "")
            .env("KDEDIRS", "")
            .env("TS3_CONFIG_DIR", path_config.to_string_lossy().into_owned())
            .env(TS_ENV_ID, id.to_string())
            .env(
                TS_ENV_CALLBACK_INTERNAL,
                format!("{}:{}", callback_host, callback_port),
            )
            .args(&["--auto-servernum", "--server-args=-screen 0 640x480x24:32"])
            .arg(path_binary.to_string_lossy().to_mut())
            .args(&SETTINGS.ts.additional_args_binary)
            .arg("-nosingleinstance")
            .arg(format!("ts3server://{}?{}", settings.host, ts_url));
        trace!("TS Workdir: {}", &SETTINGS.ts.dir);
        trace!("CMD: {:?}", cmd);

        let path_db = path_config.join(TS_SETTINGS_FILE);
        debug!("TS Instance config path: {:?}", path_config);

        if !path_db.exists() {
            info!("Missing instance settings, creating..");
            let mut child = cmd.spawn().map_err(|e| TSInstanceErr::SpawnError(e))?;
            thread::sleep(Duration::from_millis(5000));
            TSInstance::kill_by_ppid(&child.id())?;
            child.kill()?; // evaluate
            TSInstance::wait_for_child_timeout(1000, &mut child)?;
            if !path_config.exists() {
                warn!("Unable to create configuration, no db existing!");
                return Err(TSInstanceErr::ConfigCreationError(
                    "No settings db, creation failed!".into(),
                )
                .into());
            }
            info!("Configuring..");
            TSInstance::configure_settings(&path_db, &path_config)?;
        }

        Ok(TSInstance {
            process: cmd.spawn().map_err(|e| TSInstanceErr::SpawnError(e))?,
        })
    }

    /// Kill childs of xvfb
    fn kill_by_ppid(ppid: &u32) -> Fallible<()> {
        let output = Command::new("pkill")
            .arg("-P")
            .arg(ppid.to_string())
            .output()?;
        trace!(
            "pkill status: {} stderr: {:?}",
            output.status,
            output.stderr
        );
        Ok(())
    }

    fn wait_for_child_timeout(sleep_ms: i32, child: &mut Child) -> Fallible<()> {
        let mut returned = false;
        let mut waited = 0;
        while waited < sleep_ms || !returned {
            if child.try_wait()?.is_some() {
                returned = true;
                break;
            }
            thread::sleep(Duration::from_millis(30));
            waited += 30;
        }

        if returned {
            Ok(())
        } else {
            Err(TSInstanceErr::InstanceExitError.into())
        }
    }

    fn configure_settings(path_db: &Path, path_config: &Path) -> Fallible<()> {
        trace!("Configuring ts instance under {:?}", path_db);
        let connection = Connection::open(path_db)?;
        {
            let values = ["en", "true", "4"];
            let keys = [
                "LastShownLicenseLang",
                "SyncOverviewShown",
                "LastShownLicense",
            ];
            TSInstance::insert_or_replace(&connection, &keys, &values, "General")?;

            let keys = [
                "DefaultCaptureProfile",
                "DefaultPlaybackProfile",
                "Capture/musik",
                "Capture/musik/PreProcessing",
                "Playback/mute",
            ];
            let values = [
                "musik",
                "mute",
                include_str!("../resources/capture.txt"),
                include_str!("../resources/capture_preprocessing.txt"),
                include_str!("../resources/playback.txt"),
            ];
            TSInstance::insert_or_replace(&connection, &keys, &values, "Profiles")?;

            // let values = ["CloseActiveServerTab"];
            // let keys = ["false"];
            // TSInstance::insert_or_replace(&connection, &keys, &values, "Ask")?;

            connection
                .close()
                .map_err(|(_, e)| TSInstanceErr::Database(e))?;
        }
        {
            let path_addons = path_config.join(TS_PLUGINS_DIR);
            DirBuilder::new()
                .recursive(true)
                .create(&path_addons)
                .map_err(|e| TSInstanceErr::ConfigCreationIOError(e))?;
            debug!(
                "Linking {} to {}",
                &SETTINGS.ts.plugin_path,
                path_addons.join("yamba_plugin.so").to_string_lossy()
            );
            symlink(
                &SETTINGS.ts.plugin_path,
                path_addons.join("yamba_plugin.so"),
            )
            .map_err(|e| TSInstanceErr::ConfigCreationIOError(e))?;
        }

        Ok(())
    }

    /// Insert or replace multiple values for sqlite connection
    fn insert_or_replace(
        connection: &Connection,
        keys: &[&str],
        values: &[&str],
        table: &'static str,
    ) -> Fallible<()> {
        let mut stmt = connection
                .prepare(&format!("INSERT OR REPLACE INTO `{}` (`timestamp`,`key`,`value`) SELECT MAX(`timestamp`),?,? FROM `{}`",table,table))?;
        for (&k, &v) in keys.iter().zip(values.iter()) {
            debug!("Inserting {}{}", k, v);
            stmt.execute(&[k, v])?;
        }
        Ok(())
    }

    pub fn kill(&mut self) -> Fallible<()> {
        Ok(self.process.kill()?)
    }

    pub fn is_running(&mut self) -> Fallible<bool> {
        Ok(self.process.try_wait()?.is_none())
    }
}

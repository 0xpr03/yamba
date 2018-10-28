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

use failure::{Fallible,ResultExt};
use serde_urlencoded;

use std::process::{Child,Command};
use std::io;

use ::SETTINGS;

const TS_ENV_CALLBACK: &'static str = "CALLBACK_YAMBA";
const TS_ENV_ID: &'static str = "ID_YAMBA";

#[derive(Fail, Debug)]
pub enum TSInstanceError {
    #[fail(display = "IO Error {}", _0)]
    Io(#[cause] io::Error),
    #[fail(display = "Instance spawn error {}",_0)]
    SpawnError(#[cause] io::Error),
    #[fail(display = "Pipe error processing ytdl output {}", _0)]
    PipeError(String),
    #[fail(display = "Thread panicked at {}", _0)]
    ThreadPanic(String),

}

pub struct TSInstance{
    id: i32,
    process: Child
}

impl TSInstance {
    /// Create a new instance controller
    /// ID is used on callbacks
    pub fn spawn(id: i32, address: &str, port: u16, password: &str, cid: i32, name: &str) -> Fallible<TSInstance> {
        let ts_url = serde_urlencoded::to_string(vec![
        ("port".to_owned(), port.to_string()),
        ("nickname".to_owned(), name.to_string()),
        ("password".to_owned(), password.to_string()),
        ("cid".to_owned(), cid.to_string())
        ])?;
        Ok(TSInstance{
            id,
            process: Command::new(&SETTINGS.ts.start_script)
            .current_dir(&SETTINGS.ts.dir)
            .env("QT_PLUGIN_PATH",&SETTINGS.ts.dir)
            .env("QTDIR",&SETTINGS.ts.dir)
            .env("LD_LIBRARY_PATH",&SETTINGS.ts.dir)
            .env(TS_ENV_ID,id.to_string())
            .env(TS_ENV_CALLBACK,"")
            .args(&SETTINGS.ts.additional_args)
            .arg("-nosingleinstance")
            .arg(format!("ts3server://{}?{}",address,ts_url))
            .spawn().map_err(|e|TSInstanceError::SpawnError(e))?
        })
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn kill(&mut self) -> Fallible<()> {
        Ok(self.process.kill()?)
    }

}
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

use metrohash::MetroHash128;
use mysql::chrono::prelude::NaiveDateTime;
use mysql::error::Error as MySqlError;
use mysql::{from_row_opt, Opts, OptsBuilder, Pool};

use ytdl::Track;

use std::hash::Hash;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::vec::Vec;

use SETTINGS;

use models::TSSettings;

/// DB stuff

#[derive(Fail, Debug)]
pub enum DatabaseErr {
    #[fail(display = "Couldn't find data for ID {}", _0)]
    InstanceNotFoundErr(i32),
    #[fail(display = "DB error: {}", _0)]
    DBError(#[cause] MySqlError),
}

/// Init db connection pool
pub fn init_pool() -> Fallible<Pool> {
    let mut builder = OptsBuilder::new();
    builder
        .ip_or_hostname(Some(SETTINGS.db.host.clone()))
        .db_name(Some(SETTINGS.db.db.clone()))
        .user(Some(SETTINGS.db.user.clone()))
        .pass(Some(SETTINGS.db.password.clone()))
        .tcp_port(SETTINGS.db.port);
    let opts: Opts = builder.into();
    Ok(Pool::new(opts)?)
}

/// Init db connection pool with retry timeout
pub fn init_pool_timeout() -> Fallible<Pool> {
    let start = Instant::now();

    loop {
        match init_pool() {
            Err(e) => {
                if start.elapsed().as_secs() > SETTINGS.db.retry_time as u64 {
                    error!("Timeout during db connection!");
                    return Err(e);
                } else {
                    info!("Retrying DB connect");
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            }
            Ok(v) => return Ok(v),
        }
    }
}

/// Get instance ids with enabled autostart
pub fn get_autostart_instance_ids(pool: &Pool) -> Fallible<Vec<i32>> {
    let instances: Fallible<Vec<i32>> = pool
        .prep_exec("SELECT id from `instances` WHERE `autostart` = ?", (true,))?
        .map(|result| {
            let (id,) = from_row_opt::<(i32,)>(result?)?;
            Ok(id)
        }).collect();
    instances
}

/// Load data for specified instance ID
pub fn load_instance_data(pool: &Pool, id: &i32) -> Fallible<TSSettings> {
    let mut result = pool.prep_exec(
        "SELECT id, host,port,identity,cid,name,password,autostart from `instances`",
        (&id,),
    )?;
    let row = result
        .next()
        .ok_or(DatabaseErr::InstanceNotFoundErr(id.clone()))?;

    let (id, host, port, identity, cid, name, password, autostart) = from_row_opt(row?)?;
    Ok(TSSettings {
        id,
        host,
        port,
        identity,
        cid,
        name,
        password,
        autostart,
    })
}

/// Save a set of tracks into the DB and return their IDs
pub fn insert_tracks(tracks: &[Track], pool: &Pool) -> Fallible<Vec<String>> {
    let mut transaction = pool.start_transaction(false, None, None)?;

    let ids = tracks
        .iter()
        .map(|track| {
            let id = calculate_id(track);
            transaction.prep_exec(
                "INSERT INTO `titles` 
            (`id`,`name`,`source`,`downloaded`, `artist`, `length`) 
            VALUES (?,?,?,?,?,?)
            ON DUPLICATE KEY
            UPDATE name=name, length=length",
                (
                    &id,
                    &track.title,
                    &track.webpage_url,
                    0,
                    &track.artist,
                    track.duration,
                ),
            )?;
            Ok(id)
        }).collect::<Result<Vec<String>, MySqlError>>()?;

    transaction.commit()?;

    Ok(ids)
}

/// Create ID for track
fn calculate_id(track: &Track) -> String {
    let mut hasher = MetroHash128::default();
    track.hash(&mut hasher);
    let (h1, h2) = hasher.finish128();
    format!("{:x}{:x}", h1, h2)
}

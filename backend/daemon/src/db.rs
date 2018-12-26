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
use mysql::error::Error as MySqlError;
use mysql::{from_row_opt, Opts, OptsBuilder, Pool};

use ytdl::Track;

use std::hash::Hash;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::vec::Vec;

use SETTINGS;

use models::*;

const TS_TYPE: &'static str = "teamspeak_instances";

/// DB stuff

#[derive(Fail, Debug)]
pub enum DatabaseErr {
    #[fail(display = "Couldn't find instance entry for ID {}", _0)]
    InstanceNotFoundErr(i32),
    #[fail(display = "Couldn't find instance data on {} for ID {}", _1, _0)]
    InstanceDataNotFoundErr(i32, &'static str),
    #[fail(display = "Instance type {} is unknown", _0)]
    InstanceTypeUnknown(String),
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
        })
        .collect();
    instances
}

/// Load data for specified instance ID
pub fn load_instance_data(pool: &Pool, id: &i32) -> Fallible<DBInstanceType> {
    let mut type_result = pool.prep_exec("select `type` from `instances` WHERE id = ?", (&id,))?;
    let row = type_result
        .next()
        .ok_or(DatabaseErr::InstanceNotFoundErr(id.clone()))?;
    let inst_type: String = from_row_opt(row?)?;
    match inst_type.as_str() {
        TS_TYPE => {
            let mut result = pool.prep_exec(
                format!(
                    "SELECT dat.`instance_id`,`host`,`port`,`identity`,`cid`,`name`,`password` FROM `{}` dat 
                JOIN `instances` inst ON inst.`id` = dat.`instance_id` WHERE dat.`instance_id` = ?",
                    TS_TYPE
                ),
                (&id,),
            )?;
            let row = result
                .next()
                .ok_or(DatabaseErr::InstanceDataNotFoundErr(id.clone(), TS_TYPE))?;

            // TODO: fix autostart parsing
            let (id, host, port, identity, cid, name, password) = //: //(_, _, _, _, _, _, _, u8) =
                from_row_opt(row?)?;
            Ok(DBInstanceType::TS(TSSettings {
                id,
                host,
                port,
                identity,
                cid,
                name,
                password,
                autostart: true,
            }))
        }
        _ => Err(DatabaseErr::InstanceTypeUnknown(inst_type).into()),
    }
}

/// Upsert instance for testing purpose
pub fn upsert_ts_instance(settings: &TSSettings, pool: &Pool) -> Fallible<()> {
    pool.prep_exec(
        "INSERT INTO `instances` (`id`,`name`,`type`,`autostart`) VALUES (?,?,?,?)
        ON DUPLICATE KEY UPDATE `name`=VALUES(`name`), `type`=VALUES(`type`), `autostart`=VALUES(`autostart`)",
        (&settings.id, &settings.name, TS_TYPE, &settings.autostart),
    )?;

    pool.prep_exec(format!("INSERT INTO `{}` (`instance_id`,`host`,`port`,`identity`,`password`,`cid`) VALUES (?,?,?,?,?,?)
        ON DUPLICATE KEY UPDATE `host`=VALUES(`host`), `port`=VALUES(`port`), `identity`=VALUES(`identity`), `password`=VALUES(`password`), `cid`=VALUES(`cid`)",TS_TYPE),
        (&settings.id,&settings.host,&settings.port,&settings.identity,&settings.password,&settings.cid))?;

    Ok(())
}

/// Delete all instances from DB
pub fn clear_instances(pool: &Pool) -> Fallible<()> {
    info!("Deleting all instances!");
    pool.prep_exec("DELETE FROM `instances`", ())?;
    Ok(())
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
            UPDATE name=VALUES(name), length=VALUES(length)",
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
        })
        .collect::<Result<Vec<String>, MySqlError>>()?;

    transaction.commit()?;

    Ok(ids)
}

/// Get track by URL
pub fn get_track_by_url(url: &str, pool: &Pool) -> Fallible<Option<SongMin>> {
    let mut result = pool.prep_exec(
        "SELECT `id`,`name`,`source`,`artist`,`length` FROM `titles` t WHERE t.`source` = ?",
        (url.trim(),),
    )?;

    if let Some(row) = result.next() {
        let (id, name, source, artist, length) = from_row_opt(row?)?;
        Ok(Some(SongMin {
            id,
            name,
            source,
            artist,
            length,
        }))
    } else {
        Ok(None)
    }
}

/// Load instance storage
/// Returns default if none found
pub fn read_instance_storage(id: &i32, pool: &Pool) -> Fallible<InstanceStorage> {
    let mut result = pool.prep_exec(
        "SELECT `id`,`volume`,`index`,`position`,`random`,`repeat`,`queue_lock`,`volume_lock` FROM `instance_store` is 
        WHERE is.`id` = ?",
        (&id,),
    )?;

    let storage = match result.next() {
        Some(row) => {
            let (id, volume, index, position, random, repeat, queue_lock, volume_lock) =
                from_row_opt(row?)?;
            InstanceStorage {
                id,
                volume,
                index,
                position,
                random,
                repeat,
                queue_lock,
                volume_lock,
            }
        }
        None => InstanceStorage {
            id: id.clone(),
            volume: 0.2,
            index: None,
            position: None,
            random: false,
            repeat: false,
            queue_lock: false,
            volume_lock: false,
        },
    };
    Ok(storage)
}

/// Insert or update instance storage
pub fn upsert_instance_storage(storage: &InstanceStorage, pool: &Pool) -> Fallible<()> {
    pool.prep_exec(
        "INSERT INTO `instance_store` (`id`,`volume`,`index`,`position`,`random`,`repeat`,`queue_lock`,`volume_lock`) VALUES (?,?,?,?,?,?,?)
        ON DUPLICATE KEY UPDATE `volume`=VALUES(`volume`), `index`=VALUES(`index`), `position`=VALUES(`position`), `random`=VALUES(`random`), `repeat`=VALUES(`repeat`), `queue_lock`=VALUES(`queue_lock`), `volume_lock`=VALUES(`volume_lock`)",
        (&storage.id, &storage.volume, &storage.index, &storage.position, &storage.random, &storage.repeat,
        &storage.queue_lock, &storage.volume_lock),
    )?;
    Ok(())
}

/// Create ID for track
fn calculate_id(track: &Track) -> String {
    let mut hasher = MetroHash128::default();
    track.hash(&mut hasher);
    let (h1, h2) = hasher.finish128();
    format!("{:x}{:x}", h1, h2)
}

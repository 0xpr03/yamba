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
use mysql::{Opts, OptsBuilder, Pool};

use models;
use ytdl::Track;

use std::hash::Hash;
use std::vec::Vec;

use SETTINGS;

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

/// Save a set of tracks into the DB and return their IDs
pub fn insert_tracks(tracks: &[Track], pool: Pool) -> Fallible<Vec<String>> {
    let mut transaction = pool.start_transaction(false, None, None)?;

    let ids = tracks
        .iter()
        .map(|track| {
            let id = calculate_id(track);
            transaction.prep_exec("INSERT INTO `titles` (`id`,`name`,`source`,`downloaded`, `artist`, `length`) VALUES (?,?,?,?,?,?)", (&id, &track.title,&track.webpage_url,0,&track.artist,track.duration))?;
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

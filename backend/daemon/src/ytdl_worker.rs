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
use futures::{Future, Stream};
use mpmc_scheduler as scheduler;
use mysql::Pool;
use tokio::runtime::Runtime;
use tokio::timer::Interval;
use tokio_threadpool::blocking;

use std::boxed::Box;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ytdl::YtDL;

use daemon::Instances;
use db;
use instance::{SongCache, ID};
use models::SongMin;
use SETTINGS;

/// Worker for ytdl tasks

pub type R = (YTReqWrapped, RSongs);
pub type YTReqWrapped = Box<dyn YTRequest + 'static + Send + Sync>;
pub type RSongs = Fallible<Vec<SongMin>>;
pub type Controller = scheduler::Controller<ID, YTReqWrapped, R>;
pub type YTSender = scheduler::Sender<YTReqWrapped>;

pub trait YTRequest {
    /// Url to resolve
    fn url(&self) -> &str;
    /// Callback, called after resolving of requested url with return value
    /// instance calls should be done via the instance map passed
    fn callback(&mut self, RSongs, Instances);
    /// Turn YTRequest into wrapped to send to scheduler
    fn wrap(self) -> YTReqWrapped
    where
        Self: std::marker::Sized + Send + Sync + 'static,
    {
        Box::new(self)
    }
}

/// Update scheduler for ytdl
pub fn crate_yt_updater(runtime: &mut Runtime, ytdl: Arc<YtDL>) {
    let ytdl = ytdl.clone();
    let updater = Interval::new_interval(Duration::from_secs(
        u64::from(SETTINGS.ytdl.update_intervall) * 3600,
    ))
    .for_each(move |_| {
        let _ = blocking(|| match ytdl.update_downloader() {
            Ok(_) => (),
            Err(e) => warn!("Error when updating ytdl: {}", e),
        });
        Ok(())
    })
    .map_err(|_| {});
    runtime.spawn(updater);
}

pub fn crate_ytdl_scheduler(
    runtime: &mut Runtime,
    ytdl: Arc<YtDL>,
    pool: Pool,
    cache: SongCache,
    instances: Instances,
) -> Controller {
    let (controller, scheduler) = scheduler::Scheduler::new(
        SETTINGS.ytdl.workers as usize,
        move |req: YTReqWrapped| {
            let ytdl_c = ytdl.clone();
            let start = Instant::now();
            let result = scheduler_retrieve(cache.clone(), &ytdl_c, &pool, req.url());
            let end = start.elapsed();
            debug!(
                "Request {} took {}{:03}ms to process",
                req.url(),
                end.as_secs(),
                end.subsec_millis()
            );
            (req, result)
        },
        Some(move |(mut req, tracks): R| {
            let instances_c = instances.clone();
            req.callback(tracks, instances_c);
        }),
        false,
    );

    runtime.spawn(scheduler);
    controller
}

/// Retrieve function for scheduler
/// query ytdl, insert into db & update cache
/// returns all song IDs
fn scheduler_retrieve(cache: SongCache, ytdl: &YtDL, pool: &Pool, url: &str) -> RSongs {
    // check DB & cache
    // also works with playlists as playlists are not expected to
    // be a source URL entry in the database
    let tracks = db::get_track_by_url(url, pool)?.and_then(|t| {
        if cache.get(&t.id).is_some() {
            Some(vec![t])
        } else {
            None
        }
    });

    Ok(match tracks {
        None => {
            let tracks = ytdl.get_url_info(url)?;
            db::insert_tracks(Some(cache), tracks, &pool)?
        }
        Some(v) => v,
    })
}

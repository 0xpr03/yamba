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

use erased_serde::Serialize;
use failure::Fallible;
use futures::sync::mpsc;
use futures::Stream;
use mysql::Pool;
use tokio::runtime::Runtime;
use tokio_threadpool::blocking;

use std::boxed::Box;
use std::sync::Arc;
use std::time::Instant;

use api::{self, APIRequest, CallbackError, CallbackErrorType};
use ytdl::YtDL;

use SETTINGS;

use db;

/// Initialize ytdl worker
/// Has to be called after init of senders
pub fn create_ytdl_worker(
    runtime: &mut Runtime,
    rx: mpsc::Receiver<APIRequest>,
    ytdl: Arc<YtDL>,
    pool: Pool,
) {
    let worker_future = rx.for_each(move |request| {
        let ytdl = ytdl.clone();
        let pool = pool.clone();
        debug!("Received work request: {:?}", request);
        use api::RequestType;
        let start = Instant::now();
        let response = match request.request_type {
            RequestType::Playlist(v) => {
                handle_request(v, request.request_id, ytdl, pool, handle_playlist)
            }
        };

        let response: Box<Serialize> = match response {
            Ok(v) => Box::new(v),
            Err(e) => Box::new(e),
        };

        let end = start.elapsed();
        debug!(
            "Request took {}{:03}ms to process",
            end.as_secs(),
            end.subsec_millis()
        );
        if request.callback {
            //SETTINGS.
            // todo callback
            match api::api_send_callback(
                &SETTINGS.main.api_callback_ip,
                SETTINGS.main.api_callback_port,
                "music/addSongs",
                &response,
            ) {
                Ok(_) => info!("Callback successfull"),
                Err(e) => warn!("Callback errored: {}", e),
            }
        }
        Ok(())
    });
    runtime.spawn(worker_future);
}

fn handle_playlist(
    request: api::NewPlaylist,
    request_id: u32,
    ytdl: Arc<YtDL>,
    pool: Pool,
) -> Fallible<api::PlaylistAnswer> {
    let result = ytdl.get_playlist_info(&request.url)?;
    let ids = db::insert_tracks(&result, pool)?;
    //debug!("playlist result: {:?}", result);
    debug!("{} entries found", result.len());
    Ok(api::PlaylistAnswer {
        request_id,
        song_ids: ids,
        error_code: CallbackErrorType::NoError,
    })
}

fn handle_request<'a, R, T, F>(
    request: R,
    request_id: u32,
    ytdl: Arc<YtDL>,
    pool: Pool,
    mut handler: F,
) -> Result<T, CallbackError>
where
    F: FnMut(R, u32, Arc<YtDL>, Pool) -> Fallible<T>,
    T: Serialize,
{
    match handler(request, request_id, ytdl, pool) {
        Ok(v) => {
            debug!("Worker success");
            Ok(v)
        }
        Err(e) => {
            info!("Worker error: {}", e);
            Err(CallbackError {
                request_id: request_id,
                message: format!("{}", e),
                error_code: CallbackErrorType::UnknownError,
            })
        }
    }
}

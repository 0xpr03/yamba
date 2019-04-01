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
use futures::Future;
use reqwest::header;
use reqwest::{self, async};
use tokio::executor::{DefaultExecutor, Executor};

use super::APIErr;
use yamba_types::models::callback::*;
use SETTINGS;
use USERAGENT;

lazy_static! {
    static ref API_CLIENT_SYNC: reqwest::Client = {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(&USERAGENT),
        );
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_static(&SETTINGS.main.api_callback_secret),
        );
        reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap()
    };
    static ref API_CLIENT_ASYNC: async::Client = {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(&USERAGENT),
        );
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_static(&SETTINGS.main.api_callback_secret),
        );
        async::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap()
    };
    static ref CALLBACK_RESOLVE: String = format!(
        "http://{}:{}{}",
        SETTINGS.main.api_callback_ip, SETTINGS.main.api_callback_port, PATH_RESOLVE
    );
    static ref CALLBACK_PLAYBACK: String = format!(
        "http://{}:{}{}",
        SETTINGS.main.api_callback_ip, SETTINGS.main.api_callback_port, PATH_PLAYBACK
    );
    static ref CALLBACK_INSTANCE: String = format!(
        "http://{}:{}{}",
        SETTINGS.main.api_callback_ip, SETTINGS.main.api_callback_port, PATH_INSTANCE
    );
    static ref CALLBACK_SONG: String = format!(
        "http://{}:{}{}",
        SETTINGS.main.api_callback_ip, SETTINGS.main.api_callback_port, PATH_SONG
    );
    static ref CALLBACK_VOLUME: String = format!(
        "http://{}:{}{}",
        SETTINGS.main.api_callback_ip, SETTINGS.main.api_callback_port, PATH_VOLUME
    );
    static ref CALLBACK_TRACK: String = format!(
        "http://{}:{}{}",
        SETTINGS.main.api_callback_ip, SETTINGS.main.api_callback_port, PATH_POSITION
    );
}

/// Send song-info change (length..)
#[allow(unused)]
pub fn send_song_info(v: &InstanceStateResponse) -> Fallible<()> {
    let fut = API_CLIENT_ASYNC
        .post(CALLBACK_SONG.as_str())
        .json(v)
        .send()
        .map(|x| trace!("Song info callback response: {:?}", x))
        .map_err(|err| warn!("Error sending song info callbacK: {:?}", err));
    DefaultExecutor::current()
        .spawn(Box::new(fut))
        .map_err(|v| APIErr::ExcecutionFailed(v))?;
    Ok(())
}

/// Send instance state change
pub fn send_instance_state(v: &InstanceStateResponse) -> Fallible<()> {
    let fut = API_CLIENT_ASYNC
        .post(CALLBACK_INSTANCE.as_str())
        .json(v)
        .send()
        .map(|x| trace!("Instance state callback response: {:?}", x))
        .map_err(|err| warn!("Error sending instance state callbacK: {:?}", err));
    DefaultExecutor::current()
        .spawn(Box::new(fut))
        .map_err(|v| APIErr::ExcecutionFailed(v))?;
    Ok(())
}

/// Send playstate change
pub fn send_playback_state(v: &PlaystateResponse) -> Fallible<()> {
    let fut = API_CLIENT_ASYNC
        .post(CALLBACK_PLAYBACK.as_str())
        .json(v)
        .send()
        .map(|x| trace!("Playstate callback response: {:?}", x))
        .map_err(|err| warn!("Error sending playstate callbacK: {:?}", err));
    DefaultExecutor::current()
        .spawn(Box::new(fut))
        .map_err(|v| APIErr::ExcecutionFailed(v))?;
    Ok(())
}

/// Send position update
pub fn send_track_position_update(v: &TrackPositionUpdate) -> Fallible<()> {
    let fut = API_CLIENT_ASYNC
        .post(CALLBACK_TRACK.as_str())
        .json(v)
        .send()
        .map(|x| trace!("Position update callback response: {:?}", x))
        .map_err(|err| warn!("Error sending position update callback: {:?}", err));
    DefaultExecutor::current()
        .spawn(Box::new(fut))
        .map_err(|v| APIErr::ExcecutionFailed(v))?;
    Ok(())
}

/// Send volume change
pub fn send_volume_change(v: &VolumeChange) -> Fallible<()> {
    let fut = API_CLIENT_ASYNC
        .post(CALLBACK_VOLUME.as_str())
        .json(v)
        .send()
        .map(|x| trace!("Volume callback response: {:?}", x))
        .map_err(|err| warn!("Error sending volume callbacK: {:?}", err));
    DefaultExecutor::current()
        .spawn(Box::new(fut))
        .map_err(|v| APIErr::ExcecutionFailed(v))?;
    Ok(())
}

/// Send callback for url resolve
pub fn send_resolve(body: &ResolveResponse) {
    match API_CLIENT_SYNC
        .post(CALLBACK_RESOLVE.as_str())
        .json(body)
        .send()
    {
        Ok(v) => debug!("Callback response: {:?}", v),
        Err(e) => warn!("Error on resolve callback: {}", e),
    }
}

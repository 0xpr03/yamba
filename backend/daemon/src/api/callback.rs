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

use reqwest::header;
use reqwest::{Client, ClientBuilder};

use models::ResolveResponse;
use SETTINGS;
use USERAGENT;

lazy_static! {
    static ref API_CLIENT: Client = {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(&USERAGENT),
        );
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_static(&SETTINGS.main.api_callback_secret),
        );
        ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap()
    };
    static ref CALLBACK_RESOLVE: String = format!(
        "{}:{}/callback/resolve",
        SETTINGS.main.api_callback_ip, SETTINGS.main.api_callback_port
    );
    static ref CALLBACK_PLAYBACK: String = format!(
        "{}:{}/callback/playback",
        SETTINGS.main.api_callback_ip, SETTINGS.main.api_callback_port
    );
}

/// Send callback for url resolve
pub fn send_resolve(body: &ResolveResponse) {
    match API_CLIENT.post(CALLBACK_RESOLVE.as_str()).json(body).send() {
        Ok(v) => debug!("Callback response: {:?}", v),
        Err(e) => warn!("Error on callback: {}", e),
    }
}

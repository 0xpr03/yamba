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

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure_derive;

mod actions;
mod internal;

use actions::*;
use internal::*;

use failure::Fallible;
use regex::*;
use reqwest::{header, Client, ClientBuilder};
use ts3plugin::TsApi;
use ts3plugin::*;
use yamba_types::client_internal::*;

use std::collections::HashMap;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Fail, Debug)]
pub enum APIErr {
    #[fail(display = "Request response not successfull {}", _0)]
    NoSuccess(String),
    #[fail(display = "Error performing request {}", _0)]
    RequestError(#[cause] reqwest::Error),
}

lazy_static! {
    static ref ADDRESS: SocketAddr = env::var(TS_ENV_CALLBACK)
        .unwrap_or("127.0.0.1:1337".to_string())
        .parse::<SocketAddr>()
        .unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1337));
    static ref CALLBACK_INTERNAL: SocketAddr = env::var(TS_ENV_CALLBACK_INTERNAL)
        .unwrap_or("127.0.0.1:1330".to_string())
        .parse::<SocketAddr>()
        .unwrap();
    static ref AUTH_TOKEN: String = env::var(TS_ENV_CALLBACK_AUTH_TOKEN)
        .unwrap_or("".to_string());
    static ref ID: Option<i32> = env::var(TS_ENV_ID)
        .unwrap_or("".to_string())
        .parse::<i32>()
        .map(|v| Some(v))
        .unwrap_or(None);
    static ref R_IGNORE: Regex =
        Regex::new(r"^((Sorry, I didn't get that... Have you tried !help yet)|(RPC call failed)|(n not parseable))")
            .unwrap();
    static ref R_HELP: Regex = Regex::new(r"^((\?)|(help))$").unwrap();
    static ref R_VOL_LOCK: Regex = Regex::new(r"^(l(o?ck)?( )?v(ol(ume)?)?)$").unwrap();
    static ref R_VOL_UNLOCK: Regex = Regex::new(r"^(un?l(o?ck)?( )?v(ol(ume)?)?)$").unwrap();
    static ref R_VOL_SET: Regex = Regex::new(r"^(v(ol(ume)?)? (\d*))$").unwrap();
    static ref R_LOGIN: Regex = Regex::new(r"login ([a-zA-Z0-9]+)").unwrap();
    static ref R_VOL_GET: Regex = Regex::new(r"^v(ol(ume)?)?$").unwrap();
    static ref R_TRACK_GET: Regex = Regex::new(r"^playing$").unwrap();
    static ref R_TRACK_NEXT: Regex = Regex::new(r"^((n(e?xt)?)|(>>))$").unwrap();
    static ref R_TRACK_PREVIOUS: Regex = Regex::new(r"^(((prv)|(previxous))|<<)$").unwrap();
    static ref R_TRACK_RESUME: Regex = Regex::new(r"^((r(es(ume)?)?)|>)$").unwrap();
    static ref R_RANDOM: Regex = Regex::new(r"^random$").unwrap();
    static ref R_TRACK_PAUSE: Regex = Regex::new(r"^((pause)|(\|\|))$").unwrap();
    static ref R_TRACK_STOP: Regex = Regex::new(r"^s(to?p)?$").unwrap();
    static ref R_PLAYLIST_GET: Regex = Regex::new(r"^((playlist)|(plst))$").unwrap();
    static ref R_PLAYLIST_TRACKS_5: Regex = Regex::new(r"^t((rx)|(racks))?$").unwrap();
    static ref R_PLAYLIST_TRACKS_N: Regex = Regex::new(r"^t((rx)|(racks))? (\d*)$").unwrap();
    static ref R_QUEUE_CLEAR: Regex = Regex::new("^c(lear)?$").unwrap();
    static ref R_PLAYLIST_LOCK: Regex = Regex::new(r"^l(o?ck)?( )?p((laylist)|(lst))$").unwrap();
    static ref R_PLAYLIST_UNLOCK: Regex = Regex::new(r"^un?l(o?ck)?( )?p((laylist)|(lst))$").unwrap();
    static ref R_ENQUEUE: Regex = Regex::new(r"^q(ueue)? ([^ ]+)$").unwrap();
    static ref R_PLAYLIST_LOAD: Regex = Regex::new(r"^pl(oa)?d (.+)$").unwrap();
}

#[derive(Debug)]
struct MyTsPlugin {
    killer: Sender<()>,
    client: Arc<Client>,
    tokens: HashMap<String, String>,
}

const PLUGIN_NAME_I: &'static str = env!("CARGO_PKG_NAME");
const HELP: &str = r#"
[b]YAMBA HELP[/b]

[b]Help[/b]: !help

[i]Next track[/i]: [i]!next[/i] / [i]>>[/i]

[i]Get[/i] [b]volume[/b]: [i]!volume[/i]
[i]Set[/i] volume <vol>: [i]!volume[/i] <vol>

[i]Randomize[/i] queue: [i]!random[/i]

Get [b]current track[/b]: [I]!playing[/I]
Get [b]incoming tracks[/b]: [I]!tracks [amount][/I] amount is optional
[b]Enqueue[/b] <url> : [I]!queue[/I] <url>
"#;

/*
const HELP: &str = r#"
[b]YAMBA HELP[/b]

[b]Help[/b]: !help

[i]Get[/i] [b]volume[/b]: [i]!volume[/i]
[i]Set[/i] volume <vol>: [i]!volume[/i] <vol>
[i]Lock[/i] volume: [i]!lock volume[/i]
[i]Unlock[/i] volume: [i]!unlock volume[/i]

Get [b]current track[/b]: [I]!playing[/I]
[b]Enqueue[/b] <url> : [I]!queue[/I] <url>
Get [b]next X tracks[/b]: [I]!tracks[/I] <amount>
Defaults to 5 if amount not provided
Adds track to playback queue.
[b]Load playlist[/b] <playlist>: [I]!lpload [/I]<playlist>
Load playlist with specified name into queue
[b]Enqueue playlist of tracks[/b] <url>: [I]!pqueue[/I] <playlist>
Add a playlist (yt..) to playback queue.
[b]Next[/b] track: [I]!next[/I]
[b]Previous[/b] track: [I]!previous[/I]
[b]Resume[/b] playback: [I]!resume[/I]
[b]Pause[/b] playback: [I]!pause[/I]
[b]Stop[/b] playback: [I]!stop[/I]
"#;
*/

/// Print tracks for queue lookahead
pub fn print_tracks(connection: &ts3plugin::Connection, tracks: Vec<String>) {
    let mut message = String::from("Upcoming tracks:\n");
    tracks.iter().for_each(|track| {
        if message.len() + track.len() + 1 >= 1024 {
            let _ = connection.send_message(message.as_str());
            message = String::from("Upcoming tracks:\n");
        }
        message.push_str(track);
        message.push_str("\n");
    });
    if message.len() > 0 {
        let _ = connection.send_message(message.as_str());
    }
}

impl Plugin for MyTsPlugin {
    fn connect_status_change(
        &mut self,
        api: &mut TsApi,
        server_id: ServerId,
        status: ConnectStatus,
        error: ts3plugin::Error,
    ) {
        api.log_or_print(
            format!(
                "Connection status on {:?} : {:?} error: {:?}",
                server_id, status, error
            ),
            PLUGIN_NAME_I,
            LogLevel::Debug,
        );

        // No connection possible / got dc
        // Disconnected check possibly too fuzzy, triggers on short dcs?
        if status == ConnectStatus::Disconnected
            || error == ts3plugin::Error::FailedConnectionInitialisation
        {
            match self.killer.send(()) {
                Ok(_) => (),
                Err(e) => api.log_or_print(
                    format!("Unable to stop heartbeat\nReason: {}", e),
                    PLUGIN_NAME_I,
                    LogLevel::Error,
                ),
            }
        }

        if status == ConnectStatus::ConnectionEstablished {
            match connected(*ID.as_ref().unwrap(), &api, &*self.client) {
                Err(e) => {
                    api.log_or_print(
                        format!("Error trying to signal connected state to backend, stopping heartbeat: {}", e),
                        PLUGIN_NAME_I,
                        LogLevel::Error,
                    );
                    match self.killer.send(()) {
                        Ok(_) => (),
                        Err(e) => api.log_or_print(
                            format!("Unable to stop heartbeat\nReason: {}", e),
                            PLUGIN_NAME_I,
                            LogLevel::Error,
                        ),
                    }
                }
                Ok(_) => api.log_or_print(format!("Send connected"), PLUGIN_NAME_I, LogLevel::Info),
            }
        }
    }

    fn new(api: &mut TsApi) -> Result<Box<MyTsPlugin>, InitError> {
        api.log_or_print("Initializing ", PLUGIN_NAME_I, LogLevel::Debug);

        let rpc_host: String = format!("http://{}", ADDRESS.to_string());

        api.log_or_print(
            format!("RPC Host: {}", rpc_host),
            PLUGIN_NAME_I,
            LogLevel::Debug,
        );

        if ID.is_none() {
            return Err(InitError::Failure);
        }

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&AUTH_TOKEN)
                .expect("Can't parse AUTH TOKEN as header value!"),
        );

        let client = Arc::new(
            ClientBuilder::new()
                .no_proxy()
                .tcp_nodelay()
                .default_headers(headers)
                .build()
                .expect("Can't create HTTP client!"),
        );

        let (sender, receiver) = channel();
        let id_copy = ID.clone();
        let client_cpy = client.clone();
        thread::spawn(move || {
            let mut failed_heartbeats = 0;
            if let Some(id) = id_copy {
                while receiver.recv_timeout(Duration::from_secs(1)).is_err() {
                    match heartbeat(id, &*client_cpy) {
                        Ok(_) => {
                            failed_heartbeats = 0;
                        }
                        Err(e) => {
                            failed_heartbeats += 1;
                            TsApi::static_log_or_print(
                                format!(
                                    "Backend server did not respond {} times!\nReason {}",
                                    failed_heartbeats, e
                                ),
                                PLUGIN_NAME_I,
                                LogLevel::Warning,
                            );
                        }
                    }
                }
            } else {
                TsApi::static_log_or_print(
                    format!("No instance ID!"),
                    PLUGIN_NAME_I,
                    LogLevel::Critical,
                );
            }
        });

        let me = MyTsPlugin {
            killer: sender,
            client,
            tokens: HashMap::new(),
        };

        api.log_or_print(format!("{:?}", me), PLUGIN_NAME_I, LogLevel::Debug);

        Ok(Box::new(me))
    }

    fn shutdown(&mut self, api: &mut TsApi) {
        match self.killer.send(()) {
            Ok(_) => (),
            Err(e) => api.log_or_print(
                format!("Unable to stop heartbeat\nReason: {}", e),
                PLUGIN_NAME_I,
                LogLevel::Error,
            ),
        }
        api.log_or_print("Shutdown", PLUGIN_NAME_I, LogLevel::Info);
    }

    fn message(
        &mut self,
        api: &mut TsApi,
        server_id: ServerId,
        invoker: Invoker,
        target: MessageReceiver,
        message: String,
        _ignored: bool,
    ) -> bool {
        let invoker_name: String = invoker.get_name().to_string();

        // if target == MessageReceiver::Channel {
        //     if !message.starts_with("!") {
        //         return false;
        //     }
        // }
        match target {
            MessageReceiver::Channel => {
                if !message.starts_with("!") {
                    return false;
                }
            }
            _ => (),
        }

        if let Some(server) = api.get_server(server_id) {
            if Ok(invoker.get_id()) == server.get_own_connection_id() {
                return false;
            }

            if let Some(connection) = server.get_connection(invoker.get_id()) {
                let mut message = message.trim();
                if message.starts_with("!") {
                    let (_, msg) = message.split_at(1);
                    message = msg;
                }

                let invoker_id = invoker.get_uid();
                let token = self.tokens.get(invoker_id).map(|t| t.clone());

                api.log_or_print(
                    format!("\"{}\" from \"{}\"", message, invoker_name),
                    PLUGIN_NAME_I,
                    LogLevel::Info,
                );

                // handle login special case here, to avoid more param passing
                if let Some(cpt) = R_LOGIN.captures(&message) {
                    self.tokens
                        .insert(invoker.get_uid().clone(), cpt[1].to_string());
                    send_msg(connection, "Token stored", api);
                } else if let Err(e) =
                    handle_message(message, token, &*self.client, target, connection, api)
                {
                    api.log_or_print(
                        format!("Error handling command: {}", e),
                        PLUGIN_NAME_I,
                        LogLevel::Warning,
                    );
                }
            }
        }
        return false;
    }
}

fn handle_message(
    message: &str,
    token: Option<String>,
    client: &Client,
    target: MessageReceiver,
    connection: &Connection,
    api: &TsApi,
) -> Fallible<()> {
    if R_IGNORE.is_match(&message) {
        // IGNORED MESSAGES
    } else if R_HELP.is_match(&message) {
        let _ = connection.send_message(HELP);
    } else if R_TRACK_NEXT.is_match(&message) {
        handle_action("next", &token, client)?;
    } else if let Some(cpt) = R_VOL_SET.captures(&message) {
        handle_action_cap::<usize>(cpt, 4, "volume_set", &token, client, connection, api)?;
    } else if let Some(cpt) = R_ENQUEUE.captures(&message) {
        handle_action_cap::<String>(cpt, 2, "enqueue", &token, client, connection, api)?;
    } else if R_TRACK_GET.is_match(&message) {
    } else {
        if match target {
            MessageReceiver::Connection(_) => true,
            _ => false,
        } {
            let _ =
                connection.send_message("Sorry, I didn't get that... Have you tried !help yet?");
        }
    }
    Ok(())
}

/// Helper, send message to connections and log errors
fn send_msg(conn: &Connection, msg: &str, api: &TsApi) {
    if let Err(e) = conn.send_message(msg) {
        api.log_or_print(
            format!("Message: {:?}", e),
            PLUGIN_NAME_I,
            LogLevel::Warning,
        );
    }
}

create_plugin!(
    PLUGIN_NAME_I,
    env!("CARGO_PKG_VERSION"),
    env!("CARGO_PKG_AUTHORS"),
    "yamba ts3 controller",
    ConfigureOffer::No,
    true,
    MyTsPlugin
);

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
extern crate ts3plugin;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;
extern crate regex;

use jsonrpc_client_http::HttpTransport;
use regex::*;
use std::env;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use ts3plugin::TsApi;
use ts3plugin::*;

jsonrpc_client!(
    #[derive(Debug)]
    pub struct BackendRPCClient {
    // Return: message
    pub fn heartbeat(&mut self, id : i32) -> RpcRequest<(String)>;

    // Return: allowed, message, Volume [0 - 100]
    pub fn volume_get(&mut self, id : i32, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, i32)>;
    // Return: allowed, message, success
    pub fn volume_set(&mut self, id : i32, invokerName : String, invokerGroups : String, volume : i32) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn volume_lock(&mut self, id : i32, invokerName : String, invokerGroups : String, lock : bool) -> RpcRequest<(bool, String, bool)>;

    // Return: allowed, message, title
    pub fn track_get(&mut self, id : i32, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, String)>;
    // Return: allowed, message, success
    pub fn track_next(&mut self, id : i32, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn track_previous(&mut self, id : i32, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn track_resume(&mut self, id : i32, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn track_pause(&mut self, id : i32, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn track_stop(&mut self, id : i32, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, bool)>;

    // Return: allowed, message, name
    pub fn playlist_get(&mut self, id : i32, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, String)>;
    // n <= 0: return all tracks
    // n > 0: return the next n tracks
    // Return: allowed, message, tracklist
    pub fn playlist_tracks(&mut self, id : i32, invokerName : String, invokerGroups : String, n : i32) -> RpcRequest<(bool, String, Vec<String>)>;
    // Return: allowed, message, success
    pub fn playlist_clear(&mut self, id : i32, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn playlist_lock(&mut self, id : i32, invokerName : String, invokerGroups : String, lock : bool) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn playlist_queue(&mut self, id : i32, invokerName : String, invokerGroups : String, url : String) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn playlist_load(&mut self, id : i32, invokerName : String, invokerGroups : String, playlist_name : String) -> RpcRequest<(bool, String, bool)>;
});

lazy_static! {
    static ref PORT: u16 = env::var("CALLBACK_YAMBA")
        .unwrap_or("1337".to_string())
        .parse::<u16>()
        .unwrap_or(1337);
    pub static ref ID: Option<i32> = env::var("ID_YAMBA")
        .unwrap_or("".to_string())
        .parse::<i32>()
        .map(|v| Some(v))
        .unwrap_or(None);
}

#[derive(Debug)]
struct MyTsPlugin {
    killer: Sender<()>,
    rpc_host: String,
    client_mut: Arc<Mutex<BackendRPCClient<jsonrpc_client_http::HttpHandle>>>,
}

const PLUGIN_NAME_I: &'static str = env!("CARGO_PKG_NAME");

impl Plugin for MyTsPlugin {
    fn name() -> String {
        PLUGIN_NAME_I.into()
    }
    fn version() -> String {
        env!("CARGO_PKG_VERSION").into()
    }
    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }
    fn description() -> String {
        "yamba ts3 controller".into()
    }
    fn autoload() -> bool {
        true
    }
    fn configurable() -> ConfigureOffer {
        ConfigureOffer::No
    }

    fn new(api: &mut TsApi) -> Result<Box<MyTsPlugin>, InitError> {
        api.log_or_print("Initializing ", PLUGIN_NAME_I, LogLevel::Debug);

        let rpc_host: String;
        rpc_host = format!("http://localhost:{}/", PORT.to_string());

        if ID.is_none() {
            return Err(InitError::Failure);
        }

        let transport = HttpTransport::new().standalone().unwrap();
        let transport_handle = transport.handle(&rpc_host).unwrap();
        let client = BackendRPCClient::new(transport_handle);
        let client_mut_arc = Arc::new(Mutex::from(client));
        let client_mut_heartbeat = client_mut_arc.clone();
        let client_mut_self = client_mut_arc.clone();

        let (sender, receiver) = channel();
        let id_copy = ID.clone();
        thread::spawn(move || {
            let mut failed_heartbeats = 0;
            if let Some(id) = id_copy {
                while receiver.recv_timeout(Duration::from_secs(1)).is_err() {
                    if let Ok(mut client_lock) = client_mut_heartbeat.lock() {
                        match client_lock.heartbeat(id).call() {
                            Ok(res) => {
                                failed_heartbeats = 0;
                                TsApi::static_log_or_print(
                                    format!("Server responded with {}", res),
                                    PLUGIN_NAME_I,
                                    LogLevel::Debug,
                                );
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
            rpc_host: rpc_host,
            client_mut: client_mut_self,
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
        api: &mut ::TsApi,
        server_id: ::ServerId,
        invoker: ::Invoker,
        target: ::MessageReceiver,
        message: String,
        ignored: bool,
    ) -> bool {
        let id: i32 = *ID.as_ref().unwrap();
        let invoker_name: String = invoker.get_name().to_string();
        let invoker_groups: String;

        if let Some(server) = api.get_server(server_id) {
            if Ok(invoker.get_id()) == server.get_own_connection_id() {
                return false;
            }

            if let Some(connection) = server.get_connection(invoker.get_id()) {
                if let Ok(value) = api.get_string_client_properties(
                    ClientProperties::Servergroups,
                    &invoker.get_id(),
                    &server_id,
                ) {
                    invoker_groups = value.to_owned_string_lossy();

                    let r_ignore =
                        RegexSet::new(&[r"^Sorry, I didn't get that... Have you tried !help yet"])
                            .unwrap();

                    let r_help = RegexSet::new(&[r"^\?", r"^!h", r"^!help"]).unwrap();

                    let r_vol_lock = RegexSet::new(&[r"^!l volume", r"^!lock volume"]).unwrap();
                    let r_vol_unlock =
                        RegexSet::new(&[r"^!ul volume", r"^!unlock volume"]).unwrap();
                    let r_vol_set =
                        RegexSet::new(&[r"^!v (\d)", r"^!vol (\d)", r"^!volume (\d)"]).unwrap();
                    let r_vol_get = RegexSet::new(&[r"^!v", r"^!vol", r"^!volume"]).unwrap();

                    let r_track_get = RegexSet::new(&[r"^!playing", r"^!p"]).unwrap();
                    let r_track_next =
                        RegexSet::new(&[r"^!next", r"^!nxt", r"^!n", r"^>>"]).unwrap();
                    let r_track_previous =
                        RegexSet::new(&[r"^!previous", r"^!prv", r"^!p", r"^<<"]).unwrap();
                    let r_track_resume =
                        RegexSet::new(&[r"^!resume", r"^!res", r"^!r", r"^>"]).unwrap();
                    let r_track_pause = RegexSet::new(&[r"^!pause", r"^\|\|"]).unwrap();
                    let r_track_stop = RegexSet::new(&[r"^!stop", r"^!stp", r"^!s"]).unwrap();

                    let r_playlist_get = RegexSet::new(&[r"^!playlist"]).unwrap();
                    let r_playlist_tracks_5 = RegexSet::new(&[r"^!t", r"^!tracks"]).unwrap();
                    let r_playlist_tracks_n =
                        RegexSet::new(&[r"^!t (\d)", r"^!tracks (\d)"]).unwrap();
                    let r_playlist_tracks_all =
                        RegexSet::new(&[r"^!t all", r"^!tracks all"]).unwrap();
                    let r_playlist_clear = RegexSet::new(&[r"^!c", r"^!clear"]).unwrap();
                    let r_playlist_lock =
                        RegexSet::new(&[r"^!l playlist", r"^!lock playlist"]).unwrap();
                    let r_playlist_unlock =
                        RegexSet::new(&[r"^!ul playlist", r"^!unlock playlist"]).unwrap();
                    let r_playlist_queue =
                        RegexSet::new(&[r"^!q ([^ ]+://[^ ]+)", r"^!queue ([^ ]+://[^ ]+)"])
                            .unwrap();
                    let r_playlist_load =
                        RegexSet::new(&[r"^!ld ([^ ]+)", r"^!load ([^ ]+)"]).unwrap();

                    if let Ok(mut client_lock) = self.client_mut.lock() {
                        if r_ignore.is_match(&message) {
                            // IGNORED MESSAGES
                        } else if r_help.is_match(&message) {
                            let _ = connection.send_message(
                                r#"
Hi! I'm YAMBA! This is how you can use me:

?, !h, !help → Display this help

!l volume, !lock volume → lock the volume
!ul volume, !unlock volume → unlock the volume
!v <volume>, !vol <volume>, !volume <volume> → set the volume to <volume>
!v, !vol, !volume → return the current volume

!p, !playing → return the currently playing track
>>, !n, !nxt, !next → play the next track
<<, !p, !prv, !previous → play the previous track
>, !r, !res, !resume → resume the paused track
||, !pause → pause the currently playing track
!s, !stp, !stop → stop the currently playing track

!playlist → return the currently playing playlist
!t, !tracks → return the next 5 tracks in the currently playing playlist
!t <n>, !tracks <n> → return the next <n> tracks in the currently playing playlist
!t all, !tracks all → return all tracks in the currently playing playlist
!c, !clear → clear the currently playing playlist
!l playlist, !lock playlist → lock the currently playing playlist
!ul playlist, !unlock playlist → unlock the currently playing playlist
!q <url>, !queue <url> → add <url> to currently playing playlist
!ld <playlist>, !load <playlist> → load and start playing the playlist <playlist>
"#,
                            );
                        } else if r_vol_lock.is_match(&message) {
                            match client_lock
                                .volume_lock(id, invoker_name, invoker_groups, true)
                                .call()
                            {
                                Ok(res) => {}
                                Err(e) => {
                                    let _ = connection
                                        .send_message(format!("RPC call failed\nReason {}", e));
                                }
                            }
                        } else if r_vol_unlock.is_match(&message) {
                            match client_lock
                                .volume_lock(id, invoker_name, invoker_groups, false)
                                .call()
                            {
                                Ok(res) => {}
                                Err(e) => {
                                    let _ = connection
                                        .send_message(format!("RPC call failed\nReason {}", e));
                                }
                            }
                        } else if r_vol_set.is_match(&message) {
                            match client_lock
                                .volume_set(id, invoker_name, invoker_groups, -1)
                                .call()
                            {
                                Ok(res) => {}
                                Err(e) => {
                                    let _ = connection
                                        .send_message(format!("RPC call failed\nReason {}", e));
                                }
                            }
                        } else if r_vol_get.is_match(&message) {
                            match client_lock
                                .volume_get(id, invoker_name, invoker_groups)
                                .call()
                            {
                                Ok(res) => {}
                                Err(e) => {
                                    let _ = connection
                                        .send_message(format!("RPC call failed\nReason {}", e));
                                }
                            }
                        } else {
                            let _ = connection.send_message(
                                "Sorry, I didn't get that... Have you tried !help yet?",
                            );
                        }
                    }
                } else {
                    let _ =
                        connection.send_message("Internal Error: Couldn't get your server groups!");
                }
            }
        }
        return false;
    }
}

create_plugin!(MyTsPlugin);

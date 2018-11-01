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
    pub static ref R_IGNORE: Regex =
        Regex::new(r"^((Sorry, I didn't get that... Have you tried !help yet)|(RPC call failed)|(n not parseable))")
            .unwrap();
    pub static ref R_HELP: Regex = Regex::new(r"^((\?)|(!h(e?lp)?))").unwrap();
    pub static ref R_VOL_LOCK: Regex = Regex::new(r"^(!l(o?ck)?( )?v(ol(ume)?)?)").unwrap();
    pub static ref R_VOL_UNLOCK: Regex = Regex::new(r"^(!un?l(o?ck)?( )?v(ol(ume)?)?)").unwrap();
    pub static ref R_VOL_SET: Regex = Regex::new(r"^(!v(ol(ume)?)? (\d*))").unwrap();
    pub static ref R_VOL_GET: Regex = Regex::new(r"^!v(ol(ume)?)?").unwrap();
    pub static ref R_TRACK_GET: Regex = Regex::new(r"^!p(laying)?").unwrap();
    pub static ref R_TRACK_NEXT: Regex = Regex::new(r"^((!n(e?xt)?)|(>>))").unwrap();
    pub static ref R_TRACK_PREVIOUS: Regex = Regex::new(r"^((!(p(re?v)?)|(previous))|<<)").unwrap();
    pub static ref R_TRACK_RESUME: Regex = Regex::new(r"^((!r(es(ume)?)?)|>)").unwrap();
    pub static ref R_TRACK_PAUSE: Regex = Regex::new(r"^((!pause)|(\|\|))").unwrap();
    pub static ref R_TRACK_STOP: Regex = Regex::new(r"^!s(to?p)?").unwrap();
    pub static ref R_PLAYLIST_GET: Regex = Regex::new(r"^!playlist").unwrap();
    pub static ref R_PLAYLIST_TRACKS_5: Regex = Regex::new(r"^!t((rx)|(racks))?").unwrap();
    pub static ref R_PLAYLIST_TRACKS_N: Regex = Regex::new(r"^!t((rx)|(racks))? (\d*)").unwrap();
    pub static ref R_PLAYLIST_TRACKS_ALL: Regex = Regex::new(r"^!t((rx)|(racks))? a(ll)?").unwrap();
    pub static ref R_PLAYLIST_CLEAR: Regex = Regex::new("^!c(lear)?").unwrap();
    pub static ref R_PLAYLIST_LOCK: Regex = Regex::new(r"^!l(o?ck)?( )?p(laylist)?").unwrap();
    pub static ref R_PLAYLIST_UNLOCK: Regex = Regex::new(r"^!un?l(o?ck)?( )?p(laylist)?").unwrap();
    pub static ref R_PLAYLIST_QUEUE: Regex = Regex::new(r"^!q(ueue)? ([^ ]+)").unwrap();
    pub static ref R_PLAYLIST_LOAD: Regex = Regex::new(r"^!l(oa)?d (.+)").unwrap();
}

#[derive(Debug)]
struct MyTsPlugin {
    killer: Sender<()>,
    rpc_host: String,
    client_mut: Arc<Mutex<BackendRPCClient<jsonrpc_client_http::HttpHandle>>>,
}

const PLUGIN_NAME_I: &'static str = env!("CARGO_PKG_NAME");
const HELP: &str = r#"
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
"#;

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
                    if let Ok(mut client_lock) = self.client_mut.lock() {
                        let mut is_rpc_error: bool = false;
                        let mut rpc_error: jsonrpc_client_core::Error =
                            jsonrpc_client_core::Error::from_kind(
                                jsonrpc_client_core::ErrorKind::Msg(String::from("No error")),
                            );
                        let mut rpc_allowed: bool = true;
                        let mut rpc_message: String = String::from("");

                        api.log_or_print(
                            format!("\"{}\" from \"{}\"", message, invoker_name),
                            PLUGIN_NAME_I,
                            LogLevel::Info,
                        );

                        if R_IGNORE.is_match(&message) {
                            // IGNORED MESSAGES
                        } else if R_HELP.is_match(&message) {
                            let _ = connection.send_message(HELP);
                        } else if R_VOL_LOCK.is_match(&message) {
                            match client_lock
                                .volume_lock(id, invoker_name, invoker_groups, true)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let success = if res.2 { "Ok" } else { "Failure" };
                                    if rpc_allowed {
                                        let _ = connection.send_message(format!("{}", success));
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if R_VOL_UNLOCK.is_match(&message) {
                            match client_lock
                                .volume_lock(id, invoker_name, invoker_groups, false)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let success = if res.2 { "Ok" } else { "Failure" };
                                    if rpc_allowed {
                                        let _ = connection.send_message(format!("{}", success));
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if let Some(caps) = R_VOL_SET.captures(&message) {
                            if let Ok(vol) = caps[4].parse::<i32>() {
                                match client_lock
                                    .volume_set(id, invoker_name, invoker_groups, vol)
                                    .call()
                                {
                                    Ok(res) => {
                                        rpc_allowed = res.0;
                                        rpc_message = res.1;
                                        let success = if res.2 { "Ok" } else { "Failure" };
                                        if rpc_allowed {
                                            let _ = connection.send_message(format!("{}", success));
                                        }
                                    }
                                    Err(e) => {
                                        is_rpc_error = true;
                                        rpc_error = e;
                                    }
                                }
                            } else {
                                let _ = connection.send_message(format!("n not parseable"));
                            }
                        } else if R_VOL_GET.is_match(&message) {
                            match client_lock
                                .volume_get(id, invoker_name, invoker_groups)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let vol = res.2;
                                    if rpc_allowed {
                                        let _ = connection.send_message(format!("{}", vol));
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if R_TRACK_GET.is_match(&message) {
                            match client_lock
                                .track_get(id, invoker_name, invoker_groups)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let title = res.2;
                                    if rpc_allowed {
                                        let _ = connection.send_message(format!("{}", title));
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if R_TRACK_NEXT.is_match(&message) {
                            match client_lock
                                .track_next(id, invoker_name, invoker_groups)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let success = if res.2 { "Ok" } else { "Failure" };
                                    if rpc_allowed {
                                        let _ = connection.send_message(format!("{}", success));
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if R_TRACK_PREVIOUS.is_match(&message) {
                            match client_lock
                                .track_previous(id, invoker_name, invoker_groups)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let success = if res.2 { "Ok" } else { "Failure" };
                                    if rpc_allowed {
                                        let _ = connection.send_message(format!("{}", success));
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if R_TRACK_RESUME.is_match(&message) {
                            match client_lock
                                .track_resume(id, invoker_name, invoker_groups)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let success = if res.2 { "Ok" } else { "Failure" };
                                    if rpc_allowed {
                                        let _ = connection.send_message(format!("{}", success));
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if R_TRACK_PAUSE.is_match(&message) {
                            match client_lock
                                .track_pause(id, invoker_name, invoker_groups)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let success = if res.2 { "Ok" } else { "Failure" };
                                    if rpc_allowed {
                                        let _ = connection.send_message(format!("{}", success));
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if R_TRACK_STOP.is_match(&message) {
                            match client_lock
                                .track_stop(id, invoker_name, invoker_groups)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let success = if res.2 { "Ok" } else { "Failure" };
                                    if rpc_allowed {
                                        let _ = connection.send_message(format!("{}", success));
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if R_PLAYLIST_GET.is_match(&message) {
                            match client_lock
                                .playlist_get(id, invoker_name, invoker_groups)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let name = res.2;
                                    if rpc_allowed {
                                        let _ = connection.send_message(format!("{}", name));
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if R_PLAYLIST_TRACKS_ALL.is_match(&message) {
                            match client_lock
                                .playlist_tracks(id, invoker_name, invoker_groups, -1)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let tracks = res.2;
                                    if rpc_allowed {
                                        tracks.into_iter().for_each(|track| {
                                            let _ = connection.send_message(format!("{}", track));
                                        });
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if let Some(caps) = R_PLAYLIST_TRACKS_N.captures(&message) {
                            if let Ok(n) = caps[4].parse::<i32>() {
                                match client_lock
                                    .playlist_tracks(id, invoker_name, invoker_groups, n)
                                    .call()
                                {
                                    Ok(res) => {
                                        rpc_allowed = res.0;
                                        rpc_message = res.1;
                                        let tracks = res.2;
                                        if rpc_allowed {
                                            tracks.into_iter().for_each(|track| {
                                                let _ =
                                                    connection.send_message(format!("{}", track));
                                            });
                                        }
                                    }
                                    Err(e) => {
                                        is_rpc_error = true;
                                        rpc_error = e;
                                    }
                                }
                            } else {
                                let _ = connection.send_message(format!("n not parseable"));
                            }
                        } else if R_PLAYLIST_TRACKS_5.is_match(&message) {
                            match client_lock
                                .playlist_tracks(id, invoker_name, invoker_groups, 5)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let tracks = res.2;
                                    if rpc_allowed {
                                        tracks.into_iter().for_each(|track| {
                                            let _ = connection.send_message(format!("{}", track));
                                        });
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if R_PLAYLIST_CLEAR.is_match(&message) {
                            match client_lock
                                .playlist_clear(id, invoker_name, invoker_groups)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let success = if res.2 { "Ok" } else { "Failure" };
                                    if rpc_allowed {
                                        let _ = connection.send_message(format!("{}", success));
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if R_PLAYLIST_LOCK.is_match(&message) {
                            match client_lock
                                .playlist_lock(id, invoker_name, invoker_groups, true)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let success = if res.2 { "Ok" } else { "Failure" };
                                    if rpc_allowed {
                                        let _ = connection.send_message(format!("{}", success));
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if R_PLAYLIST_UNLOCK.is_match(&message) {
                            match client_lock
                                .playlist_lock(id, invoker_name, invoker_groups, false)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let success = if res.2 { "Ok" } else { "Failure" };
                                    if rpc_allowed {
                                        let _ = connection.send_message(format!("{}", success));
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if let Some(caps) = R_PLAYLIST_QUEUE.captures(&message) {
                            let url = String::from(&caps[2]);
                            match client_lock
                                .playlist_queue(id, invoker_name, invoker_groups, url)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let success = if res.2 { "Ok" } else { "Failure" };
                                    if rpc_allowed {
                                        let _ = connection.send_message(format!("{}", success));
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else if R_PLAYLIST_LOAD.is_match(&message) {
                            let playlist_name =
                                String::from(&R_VOL_SET.captures(&message).unwrap()[4]);
                            match client_lock
                                .playlist_load(id, invoker_name, invoker_groups, playlist_name)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let success = if res.2 { "Ok" } else { "Failure" };
                                    if rpc_allowed {
                                        let _ = connection.send_message(format!("{}", success));
                                    }
                                }
                                Err(e) => {
                                    is_rpc_error = true;
                                    rpc_error = e;
                                }
                            }
                        } else {
                            let _ = connection.send_message(
                                "Sorry, I didn't get that... Have you tried !help yet?",
                            );
                        }

                        if is_rpc_error {
                            let _ = connection
                                .send_message(format!("RPC call failed\nReason {}", rpc_error));
                        } else if !rpc_allowed {
                            let _ = connection.send_message(format!(
                                "Action not allowed!\nReason: {}",
                                rpc_message
                            ));
                        }
                    }
                /*
                        
                */
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

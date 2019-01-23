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
use std::process;
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
    // Call when connected
    pub fn connected(&mut self, id: i32, process: u32) -> RpcRequest<bool>;
    // Return: message
    pub fn heartbeat(&mut self, id : i32) -> RpcRequest<(String)>;

    // Return: allowed, message, Volume [0 - 100]
    pub fn volume_get(&mut self, id : i32, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, f64)>;
    // Return: allowed, message, success
    pub fn volume_set(&mut self, id : i32, invokerName : String, invokerGroups : String, volume : f64) -> RpcRequest<(bool, String, bool)>;
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
    // n > 0: return the next n tracks
    // n <= 0 isn't allowed
    // Return: allowed, message, tracklist
    pub fn queue_tracks(&mut self, id : i32, invokerName : String, invokerGroups : String, n : i32) -> RpcRequest<(bool, String, Vec<String>)>;
    // Return: allowed, message, success
    pub fn queue_clear(&mut self, id : i32, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn queue_lock(&mut self, id : i32, invokerName : String, invokerGroups : String, lock : bool) -> RpcRequest<(bool, String, bool)>;
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
    pub static ref R_TRACK_GET: Regex = Regex::new(r"^!playing").unwrap();
    pub static ref R_TRACK_NEXT: Regex = Regex::new(r"^((!n(e?xt)?)|(>>))").unwrap();
    pub static ref R_TRACK_PREVIOUS: Regex = Regex::new(r"^((!(prv)|(previous))|<<)").unwrap();
    pub static ref R_TRACK_RESUME: Regex = Regex::new(r"^((!r(es(ume)?)?)|>)").unwrap();
    pub static ref R_TRACK_PAUSE: Regex = Regex::new(r"^((!pause)|(\|\|))").unwrap();
    pub static ref R_TRACK_STOP: Regex = Regex::new(r"^!s(to?p)?").unwrap();
    pub static ref R_PLAYLIST_GET: Regex = Regex::new(r"^!((playlist)|(plst))").unwrap();
    pub static ref R_PLAYLIST_TRACKS_5: Regex = Regex::new(r"^!t((rx)|(racks))?").unwrap();
    pub static ref R_PLAYLIST_TRACKS_N: Regex = Regex::new(r"^!t((rx)|(racks))? (\d*)").unwrap();
    pub static ref R_QUEUE_CLEAR: Regex = Regex::new("^!c(lear)?").unwrap();
    pub static ref R_PLAYLIST_LOCK: Regex = Regex::new(r"^!l(o?ck)?( )?p((laylist)|(lst))").unwrap();
    pub static ref R_PLAYLIST_UNLOCK: Regex = Regex::new(r"^!un?l(o?ck)?( )?p((laylist)|(lst))").unwrap();
    pub static ref R_ENQUEUE: Regex = Regex::new(r"^!q(ueue)? ([^ ]+)").unwrap();
    pub static ref R_PLAYLIST_LOAD: Regex = Regex::new(r"^!pl(oa)?d (.+)").unwrap();
}

#[derive(Debug)]
struct MyTsPlugin {
    killer: Sender<()>,
    rpc_host: String,
    client_mut: Arc<Mutex<BackendRPCClient<jsonrpc_client_http::HttpHandle>>>,
}

const PLUGIN_NAME_I: &'static str = env!("CARGO_PKG_NAME");
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

pub fn print_tracks(connection: &ts3plugin::Connection, mut tracks: Vec<String>) {
    let mut message = String::from("");
    while let Some(track) = tracks.pop() {
        if message.len() + track.len() + 1 >= 1024 {
            let _ = connection.send_message(format!("{}", message));
            message = String::from(format!("{}\n", track));
        } else {
            message = String::from(format!("{}{}\n", message, track));
        }
    }
    if message.len() > 0 {
        let _ = connection.send_message(format!("{}", message));
    }
}

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
            let mut rpc_api = self.client_mut.lock().unwrap();
            let pid = process::id();
            api.log_or_print(format!("PID: {}", pid), PLUGIN_NAME_I, LogLevel::Error);
            match rpc_api.connected(*ID.as_ref().unwrap(), pid).call() {
                Ok(v) => api.log_or_print(
                    format!("Connected response: {}", v),
                    PLUGIN_NAME_I,
                    LogLevel::Debug,
                ),
                Err(e) => {
                    api.log_or_print(
                        format!("Error trying to signal connected state to backend: {}", e),
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
            } // if we don't get this through, we're not gonna receive any audio
        }
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
        _ignored: bool,
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
                    let mut client_lock =
                        self.client_mut.lock().expect("Can't get client rpc lock!");
                    let mut is_rpc_error: bool = false;
                    let mut rpc_error: jsonrpc_client_core::Error =
                        jsonrpc_client_core::Error::from_kind(jsonrpc_client_core::ErrorKind::Msg(
                            String::from("No error"),
                        ));
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
                                .volume_set(id, invoker_name, invoker_groups, vol as f64 / 100.0)
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
                                    let _ = connection
                                        .send_message(format!("{}", (vol * 100.0) as i32));
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
                    } else if let Some(caps) = R_PLAYLIST_TRACKS_N.captures(&message) {
                        if let Ok(n) = caps[4].parse::<i32>() {
                            match client_lock
                                .queue_tracks(id, invoker_name, invoker_groups, n)
                                .call()
                            {
                                Ok(res) => {
                                    rpc_allowed = res.0;
                                    rpc_message = res.1;
                                    let tracks = res.2;
                                    if rpc_allowed {
                                        print_tracks(connection, tracks);
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
                            .queue_tracks(id, invoker_name, invoker_groups, 5)
                            .call()
                        {
                            Ok(res) => {
                                rpc_allowed = res.0;
                                rpc_message = res.1;
                                let tracks = res.2;
                                if rpc_allowed {
                                    print_tracks(connection, tracks);
                                }
                            }
                            Err(e) => {
                                is_rpc_error = true;
                                rpc_error = e;
                            }
                        }
                    } else if R_QUEUE_CLEAR.is_match(&message) {
                        match client_lock
                            .queue_clear(id, invoker_name, invoker_groups)
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
                            .queue_lock(id, invoker_name, invoker_groups, true)
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
                            .queue_lock(id, invoker_name, invoker_groups, false)
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
                    } else if let Some(caps) = R_ENQUEUE.captures(&message) {
                        let url = String::from(&caps[2])
                            .replace("[URL]", "")
                            .replace("[/URL]", "");
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
                        let playlist_name = String::from(&R_VOL_SET.captures(&message).unwrap()[4]);
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
                        if match target {
                            MessageReceiver::Connection(_) => true,
                            _ => false,
                        } {
                            let _ = connection.send_message(
                                "Sorry, I didn't get that... Have you tried !help yet?",
                            );
                        }
                    }

                    if is_rpc_error {
                        let _ = connection
                            .send_message(format!("RPC call failed\nReason: {}", rpc_error));
                    } else if !rpc_allowed {
                        let _ = connection
                            .send_message(format!("Action not allowed!\nReason: {}", rpc_message));
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

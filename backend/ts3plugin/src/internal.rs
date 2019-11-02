use super::*;

use failure::Fallible;
use reqwest::Client;
use ts3plugin::TsApi;
use ts3plugin::*;
use yamba_types::client_internal::*;

use std::process;

lazy_static! {
    // make static, used every second
    static ref HEARTBEAT_URL: String = format!("http://{}/internal/heartbeat",*CALLBACK_INTERNAL);
}

/// run heartbeat command
pub fn heartbeat(id: i32, client: &Client) -> Fallible<()> {
    match client
        .post(HEARTBEAT_URL.as_str())
        .json(&Heartbeat { id })
        .send()
    {
        Err(e) => Err(APIErr::RequestError(e).into()),
        Ok(v) => {
            if v.status() == reqwest::StatusCode::ACCEPTED {
                Ok(())
            } else {
                Err(APIErr::NoSuccess(format!("Heartbeat failed: {}", v.status())).into())
            }
        }
    }
}


/// Handle connection establish to ts server
pub fn connected(id: i32, api: &TsApi, client: &Client) -> Fallible<()> {
    let host_internal = format!("http://{}/internal/started", *CALLBACK_INTERNAL);
    api.log_or_print(
        format!("Internal RPC Host: {}", host_internal),
        PLUGIN_NAME_I,
        LogLevel::Debug,
    );
    match client
        .post(&host_internal)
        .json(&Connected {
            id,
            pid: process::id(),
        })
        .send()
    {
        Ok(v) => {
            if v.status() == reqwest::StatusCode::ACCEPTED {
                Ok(())
            } else {
                Err(APIErr::NoSuccess(format!(
                    "No success during connect-callback: {}",
                    v.status()
                ))
                .into())
            }
        }
        Err(e) => Err(APIErr::RequestError(e).into()),
    }
}
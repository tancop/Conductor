/*
 *  inject.rs
 *  (c) 2025 Teodor Potancok
 *
 *  This Source Code Form is subject to the terms of the Mozilla Public
 *  License, v. 2.0. If a copy of the MPL was not distributed with this
 *  file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use futures_util::{SinkExt, StreamExt};
use reqwest::{Error, Response};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use thiserror::Error;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TabEntry<'a> {
    title: &'a str,
    web_socket_debugger_url: String,
}

async fn find_url(res: Result<Response, Error>) -> Option<String> {
    let Ok(res) = res else {
        return None;
    };

    let Ok(bytes) = res.bytes().await else {
        return None;
    };

    let Ok(tabs) = serde_json::from_slice::<Vec<TabEntry>>(bytes.as_ref()) else {
        return None;
    };

    for tab in tabs {
        if tab.title == "SharedJSContext" {
            return Some(tab.web_socket_debugger_url);
        }
    }

    None
}

#[derive(Debug, Error)]
pub enum DebuggerUrlError {
    #[error("Failed to create HTTP client")]
    CreateClientFailed,
    #[error("Max retries reached")]
    MaxRetriesReached,
}

pub async fn try_get_debugger_url(max_tries: Option<u32>) -> Result<String, DebuggerUrlError> {
    let mut tries_left = max_tries.unwrap_or(0);

    if let Ok(client) = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
    {
        loop {
            let res = client.get("http://localhost:8080/json").send().await;

            if let Some(url) = find_url(res).await {
                return Ok(url);
            } else {
                log::info!("Connection to Steam client failed, retrying...");
                tokio::time::sleep(Duration::from_millis(1000)).await;

                if max_tries.is_some() {
                    tries_left -= 1;
                    if tries_left == 0 {
                        return Err(DebuggerUrlError::MaxRetriesReached);
                    }
                }
            }
        }
    }

    Err(DebuggerUrlError::CreateClientFailed)
}

#[derive(Deserialize)]
struct TypeContainer {
    r#type: String,
    value: Option<String>,
}

#[derive(Deserialize)]
struct Wrapper {
    result: TypeContainer,
}

#[derive(Deserialize)]
struct InstanceCheckResult {
    result: Wrapper,
}

pub async fn is_another_instance_running(url: &str, rpc_secret: &str) -> bool {
    if let Ok((ws_stream, _)) = connect_async(url).await {
        let (mut tx, mut rx) = ws_stream.split();

        let msg = json!({
            "id": 0,
            "method": "Runtime.evaluate",
            "params": {
                "expression": "window.rpcSecret",
                "awaitPromise": true
            }
        });

        let json = serde_json::ser::to_string(&msg).unwrap();

        let Ok(_) = tx.send(Message::from(json.to_owned())).await else {
            return false;
        };

        let Some(Ok(res)) = rx.next().await else {
            return false;
        };

        let Ok(res) = res.into_text() else {
            return false;
        };

        let Ok(res) = serde_json::from_str::<InstanceCheckResult>(&res) else {
            return false;
        };

        if res.result.result.r#type == "undefined" {
            return false;
        }

        let res = res.result.result.value;

        return if let Some(res) = res {
            res.as_str() != rpc_secret
        } else {
            false
        };
    }

    false
}

#[derive(Debug, Error)]
pub enum InjectError {
    #[error("Failed to connect with Steam client")]
    NotConnected,
    #[error("Failed to send payload")]
    NotSent,
}

pub async fn kill_running_instance(url: &str, max_tries: u32) -> Result<(), InjectError> {
    if let Ok((ws_stream, _)) = connect_async(url).await {
        let (mut tx, _) = ws_stream.split();

        let msg = json!({
            "id": 1,
            "method": "Runtime.evaluate",
            "params": {
                "expression": "window.terminate()",
                "awaitPromise": true
            }
        });

        let json = serde_json::ser::to_string(&msg).unwrap();

        for _ in 0..max_tries {
            if let Ok(_) = tx.send(Message::from(json.to_owned())).await {
                log::debug!("Terminated other instance");
                return Ok(());
            } else {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }

        Err(InjectError::NotSent)
    } else {
        Err(InjectError::NotConnected)
    }
}

pub async fn inject_payload(url: &str, payload: &str, max_tries: u32) -> Result<(), InjectError> {
    if let Ok((ws_stream, _)) = connect_async(url).await {
        let (mut tx, _) = ws_stream.split();

        let msg = json!({
            "id": 1,
            "method": "Runtime.evaluate",
            "params": {
                "expression": payload,
                "awaitPromise": true
            }
        });

        let json = serde_json::ser::to_string(&msg).unwrap();

        for _ in 0..max_tries {
            if let Ok(_) = tx.send(Message::from(json.to_owned())).await {
                log::debug!("Injected payload");
                return Ok(());
            } else {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }

        Err(InjectError::NotSent)
    } else {
        Err(InjectError::NotConnected)
    }
}

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
pub async fn try_get_debugger_url() -> Option<String> {
    if let Ok(client) = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
    {
        loop {
            let res = client.get("http://localhost:8080/json").send().await;

            if let Some(url) = find_url(res).await {
                return Some(url);
            } else {
                log::info!("Connection to Steam client failed, retrying...");
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    None
}

#[derive(Debug, Error)]
pub enum InjectError {
    #[error("Failed to connect with Steam client")]
    NotConnected,
    #[error("Failed to send payload")]
    NotSent,
}

pub async fn inject_payload(url: &str, payload: &str, max_tries: u32) -> Result<(), InjectError> {
    if let Ok((ws_stream, _)) = connect_async(url).await {
        let (mut write, _) = ws_stream.split();

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
            if let Ok(_) = write.send(Message::from(json.to_owned())).await {
                log::debug!("Injected payload");
                return Ok(());
            }
        }

        Err(InjectError::NotSent)
    } else {
        Err(InjectError::NotConnected)
    }
}

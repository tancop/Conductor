use futures_util::{SinkExt, StreamExt};
use reqwest::{Error, Response};
use serde::Deserialize;
use std::fs::File;
use std::time::Duration;
use tokio_tungstenite::connect_async;

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
                println!("Connection to Steam client failed, retrying...");
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    None
}

pub async fn inject_payload<'a>(url: &'a str) {
    if let Ok((ws_stream, _)) = connect_async(url).await {
        let (mut write, mut read) = ws_stream.split();
    }
}

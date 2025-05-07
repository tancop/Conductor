use crate::message::RpcRequest;
use cfg_if::cfg_if;
use futures_util::{SinkExt, StreamExt};
use log::LevelFilter;
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio::sync::{Mutex, RwLock};
use tokio::{
    io::Error,
    net::{TcpListener, TcpStream},
};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

mod inject;
mod message;
mod payload;
mod process;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Setup logger
    let filter =
        LevelFilter::from_str(&std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned()))
            .unwrap_or(LevelFilter::Info);

    env_logger::Builder::new()
        .filter_level(filter)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}] {}",
                chrono::Local::now().format("%T%.3f"),
                record.level(),
                record.args()
            )
        })
        .init();

    log::info!("Starting Conductor...");

    // Spawn server task
    tokio::spawn(start());

    // Wait for exit event
    cfg_if! {
        if #[cfg(unix)] {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {},
                _ = tokio::signal::unix::ctrl_close() => {},
            }
        } else {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {},
            }
        }
    }

    log::info!("Goodbye!");

    Ok(())
}

async fn start() {
    // Kill other instances if running
    if let Err(e) = process::kill_other_instances().await {
        log::error!("{}", e);
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    if let Err(e) = process::store_pid() {
        log::warn!("Failed to store PID: {}", e);
    }

    // Find and open payload file
    let Ok(current_dir) = std::env::current_dir() else {
        log::error!("Could not get current directory");
        std::process::exit(1);
    };

    let Ok(mut js_file) = File::open(current_dir.join("dist").join("payload.template.js")) else {
        log::error!("Could not open payload file at {}", current_dir.display());
        std::process::exit(1);
    };

    let mut payload = String::with_capacity(10_000);
    let Ok(_) = js_file.read_to_string(&mut payload) else {
        log::error!("Could not read payload file at {}", current_dir.display());
        std::process::exit(1);
    };

    // Get SteamWebHelper's debugger URL
    let Some(debugger_url) = inject::try_get_debugger_url().await else {
        log::error!("Could not find debugger url");
        std::process::exit(1);
    };

    log::debug!("Sending payload to URL: {debugger_url}");

    // Setup payload with port and secret
    let payload = payload::make_payload(&payload, 7355, true, "Secret!".to_string());

    // Start server
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:7355".to_string());

    tokio::spawn(serve(addr));

    // Inject payload into SteamWebHelper
    match inject::inject_payload(&debugger_url, &payload, 5).await {
        Ok(()) => {}
        Err(err) => {
            log::error!("{err}");
            std::process::exit(1);
        }
    }

    log::info!("Conductor initialized!");
}

struct Context {
    connected: AtomicBool,
    steam_tx: RwLock<Option<UnboundedSender<String>>>,
    last_message_id: AtomicU32,
    message_senders: RwLock<HashMap<u32, UnboundedSender<String>>>,
}

async fn serve(addr: String) {
    // Create the event loop and TCP listener we'll accept connections on
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    log::info!("Listening on: {}", addr);

    let ctx = Arc::new(Context {
        connected: false.into(),
        steam_tx: None.into(),
        last_message_id: 0.into(),
        message_senders: HashMap::new().into(),
    });

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(ctx.clone(), stream));
    }
}
async fn handle_connection(ctx: Arc<Context>, stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    log::debug!("Peer address: {}", addr);

    let mut ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            log::error!("WebSocket handshake failed: {}", e);
            return;
        }
    };

    log::debug!("New WebSocket connection: {}", addr);

    let (tx, mut rx) = unbounded_channel::<String>();
    let mut is_steam = false;

    // Handle initial message
    let initial_msg = ws_stream.next().await;
    if initial_msg.is_none() {
        return;
    }
    let initial_msg = initial_msg.unwrap();
    let initial_msg = match initial_msg {
        Ok(msg) => msg,
        Err(e) => {
            log::error!("Error receiving initial message: {}", e);
            return;
        }
    };

    let msg_text = match initial_msg.into_text() {
        Ok(text) => text,
        Err(e) => {
            log::error!("Invalid initial message format: {}", e);
            return;
        }
    };

    if !ctx.connected.load(Ordering::Relaxed) && msg_text.starts_with("init:") {
        // Steam connection
        log::debug!("Received init message: '{}'", msg_text);
        ctx.connected.store(true, Ordering::Relaxed);
        is_steam = true;

        let mut steam_tx = ctx.steam_tx.write().await;
        steam_tx.replace(tx);

        // Drop the write lock immediately
        drop(steam_tx);
    } else {
        // Client connection
        let steam_tx = ctx.steam_tx.read().await;
        if let Some(steam_tx) = steam_tx.as_ref() {
            if let Ok(mut req) = serde_json::from_str::<RpcRequest>(&msg_text) {
                req.secret = Some("Secret!".to_string());
                let message_id = ctx.last_message_id.fetch_add(1, Ordering::Relaxed);
                req.message_id = Some(message_id);

                ctx.message_senders.write().await.insert(message_id, tx);

                let req = serde_json::to_string(&req).expect("failed to serialize message");
                if let Err(e) = steam_tx.send(req) {
                    log::error!("Error sending message to Steam: {}", e);
                    return;
                }
            } else {
                log::warn!("Received invalid message: {}", msg_text);
                send_message(
                    &mut ws_stream,
                    &json!({
                        "success": false,
                        "error": "Message is not valid",
                    }),
                )
                .await;
                return;
            }
        } else {
            log::warn!("Received command before connecting to Steam");
            send_message(
                &mut ws_stream,
                &json!({
                    "success": false,
                    "error": "Not connected to Steam",
                }),
            )
            .await;
            return;
        }
    }

    // Main message loop
    loop {
        tokio::select! {
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        let msg_text = match msg.into_text() {
                            Ok(text) => text,
                            Err(e) => {
                                log::error!("Invalid message format: {}", e);
                                continue;
                            }
                        };

                        if is_steam {
                            log::debug!("Received steam message: '{}'", msg_text);
                            // Handle Steam messages
                            if let Ok(mut req) = serde_json::from_str::<serde_json::Value>(&msg_text) {
                                if let Some(req) = req.as_object_mut() {
                                    if let Some(id) = req.get("messageId").and_then(|id| id.as_u64().map(|id| id as u32)) {
                                        req.remove("messageId");

                                        if let Some(tx) = ctx.message_senders.read().await.get(&id) {
                                            if let Ok(msg) = serde_json::to_string(req) {
                                                if let Err(e) = tx.send(msg) {
                                                    log::error!("Error sending message to client: {}", e);
                                                }
                                            } else {
                                                log::error!("Failed to serialize steam message");
                                            }
                                        } else {
                                            log::warn!("No client channel found for id {id}");
                                        }
                                    } else {
                                        log::error!("Steam message has no ID");
                                    }
                                } else {
                                    log::error!("Steam message is not a JSON object");
                                }
                            } else {
                                log::error!("Failed to deserialize steam message");
                            }
                        } else {
                            log::debug!("Received client message: '{}'", msg_text);
                            // Handle client messages
                            let steam_tx = ctx.steam_tx.read().await;
                            if let Some(steam_tx) = steam_tx.as_ref() {
                                if let Ok(mut req) = serde_json::from_str::<RpcRequest>(&msg_text) {
                                    req.secret = Some("Secret!".to_string());
                                    req.message_id = Some(ctx.last_message_id.fetch_add(1, Ordering::Relaxed));

                                    if let Ok(req) = serde_json::to_string(&req) {
                                        if let Err(e) = steam_tx.send(req) {
                                            log::error!("Error sending message to Steam: {}", e);
                                            // Steam connection might be dead, reset the connection state
                                            ctx.connected.store(false, Ordering::Relaxed);
                                            break;
                                        }
                                    }
                                }
                            } else {
                                log::warn!("Steam connection not available");
                                send_message(&mut ws_stream, &json!({
                                    "success": false,
                                    "error": "Not connected to Steam",
                                })).await;
                            }
                        }
                    }
                    Some(Err(e)) => {
                        log::error!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        log::debug!("WebSocket connection closed");
                        break;
                    }
                }
            }
            msg = rx.recv() => {
                match msg {
                    Some(msg) => {
                        if let Err(e) = ws_stream.send(Message::text(msg)).await {
                            log::error!("Failed to send message: {}", e);
                            break;
                        }
                    }
                    None => {
                        log::debug!("Channel closed");
                        break;
                    }
                }
            }
        }
    }

    // Cleanup
    if is_steam {
        log::info!("Steam connection terminated");
        ctx.connected.store(false, Ordering::Relaxed);
        let mut steam_tx = ctx.steam_tx.write().await;
        steam_tx.take();
    }
}

async fn send_message(stream: &mut WebSocketStream<TcpStream>, msg: &serde_json::Value) {
    if let Ok(msg_str) = serde_json::to_string(msg) {
        if let Err(e) = stream.send(Message::text(msg_str)).await {
            log::warn!("Failed to send message: {}", e);
        }
    }
}

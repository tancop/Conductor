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

    let mut ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    log::debug!("New WebSocket connection: {}", addr);

    let mut connected = false;
    let mut is_steam = false;
    let (tx, mut rx) = unbounded_channel::<String>();

    tokio::select! {
        res = ws_stream.next() => {
            if let Some(Ok(msg)) = res {
                log::debug!("Received message from network: {msg}");

                if let Ok(msg) = msg.into_text() {
                    if !connected {
                        if !ctx.connected.load(Ordering::Relaxed) && msg.starts_with("init:") {
                            // Init message from Steam
                            log::debug!("Received init message: '{msg}'");
                            _ = ctx.connected.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed);
                            is_steam = true;
                            let mut steam_tx = ctx.steam_tx.write().await;
                            if steam_tx.is_none() {
                                steam_tx.replace(tx);
                            } else {
                                log::error!("Double init: {msg}");
                            }
                        } else {
                            // Forward message to Steam
                            let steam_tx = ctx.steam_tx.read().await;

                            if let Some(steam_tx) = steam_tx.as_ref() {
                                if let Ok(mut req) = serde_json::from_str::<RpcRequest>(&msg) {
                                    req.secret = None;
                                    let message_id = ctx.last_message_id.fetch_add(1, Ordering::Relaxed);
                                    req.message_id = Some(message_id);

                                    ctx.message_senders.write().await.insert(message_id, tx.clone());

                                    let req = serde_json::to_string(&req).expect("failed to serialize message");

                                    log::debug!("Forwarding first request: {req}");

                                    if let Err(e) = steam_tx.send(req) {
                                        log::error!("Error sending message to Steam: {e}");
                                    }
                                } else {
                                    log::warn!("Received invalid message: {msg}");
                                    send_message(&mut ws_stream, json!({
                                        "success": false,
                                        "error": "Message is not valid",
                                    }).as_str().expect("failed to serialize error message")).await;
                                }
                            } else {
                                log::warn!("Received command before connecting to Steam");
                                send_message(&mut ws_stream, json!({
                                    "success": false,
                                    "error": "Not connected to Steam",
                                }).as_str().expect("failed to serialize error message")).await;
                            }
                        }
                        connected = true;
                    } else {
                        if is_steam {
                            // Forward message to client
                            if let Ok(mut req) = serde_json::from_str::<serde_json::Value>(&msg) {
                                if let Some(req) = req.as_object_mut() {
                                    // Get the right client from message ID
                                    if let Some(id) = req.get("message_id").and_then(|id| id.as_u64().map(|id| id as u32)) {
                                        req.remove("message_id");

                                        if let Some(tx) = ctx.message_senders.read().await.get(&id) {
                                            if let Err(e) = serde_json::to_string(req).and_then(|msg| Ok(tx.send(msg))) {
                                                log::error!("Error sending message to client: {e}");
                                            }
                                        }
                                    }
                                }
                            } else {
                                log::error!("Received invalid message from Steam: {msg}");
                            }
                        } else {
                            // Forward message to Steam
                            let steam_tx = ctx.steam_tx.read().await;

                            if let Some(steam_tx) = steam_tx.as_ref() {
                                if let Ok(mut req) = serde_json::from_str::<RpcRequest>(&msg) {
                                    req.secret = None;
                                    req.message_id = Some(ctx.last_message_id.fetch_add(1, Ordering::Relaxed));

                                    let req = serde_json::to_string(&req).expect("failed to serialize message");

                                    log::debug!("Forwarding request: {req}");

                                    if let Err(e) = steam_tx.send(req) {
                                        log::error!("Error sending message to Steam: {e}");
                                    }
                                } else {
                                    log::warn!("Received invalid message: {msg}");
                                    send_message(&mut ws_stream, json!({
                                        "success": false,
                                        "error": "Message is not valid",
                                    }).as_str().expect("failed to serialize error message")).await;
                                }
                            } else {
                                log::warn!("Received command before connecting to Steam");
                                send_message(&mut ws_stream, json!({
                                    "success": false,
                                    "error": "Not connected to Steam",
                                }).as_str().expect("failed to serialize error message")).await;
                            }
                        }
                    }
                }
            }
        },
        res = rx.recv() => {
            if let Some(msg) = res {
                log::debug!("Received message from other handler: {msg}");
            }
        }
    }
}

async fn send_message(stream: &mut WebSocketStream<TcpStream>, msg: &str) {
    if let Err(e) = stream.send(Message::from(msg)).await {
        log::warn!("Failed to send message: {}", e);
    }
}

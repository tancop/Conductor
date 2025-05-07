use crate::message::RpcRequest;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};

struct Context {
    connected: AtomicBool,
    steam_tx: RwLock<Option<UnboundedSender<String>>>,
    last_message_id: AtomicU32,
    message_senders: RwLock<HashMap<u32, UnboundedSender<String>>>,
}

pub async fn serve(addr: String) {
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
                            handle_steam_message(ctx.clone(), &msg_text).await;
                        } else {
                            log::debug!("Received client message: '{}'", msg_text);
                            handle_client_message(ctx.clone(), &msg_text, &mut ws_stream).await;
                        }
                    },
                    Some(Err(e)) => {
                        log::error!("WebSocket error: {}", e);
                        break;
                    },
                    None => {
                        log::debug!("WebSocket connection closed");
                        break;
                    },
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

async fn handle_client_message(
    ctx: Arc<Context>,
    msg: &Utf8Bytes,
    ws_stream: &mut WebSocketStream<TcpStream>,
) {
    let steam_tx = ctx.steam_tx.read().await;
    if let Some(steam_tx) = steam_tx.as_ref() {
        let Ok(mut req) = serde_json::from_str::<RpcRequest>(&msg) else {
            log::error!("Failed to deserialize client message: {msg}");
            return;
        };

        req.secret = Some("Secret!".to_string());
        req.message_id = Some(ctx.last_message_id.fetch_add(1, Ordering::Relaxed));

        let Ok(req) = serde_json::to_string(&req) else {
            log::error!("Failed to serialize client message");
            return;
        };

        if let Err(e) = steam_tx.send(req) {
            log::error!("Error sending message to Steam: {}", e);
            // Steam connection might be dead, reset the connection state
            ctx.connected.store(false, Ordering::Relaxed);
            return;
        }
    } else {
        log::warn!("Steam connection not available");
        send_message(
            ws_stream,
            &json!({
                "success": false,
                "error": "Not connected to Steam",
            }),
        )
        .await;
    }
}

async fn handle_steam_message(ctx: Arc<Context>, msg: &Utf8Bytes) {
    let Ok(mut req) = serde_json::from_str::<serde_json::Value>(&msg) else {
        log::error!("Failed to deserialize steam message");
        return;
    };

    let Some(req) = req.as_object_mut() else {
        log::error!("Steam message is not a JSON object");
        return;
    };

    let Some(id) = req
        .get("messageId")
        .and_then(|id| id.as_u64().map(|id| id as u32))
    else {
        log::error!("Steam message has no ID");
        return;
    };

    req.remove("messageId");

    let senders = ctx.message_senders.read().await;
    let Some(tx) = senders.get(&id) else {
        log::warn!("No client channel found for id {id}");
        return;
    };

    let Ok(msg) = serde_json::to_string(req) else {
        log::error!("Failed to serialize steam message");
        return;
    };

    if let Err(e) = tx.send(msg) {
        log::error!("Error sending message to client: {}", e);
    }
}

async fn send_message(stream: &mut WebSocketStream<TcpStream>, msg: &serde_json::Value) {
    if let Ok(msg_str) = serde_json::to_string(msg) {
        if let Err(e) = stream.send(Message::text(msg_str)).await {
            log::warn!("Failed to send message: {}", e);
        }
    }
}

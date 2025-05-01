use futures_util::{StreamExt, TryStreamExt};
use regex::Regex;
use std::fs::File;
use std::io::Read;
use tokio::{
    io::Error,
    net::{TcpListener, TcpStream},
};

mod inject;
mod payload;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    println!("Starting Conductor");

    let debugger_url = inject::try_get_debugger_url().await;

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

    if let Some(url) = debugger_url {
        log::debug!("Sending payload to URL: {url}");

        let payload = payload::make_payload(&payload, 7355, true, "Secret!".to_string());

        log::debug!("payload contents: {payload}");
    }

    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:7355".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", addr);

    let (write, read) = ws_stream.split();

    read.try_filter(|msg| std::future::ready(msg.is_text() || msg.is_binary()))
        .forward(write)
        .await
        .expect("Failed to forward messages")
}

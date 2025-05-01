use futures_util::StreamExt;
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
    log::info!("Starting Conductor");

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

    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:7355".to_string());

    // Create the event loop and TCP listener we'll accept connections on
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    log::info!("Listening on: {}", addr);

    // Inject payload into SteamWebHelper
    match inject::inject_payload(&debugger_url, &payload, 5).await {
        Ok(()) => {}
        Err(err) => {
            log::error!("{err}");
            std::process::exit(1);
        }
    }

    log::info!("Conductor initialized!");

    // Start server
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    log::debug!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    log::debug!("New WebSocket connection: {}", addr);

    let (_, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        if let Ok(msg) = msg {
            println!("Received message: {msg}");
        }
    }
}

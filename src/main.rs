use futures_util::StreamExt;
use log::LevelFilter;
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;
use tokio::{
    io::Error,
    net::{TcpListener, TcpStream},
};

mod inject;
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

    // Wait for keyboard interrupt
    tokio::signal::ctrl_c().await?;

    log::info!("Goodbye!");

    Ok(())
}

async fn start() {
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

async fn serve(addr: String) {
    // Create the event loop and TCP listener we'll accept connections on
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    log::info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }
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
            log::debug!("Received message: {msg}");
        }
    }
}

use crate::enable_cef::enable_cef_debugging;
use crate::secrets::generate_secret;
use clap::Parser;
use log::LevelFilter;
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;
use tokio::io::Error;

mod enable_cef;
mod inject;
mod message;
mod payload;
mod process;
mod secrets;
mod server;

#[derive(Parser)]
#[command(
    name = "conductor",
    about = "Conductor exposes an easy to use WebSockets API for controlling the Steam client."
)]
struct Args {
    /// Port used for opening connections on localhost. The Steam payload will always connect
    /// to `ws://localhost:[port]`.
    #[arg(short, long, default_value_t = 7355)]
    port: u16,
    /// Secret for client authentication. If this option is set all requests need to have a
    /// `secret` field with the provided value.
    #[arg(short, long)]
    secret: Option<String>,
    /// Hostname used for client connections. Defaults to `localhost:[port]`.
    #[arg(short, long)]
    address: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

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

    if args.secret.is_some() {
        log::info!("Authentication enabled");
    }

    match enable_cef_debugging() {
        Ok(_) => {}
        Err(e) => {
            log::error!("Failed to enable CEF debugging: {e}");
            return Err(e);
        }
    };

    // Spawn server task
    tokio::spawn(start(args.port, args.secret, args.address));

    // Wait for exit event
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
    }

    log::info!("Goodbye!");

    Ok(())
}

async fn start(port: u16, secret: Option<String>, address: Option<String>) {
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

    let steam_secret = generate_secret();

    // Setup payload with port and secret
    let payload = payload::make_payload(&payload, port, true, steam_secret.clone());

    // Start server
    let addr = address.unwrap_or(format!("localhost:{port}"));

    tokio::spawn(server::serve(addr, steam_secret, secret));

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

/*
 *  main.rs
 *  (c) 2025 Teodor Potancok
 *
 *  This Source Code Form is subject to the terms of the Mozilla Public
 *  License, v. 2.0. If a copy of the MPL was not distributed with this
 *  file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::config::Config;
use crate::enable_cef::enable_cef_debugging;
use crate::secrets::generate_secret;
use clap::Parser;
use log::LevelFilter;
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;
use tokio::io::Error;

mod config;
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
    about = "Conductor lets you control the Steam client over WebSockets."
)]
struct Args {
    /// Path to the settings file
    #[arg(short, long, default_value_t = String::from("./settings.toml"))]
    settings_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let settings_path = Args::parse().settings_path;

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

    let mut cfg = match Config::load(settings_path) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    log::info!("Starting Conductor...");

    if let Some(auth) = &mut cfg.auth {
        if auth.enabled {
            log::info!("Authentication enabled");

            match &mut auth.tokens {
                Some(vec) => {
                    if vec.is_empty() {
                        log::error!("No tokens found, add some or set `auth.enabled` to false");
                        std::process::exit(1);
                    }
                }
                None => {
                    log::error!("No tokens found, add some or set `auth.enabled` to false");
                    std::process::exit(1);
                }
            }
        }
    }

    match enable_cef_debugging() {
        Ok(_) => {}
        Err(e) => {
            log::error!("Failed to enable CEF debugging: {e}");
            return Err(e);
        }
    };

    // Spawn server task
    tokio::spawn(start(cfg));

    // Wait for exit event
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
    }

    log::info!("Goodbye!");

    Ok(())
}

async fn start(cfg: Config) {
    // Kill other instances if running
    if let Err(e) = process::kill_other_instances().await {
        log::error!("{}", e);
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    if let Err(e) = process::store_pid() {
        log::warn!("Failed to store PID: {}", e);
    }

    let Ok(current_exe) = std::env::current_exe() else {
        log::error!("Could not get current executable");
        std::process::exit(1);
    };

    // Find and open payload file
    let Some(current_dir) = current_exe.parent() else {
        log::error!("Could not get current directory");
        std::process::exit(1);
    };

    let Ok(mut js_file) = File::open(current_dir.join(cfg.conductor.payload_path)) else {
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
    let payload = payload::make_payload(
        &payload,
        &cfg.conductor.hostname,
        true,
        steam_secret.clone(),
    );

    // Start server
    tokio::spawn(server::serve(
        cfg.conductor.hostname,
        steam_secret,
        cfg.auth,
    ));

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

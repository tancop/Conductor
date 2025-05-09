/*
 *  config.rs
 *  (c) 2025 Teodor Potancok
 *
 *  This Source Code Form is subject to the terms of the Mozilla Public
 *  License, v. 2.0. If a copy of the MPL was not distributed with this
 *  file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub tokens: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct MainConfig {
    pub hostname: String,
    pub payload_path: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub conductor: MainConfig,
    pub auth: Option<AuthConfig>,
}

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("Error reading file: {0}")]
    ReadFile(std::io::Error),
    #[error("Error deserializing file: {}", .0.message())]
    Deserialize(toml::de::Error),
}

impl Config {
    pub fn load(path: String) -> Result<Self, LoadError> {
        let file = match std::fs::read_to_string(path) {
            Ok(f) => f,
            Err(e) => return Err(LoadError::ReadFile(e)),
        };
        let cfg = match toml::from_str::<Config>(&file) {
            Ok(f) => f,
            Err(e) => return Err(LoadError::Deserialize(e)),
        };

        Ok(cfg)
    }
}

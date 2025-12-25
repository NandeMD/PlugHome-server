//! Configuration helpers shared across crates.

use std::env;

use anyhow::{Context, Result};

/// Load environment variables from a local `.env` file if it exists.
///
/// Call this once near startup before reading configuration.
pub fn load_env() {
    let _ = dotenvy::dotenv();
}

/// Server address/port configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub addr: String,
    pub port: String,
}

impl ServerConfig {
    /// Build `ServerConfig` from environment variables.
    ///
    /// Required:
    /// - `ADDR`
    /// - `PORT`
    pub fn from_env() -> Result<Self> {
        let addr = env::var("ADDR").context("ADDR must be set")?;
        let port = env::var("PORT").context("PORT must be set")?;

        Ok(Self { addr, port })
    }

    /// Combined host:port string.
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.addr, self.port)
    }
}

/// List of allowed charger serial numbers parsed from `ALLOWED_SERIAL_NUMBERS`.
/// Comma-separated; empty or missing disables the check.
pub fn allowed_serial_numbers() -> Result<Vec<String>> {
    let raw = env::var("ALLOWED_SERIAL_NUMBERS").unwrap_or_default();
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    let serials = raw
        .split(',')
        .filter_map(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect();
    Ok(serials)
}

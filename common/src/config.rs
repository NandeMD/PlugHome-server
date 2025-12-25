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

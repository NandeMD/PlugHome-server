//! Logging utilities shared across crates.

use anyhow::{Result, anyhow};
use tracing_subscriber::EnvFilter;

/// Initialize tracing with an env-controlled filter.
///
/// If `RUST_LOG` (or `LOG_LEVEL` for EnvFilter) is absent, falls back to the
/// provided `default_level` (e.g., `"info"`).
pub fn init_tracing(default_level: &str) -> Result<()> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(true)
        .try_init()
        .map_err(|err| anyhow!(err))?;

    Ok(())
}

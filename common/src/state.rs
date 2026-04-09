use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::OnceCell;
use tracing::warn;

use crate::{ServerConfig, allowed_serial_numbers};

pub static START_TIME: OnceCell<DateTime<Utc>> = OnceCell::const_new();
pub static ALLOWED_SERIAL_NUMBERS: OnceCell<Vec<String>> = OnceCell::const_new();

pub struct AppState<DB> {
    pub db: Arc<DB>,
    pub config: Arc<ServerConfig>,
}

impl<DB> Clone for AppState<DB> {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            config: Arc::clone(&self.config),
        }
    }
}

pub fn get_allowed_serial_numbers() -> Option<&'static Vec<String>> {
    ALLOWED_SERIAL_NUMBERS.get()
}

pub async fn load_allowed_serial_numbers() -> &'static Vec<String> {
    ALLOWED_SERIAL_NUMBERS
        .get_or_init(|| async {
            match allowed_serial_numbers() {
                Ok(list) => list,
                Err(err) => {
                    warn!("Failed to load ALLOWED_SERIAL_NUMBERS: {err}");
                    Vec::new()
                }
            }
        })
        .await
}

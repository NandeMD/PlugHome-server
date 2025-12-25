use chrono::{DateTime, Utc};
use common::allowed_serial_numbers;
use tokio::sync::OnceCell;
use tracing::warn;

pub static START_TIME: OnceCell<DateTime<Utc>> = OnceCell::const_new();
pub static ALLOWED_SERIAL_NUMBERS: OnceCell<Vec<String>> = OnceCell::const_new();

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

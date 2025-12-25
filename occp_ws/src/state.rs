use chrono::{DateTime, Utc};
use tokio::sync::OnceCell;

pub static START_TIME: OnceCell<DateTime<Utc>> = OnceCell::const_new();

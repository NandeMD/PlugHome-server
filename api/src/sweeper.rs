use db::Db;
use std::{sync::Arc, time::Duration};
use tokio::task::JoinHandle;
use tokio::time::sleep as tk_sleep;
use tracing::{info, warn};

pub async fn start_sweeper(db: Arc<Db>) -> JoinHandle<impl Send + 'static> {
    tokio::spawn(async move { sweeper(db).await })
}

// Should this return a `Result`?
// Idk, errors in this are not trivial (i hope so)
// So just logging will suffice
async fn sweeper(db: Arc<Db>) {
    loop {
        tk_sleep(Duration::from_secs(60)).await;

        match db.sweep().await {
            Ok(sweeped) => {
                for s in sweeped {
                    info!("Station `{s}` marked as offline due to timeout.");
                }
            }
            Err(e) => {
                warn!("An error occured while sweeping station timeouts! Error: {e}");
            }
        }
    }
}

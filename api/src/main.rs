use std::{net::SocketAddr, panic};

use anyhow::{Context, Result};
use axum::{Router, routing::get};
use chrono::{DateTime, Utc};
use tokio::net;
use tower_http::trace::TraceLayer;
use tracing::info;

use common::{ServerConfig, init_tracing, load_env};

use occp_ws::routes::{healthcheck_route, upgrade_to_ws};
use occp_ws::state::START_TIME;

async fn run() -> Result<()> {
    async fn time_now() -> DateTime<Utc> {
        Utc::now()
    }
    let _time_now = START_TIME.get_or_init(time_now).await;

    load_env();

    init_tracing("info")?;

    panic::set_hook(Box::new(|err| {
        tracing::error!("\n\nPanic: {err:#?}\n\n");
    }));

    let config = ServerConfig::from_env()?;
    let tcp_listener = net::TcpListener::bind(config.socket_addr())
        .await
        .with_context(|| format!("Failed to bind to address: {}", config.socket_addr()))?;
    info!("Server listening on {}", config.socket_addr());

    let router = Router::new()
        .route("/:station_id", get(upgrade_to_ws))
        .route("/", get(healthcheck_route))
        .layer(TraceLayer::new_for_http());

    axum::serve(
        tcp_listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .with_context(|| format!("Failed to start server on {}", config.socket_addr()))?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}

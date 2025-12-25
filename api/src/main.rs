use std::{env, net::SocketAddr, panic};

use anyhow::{Context, Result};
use axum::{Router, routing::get};
use chrono::{DateTime, Utc};
use dotenvy::dotenv;
use tokio::net;
use tracing::{Level, info};

use occp_ws::routes::{healthcheck_route, upgrade_to_ws};
use occp_ws::state::START_TIME;

async fn run() -> Result<()> {
    async fn time_now() -> DateTime<Utc> {
        Utc::now()
    }
    let _time_now = START_TIME.get_or_init(time_now).await;

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_ansi(true)
        .init();

    panic::set_hook(Box::new(|err| {
        tracing::error!("\n\nPanic: {err:#?}\n\n");
    }));

    dotenv().ok();

    let addr = env::var("ADDR").context("ADDR must be set")?;
    let port = env::var("PORT").context("PORT must be set")?;
    let tcp_listener = net::TcpListener::bind(format!("{addr}:{port}"))
        .await
        .with_context(|| format!("Failed to bind to address: {addr}:{port}"))?;
    info!("Server listening on {addr}:{port}");

    let router = Router::new()
        .route("/:station_id", get(upgrade_to_ws))
        .route("/", get(healthcheck_route));

    axum::serve(
        tcp_listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .with_context(|| format!("Failed to start server on {addr}:{port}"))?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}

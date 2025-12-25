use std::{env, net::SocketAddr, panic};

use anyhow::{Context, Result};
use axum::{Router, routing::get};
use chrono::{DateTime, Utc};
use dotenvy::dotenv;
use tokio::net;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

use occp_ws::routes::{healthcheck_route, upgrade_to_ws};
use occp_ws::state::START_TIME;

async fn run() -> Result<()> {
    async fn time_now() -> DateTime<Utc> {
        Utc::now()
    }
    let _time_now = START_TIME.get_or_init(time_now).await;

    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_ansi(true)
        .init();

    panic::set_hook(Box::new(|err| {
        tracing::error!("\n\nPanic: {err:#?}\n\n");
    }));

    let addr = env::var("ADDR").context("ADDR must be set")?;
    let port = env::var("PORT").context("PORT must be set")?;
    let tcp_listener = net::TcpListener::bind(format!("{addr}:{port}"))
        .await
        .with_context(|| format!("Failed to bind to address: {addr}:{port}"))?;
    info!("Server listening on {addr}:{port}");

    let router = Router::new()
        .route("/:station_id", get(upgrade_to_ws))
        .route("/", get(healthcheck_route))
        .layer(TraceLayer::new_for_http());

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

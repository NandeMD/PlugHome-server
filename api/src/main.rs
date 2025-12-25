use std::{env, net::SocketAddr, panic};

use axum::{Router, routing::get};
use chrono::Utc;
use dotenvy::dotenv;
use tokio::net;
use tracing::{Level, info};

use occp_ws::routes::{healthcheck_route, upgrade_to_ws};
use occp_ws::state::TIME_NOW;

async fn run() {
    async fn time_now() -> String {
        let date_time = Utc::now();
        format!("{}", date_time.format("%d/%m/%Y %H:%M"))
    }
    let _time_now = TIME_NOW.get_or_init(time_now).await;

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_ansi(true)
        .init();

    panic::set_hook(Box::new(|err| {
        tracing::error!("\n\nPanic: {err:#?}\n\n");
    }));

    dotenv().ok();

    let addr = env::var("ADDR").expect("ADDR must be set");
    let port = env::var("PORT").expect("PORT must be set");
    let tcp_listener = net::TcpListener::bind(format!("{addr}:{port}"))
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to address: {addr}"));
    info!("Server listening on {addr}:{port}");

    let router = Router::new()
        .route("/:station_id", get(upgrade_to_ws))
        .route("/", get(healthcheck_route));

    axum::serve(
        tcp_listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Failed to start server");
}

#[tokio::main]
async fn main() {
    run().await;
}

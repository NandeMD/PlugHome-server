use std::{net::SocketAddr, panic};

use axum::{routing::get, Router};
use chrono::Utc;
use dotenvy_macro::dotenv;
use tokio::net;
use tracing::{info, Level};

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

    const ADDR: &str = dotenv!("ADDR");
    const PORT: &str = dotenv!("PORT");
    let tcp_listener = net::TcpListener::bind(format!("{ADDR}:{PORT}"))
        .await
        .expect(&format!("Failed to bind to address: {ADDR}"));
    info!("Server listening on {ADDR}:{PORT}");

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

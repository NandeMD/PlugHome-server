use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, ws::WebSocketUpgrade},
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use tracing::{info, warn};

use crate::handlers::handle_socket;
use crate::state::TIME_NOW;

pub async fn upgrade_to_ws(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    match user_agent {
        Some(TypedHeader(agent)) => {
            if agent.as_str() == "Websocket Client" {
                info!("{agent} user agent is a valid client");
            } else {
                warn!("User agent {agent} is not a valid Websocket Client");
            }
        }
        None => warn!("User agent is not present. Continue without specific platform check"),
    }

    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

pub async fn healthcheck_route() -> impl IntoResponse {
    if let Some(time) = TIME_NOW.get() {
        (
            axum::http::StatusCode::OK,
            [(axum::http::header::CACHE_CONTROL, "public, max-age=60")],
            axum::Json(serde_json::json!({
                "status": "ok",
                "started_at": time,
            })),
        )
    } else {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            [(axum::http::header::CACHE_CONTROL, "no-store")],
            axum::Json(serde_json::json!({
                "status": "unavailable",
                "message": "Server has not started yet",
            })),
        )
    }
}

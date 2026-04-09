use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{ConnectInfo, Path, State, ws::WebSocketUpgrade},
    response::IntoResponse,
};
use axum_extra::TypedHeader;

use crate::handlers::handle_socket;
use crate::state::{AppState, START_TIME};

pub async fn upgrade_to_ws(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
    Path(station_id): Path<String>,
    _user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let db = state.db;
    let conf = Arc::clone(&state.config);
    ws.on_upgrade(move |socket| handle_socket(socket, addr, station_id, db, conf))
}

pub async fn healthcheck_route() -> impl IntoResponse {
    if let Some(time) = START_TIME.get() {
        (
            axum::http::StatusCode::OK,
            [(axum::http::header::CACHE_CONTROL, "public, max-age=60")],
            axum::Json(serde_json::json!({
                "status": "ok",
                "started_at": time.to_rfc3339(),
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

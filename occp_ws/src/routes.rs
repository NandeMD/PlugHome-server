use std::net::SocketAddr;

use axum::{
    extract::{ws::WebSocketUpgrade, ConnectInfo},
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
        },
        None => warn!("User agent is not present. Continue without specific platform check"),
    }

    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

pub async fn healthcheck_route() -> impl IntoResponse {
    if let Some(time) = TIME_NOW.get() {
        axum::response::Html::from(format!("<h1>Server working. Started at: {time}</h1>"))
    } else {
        axum::response::Html::from(format!("<h1>Server has not started yet</h1>"))
    }
}

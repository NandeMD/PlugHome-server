use std::{
    error::Error,
    io::{self, ErrorKind},
    net::SocketAddr,
    time::Duration,
};

use axum::{Router, routing::get};
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use occp_ws::routes::{healthcheck_route, upgrade_to_ws};
use occp_ws::state::START_TIME;
use occp_ws::types::*;
use rust_ocpp::v1_6::messages::boot_notification::BootNotificationRequest;
use rust_ocpp::v1_6::messages::heart_beat::HeartbeatRequest;
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle, time::timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing_subscriber::EnvFilter;

async fn start_test_server() -> (SocketAddr, oneshot::Sender<()>, JoinHandle<()>) {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_test_writer()
        .try_init();

    START_TIME.get_or_init(|| async { Utc::now() }).await;

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let addr = listener.local_addr().expect("listener addr");

    let router = Router::new()
        .route("/:station_id", get(upgrade_to_ws))
        .route("/", get(healthcheck_route));

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let server = axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = shutdown_rx.await;
    });

    let handle = tokio::spawn(async move {
        if let Err(err) = server.await {
            panic!("test server error: {err}");
        }
    });

    (addr, shutdown_tx, handle)
}

async fn recv_text_within(
    socket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    dur: Duration,
) -> Result<String, Box<dyn Error>> {
    loop {
        let msg = timeout(dur, socket.next())
            .await
            .map_err(|_| io::Error::new(ErrorKind::TimedOut, "timeout waiting for text frame"))?
            .ok_or_else(|| io::Error::new(ErrorKind::UnexpectedEof, "websocket closed"))??;

        match msg {
            WsMessage::Text(body) => return Ok(body),
            WsMessage::Ping(_) | WsMessage::Pong(_) => continue,
            WsMessage::Close(frame) => {
                return Err(Box::new(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    format!("received close frame: {frame:?}"),
                )));
            }
            other => {
                return Err(Box::new(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("unexpected ws message: {other:?}"),
                )));
            }
        }
    }
}

#[tokio::test]
async fn handles_heartbeat_call_over_websocket() -> Result<(), Box<dyn Error>> {
    let (addr, shutdown, server) = start_test_server().await;

    let url = format!("ws://{addr}/station-123");
    let (mut socket, _) = connect_async(&url).await?;

    // Perform boot notification first to mimic realistic startup flow
    let boot_payload =
        OcppPayload::BootNotification(BootNotificationKind::Request(BootNotificationRequest {
            charge_point_model: "ModelX".to_string(),
            charge_point_vendor: "AcmeCorp".to_string(),
            charge_point_serial_number: Some("SN-hb-123".to_string()),
            ..Default::default()
        }));
    let boot_call = OcppCall(
        2,
        "boot-before-heartbeat".to_string(),
        OcppActionEnum::BootNotification,
        boot_payload,
    );
    let boot_frame = serde_json::to_string(&boot_call)?;
    socket.send(WsMessage::Text(boot_frame)).await?;

    let boot_response = timeout(Duration::from_secs(5), socket.next())
        .await
        .expect("boot notification should respond within timeout")
        .ok_or_else(|| io::Error::new(ErrorKind::UnexpectedEof, "no boot response"))?;
    let boot_response = boot_response?;
    let boot_text = match boot_response {
        WsMessage::Text(body) => body,
        other => panic!("unexpected websocket message during boot: {other:?}"),
    };
    let parsed_boot: OcppMessageType = serde_json::from_str(&boot_text)?;
    match parsed_boot {
        OcppMessageType::CallResult(mt, id, _) => {
            assert_eq!(mt, 3);
            assert_eq!(id, "boot-before-heartbeat");
        }
        other => panic!("unexpected boot response: {other:?}"),
    }

    let message_id = "hb-1".to_string();
    let payload = OcppPayload::Heartbeat(HeartbeatKind::Request(HeartbeatRequest {}));
    let call = OcppCall(2, message_id.clone(), OcppActionEnum::Heartbeat, payload);
    let frame = serde_json::to_string(&call)?;

    socket.send(WsMessage::Text(frame)).await?;

    let text = recv_text_within(&mut socket, Duration::from_secs(5)).await?;

    let parsed: OcppMessageType = serde_json::from_str(&text)?;
    match parsed {
        OcppMessageType::CallResult(message_type, id, payload) => {
            assert_eq!(message_type, 3);
            assert_eq!(id, message_id);

            let current_time = payload
                .get("currentTime")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    io::Error::new(
                        ErrorKind::InvalidData,
                        "heartbeat response missing currentTime",
                    )
                })?;
            assert!(!current_time.is_empty());
        }
        other => panic!("unexpected message type: {other:?}"),
    }

    socket.close(None).await?;

    shutdown.send(()).ok();
    server.await.expect("server task panicked");

    Ok(())
}

#[tokio::test]
async fn handles_ping_pong_and_close() -> Result<(), Box<dyn Error>> {
    let (addr, shutdown, server) = start_test_server().await;

    let url = format!("ws://{addr}/station-789");
    let (mut socket, _) = connect_async(&url).await?;

    // Send ping and expect pong
    socket.send(WsMessage::Ping(b"hello".to_vec())).await?;
    let pong = timeout(Duration::from_secs(2), socket.next())
        .await
        .expect("pong within timeout")
        .ok_or_else(|| io::Error::new(ErrorKind::UnexpectedEof, "no pong"))??;
    match pong {
        WsMessage::Pong(payload) => assert_eq!(payload, b"hello"),
        other => panic!("expected pong, got {:?}", other),
    }

    // Send a happy-path heartbeat call and expect CallResult
    let message_id = "hb-integration".to_string();
    let payload = OcppPayload::Heartbeat(HeartbeatKind::Request(HeartbeatRequest {}));
    let call = OcppCall(2, message_id.clone(), OcppActionEnum::Heartbeat, payload);
    let frame = serde_json::to_string(&call)?;
    socket.send(WsMessage::Text(frame)).await?;

    let text = recv_text_within(&mut socket, Duration::from_secs(5)).await?;

    let parsed: OcppMessageType = serde_json::from_str(&text)?;
    match parsed {
        OcppMessageType::CallResult(message_type, id, payload) => {
            assert_eq!(message_type, 3);
            assert_eq!(id, message_id);
            assert!(
                payload
                    .get("currentTime")
                    .and_then(|v| v.as_str())
                    .is_some()
            );
        }
        other => panic!("unexpected heartbeat response: {:?}", other),
    }

    // Close should be accepted gracefully
    socket.close(None).await?;

    shutdown.send(()).ok();
    server.await.expect("server task panicked");

    Ok(())
}

#[tokio::test]
async fn accepts_boot_notification_call_over_websocket() -> Result<(), Box<dyn Error>> {
    let (addr, shutdown, server) = start_test_server().await;

    let url = format!("ws://{addr}/station-456");
    let (mut socket, _) = connect_async(&url).await?;

    let message_id = "boot-1".to_string();
    let payload =
        OcppPayload::BootNotification(BootNotificationKind::Request(BootNotificationRequest {
            charge_point_model: "ModelX".to_string(),
            charge_point_vendor: "AcmeCorp".to_string(),
            charge_point_serial_number: Some("SN-12345".to_string()),
            ..Default::default()
        }));
    let call = OcppCall(
        2,
        message_id.clone(),
        OcppActionEnum::BootNotification,
        payload,
    );
    let frame = serde_json::to_string(&call)?;

    socket.send(WsMessage::Text(frame)).await?;

    let response = timeout(Duration::from_secs(5), socket.next())
        .await
        .expect("server should respond to boot notification within timeout")
        .ok_or_else(|| io::Error::new(ErrorKind::UnexpectedEof, "no boot response"))?;
    let response = response?;

    let text = match response {
        WsMessage::Text(body) => body,
        other => panic!("unexpected websocket message: {other:?}"),
    };

    let parsed: OcppMessageType = serde_json::from_str(&text)?;
    match parsed {
        OcppMessageType::CallResult(message_type, id, payload) => {
            assert_eq!(message_type, 3);
            assert_eq!(id, message_id);

            let status = payload
                .get("status")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    io::Error::new(ErrorKind::InvalidData, "boot response missing status")
                })?;
            assert_eq!(status, "Accepted");

            let interval = payload
                .get("interval")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| {
                    io::Error::new(ErrorKind::InvalidData, "boot response missing interval")
                })?;
            assert!(interval > 0);
        }
        other => panic!("unexpected message type: {other:?}"),
    }

    socket.close(None).await?;

    shutdown.send(()).ok();
    server.await.expect("server task panicked");

    Ok(())
}

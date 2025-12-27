use std::{net::SocketAddr, str::FromStr};

use axum::extract::ws::{Message as AxumWSMessage, WebSocket};
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use rust_ocpp::v1_6::messages::{
    authorize::AuthorizeResponse, boot_notification::BootNotificationResponse,
    data_transfer::DataTransferResponse, heart_beat::HeartbeatResponse,
    stop_transaction::StopTransactionResponse,
};
use serde::Serialize;
use serde_json::json;
use tokio::{
    select,
    sync::{mpsc, oneshot},
};
use tracing::{debug, error, info, warn};

use crate::state::load_allowed_serial_numbers;
use crate::types::*;

// OCPP 1.6 JSON framing message type identifiers
const CALL_MESSAGE_TYPE_ID: OcppMessageTypeId = 2;
const CALL_RESULT_MESSAGE_TYPE_ID: OcppMessageTypeId = 3;
const CALL_ERROR_MESSAGE_TYPE_ID: OcppMessageTypeId = 4;

enum OcppOutcome {
    Continue(Vec<AxumWSMessage>),
    Close(Vec<AxumWSMessage>),
}

pub async fn handle_socket(socket: WebSocket, addr: SocketAddr) {
    info!(addr = %addr, "New WebSocket connection: {addr}");

    let (mut ws_tx, mut ws_rx) = socket.split();
    let (out_tx, mut out_rx) = mpsc::channel::<AxumWSMessage>(64);
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

    let reader = {
        let out_tx = out_tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = ws_rx.next().await {
                let msg = match msg {
                    Ok(m) => m,
                    Err(err) => {
                        warn!(addr = %addr, "WebSocket read error: {err}");
                        break;
                    }
                };

                match msg {
                    AxumWSMessage::Text(text) => {
                        info!("\nINCOMING CALL\nFROM CHARGER\n\tMessage: {text}\n\tAddr: {addr}\n");
                        let outcome = handle_ocpp_messages(text).await;
                        let (outgoing, should_close) = match outcome {
                            OcppOutcome::Continue(out) => (out, false),
                            OcppOutcome::Close(out) => (out, true),
                        };

                        if !send_outgoing(&out_tx, outgoing).await {
                            break;
                        }
                        if should_close {
                            break;
                        }
                    }
                    AxumWSMessage::Binary(_) => warn!("Unexpected binary message"),
                    AxumWSMessage::Ping(payload) => {
                        // Reply with a Pong frame carrying the same payload
                        if out_tx.send(AxumWSMessage::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    AxumWSMessage::Pong(_) => debug!("Received WebSocket Pong"),
                    AxumWSMessage::Close(frame) => {
                        let _ = out_tx.send(AxumWSMessage::Close(frame)).await;
                        break;
                    }
                }
            }
        })
    };

    let writer = tokio::spawn(async move {
        loop {
            select! {
                biased;

                _ = &mut shutdown_rx => break,

                maybe_msg = out_rx.recv() => {
                    match maybe_msg {
                        Some(msg) => {
                            if let Err(err) = ws_tx.send(msg).await {
                                warn!(addr = %addr, "WebSocket write error: {err}");
                                break;
                            }
                        }
                        None => break, // all senders dropped
                    }
                }
            }
        }
    });

    let _ = reader.await;
    let _ = shutdown_tx.send(());
    let _ = writer.await;

    info!(addr = %addr, "WebSocket connection closed");
}

async fn send_outgoing(out_tx: &mpsc::Sender<AxumWSMessage>, outgoing: Vec<AxumWSMessage>) -> bool {
    for msg in outgoing {
        if out_tx.send(msg).await.is_err() {
            return false;
        }
    }
    true
}

async fn handle_ocpp_messages(message: String) -> OcppOutcome {
    match serde_json::from_str(&message) {
        Ok(ocpp_message) => match ocpp_message {
            OcppMessageType::Call(message_type_id, message_id, action, payload) => {
                if message_type_id != CALL_MESSAGE_TYPE_ID {
                    warn!(
                        expected = CALL_MESSAGE_TYPE_ID,
                        received = message_type_id,
                        "Invalid MessageTypeId for Call"
                    );
                    let mut outgoing = Vec::new();
                    handle_ocpp_call_error(
                        CALL_ERROR_MESSAGE_TYPE_ID,
                        message_id,
                        "ProtocolError".to_string(),
                        "Invalid MessageTypeId for Call".to_string(),
                        json!({ "expected": CALL_MESSAGE_TYPE_ID, "received": message_type_id }),
                        &mut outgoing,
                    )
                    .await;
                    return OcppOutcome::Continue(outgoing);
                }

                let action = match OcppActionEnum::from_str(&action) {
                    Ok(action) => {
                        debug!("Parsed OCPP call action: {action:?}");
                        action
                    }
                    Err(err) => {
                        error!("Failed to parse OCPP Call Action: {err:?}");
                        let mut outgoing = Vec::new();
                        handle_ocpp_call_error(
                            CALL_ERROR_MESSAGE_TYPE_ID,
                            message_id,
                            "NotSupported".to_string(),
                            format!("Unknown action: {action}"),
                            json!({ "action": action, "reason": err.to_string() }),
                            &mut outgoing,
                        )
                        .await;
                        return OcppOutcome::Continue(outgoing);
                    }
                };
                handle_ocpp_call(message_id, action, payload).await
            }
            OcppMessageType::CallResult(message_type_id, _message_id, payload) => {
                if message_type_id != CALL_RESULT_MESSAGE_TYPE_ID {
                    warn!(
                        expected = CALL_RESULT_MESSAGE_TYPE_ID,
                        received = message_type_id,
                        "Invalid MessageTypeId for CallResult"
                    );
                }
                handle_ocpp_call_result(payload).await;
                OcppOutcome::Continue(Vec::new())
            }
            OcppMessageType::CallError(
                message_type_id,
                message_id,
                error_code,
                error_description,
                error_details,
            ) => {
                if message_type_id != CALL_ERROR_MESSAGE_TYPE_ID {
                    warn!(
                        expected = CALL_ERROR_MESSAGE_TYPE_ID,
                        received = message_type_id,
                        "Invalid MessageTypeId for CallError"
                    );
                }
                handle_ocpp_call_error(
                    message_type_id,
                    message_id,
                    error_code,
                    error_description,
                    error_details,
                    &mut Vec::new(),
                )
                .await;
                OcppOutcome::Continue(Vec::new())
            }
        },
        Err(err) => {
            warn!("Failed to parse OCPP message: {err:?}");
            OcppOutcome::Continue(Vec::new())
        }
    }
}

async fn handle_ocpp_call(
    message_id: OcppMessageId,
    action: OcppActionEnum,
    payload: serde_json::Value,
) -> OcppOutcome {
    let payload = match serde_json::from_value::<OcppPayload>(payload) {
        Ok(ocpp_payload) => ocpp_payload,
        Err(err) => {
            error!("Failed to parse OCPP Payload: {err:?}");
            return OcppOutcome::Continue(Vec::new());
        }
    };

    use OcppActionEnum::*;
    let mut outgoing = Vec::new();

    match action {
        Authorize => {
            if let OcppPayload::Authorize(AuthorizeKind::Request(authorize)) = payload {
                info!("CALL REQUEST:\n{authorize:#?}");
                let response = OcppCallResult(
                    CALL_RESULT_MESSAGE_TYPE_ID,
                    message_id,
                    OcppPayload::Authorize(AuthorizeKind::Response(AuthorizeResponse {
                        id_tag_info: rust_ocpp::v1_6::types::IdTagInfo {
                            status: rust_ocpp::v1_6::types::AuthorizationStatus::Accepted,
                            expiry_date: None,
                            parent_id_tag: None,
                        },
                    })),
                );
                push_json(&response, &mut outgoing, "Authorize response");
            }
            OcppOutcome::Continue(outgoing)
        }
        BootNotification => {
            if let OcppPayload::BootNotification(BootNotificationKind::Request(boot_notification)) =
                payload
            {
                let allowed_serials = load_allowed_serial_numbers().await;
                let serial = boot_notification.charge_point_serial_number.clone();
                let serial_is_allowed = if allowed_serials.is_empty() {
                    true
                } else {
                    serial
                        .as_ref()
                        .map(|s| allowed_serials.iter().any(|allowed| allowed == s))
                        .unwrap_or(false)
                };

                if serial_is_allowed {
                    info!("CALL REQUEST:\n{boot_notification:#?}");
                    let response = OcppCallResult(
                        CALL_RESULT_MESSAGE_TYPE_ID,
                        message_id,
                        OcppPayload::BootNotification(BootNotificationKind::Response(
                            BootNotificationResponse {
                                status: rust_ocpp::v1_6::types::RegistrationStatus::Accepted,
                                current_time: Utc::now(),
                                interval: 300,
                            },
                        )),
                    );
                    push_json(&response, &mut outgoing, "BootNotification response");
                    OcppOutcome::Continue(outgoing)
                } else {
                    error!(
                        "Invalid Charger Serial Number. BootNotification: \
                             {boot_notification:?}"
                    );
                    outgoing.push(AxumWSMessage::Close(None));
                    OcppOutcome::Close(outgoing)
                }
            } else {
                error!("Invalid OCPP BootNotification payload");
                OcppOutcome::Continue(outgoing)
            }
        }
        DataTransfer => {
            if let OcppPayload::DataTransfer(DataTransferKind::Request(data_transfer)) = payload {
                info!("CALL REQUEST:\n{data_transfer:#?}");
                let response = OcppCallResult(
                    CALL_RESULT_MESSAGE_TYPE_ID,
                    message_id,
                    OcppPayload::DataTransfer(DataTransferKind::Response(DataTransferResponse {
                        status: rust_ocpp::v1_6::types::DataTransferStatus::Accepted,
                        data: Some("Data Transfer Accepted".to_string()),
                    })),
                );
                push_json(&response, &mut outgoing, "DataTransfer response");
            }
            OcppOutcome::Continue(outgoing)
        }
        Heartbeat => {
            if let OcppPayload::Heartbeat(HeartbeatKind::Request(heartbeat)) = &payload {
                info!("CALL REQUEST:\n{heartbeat:#?}");
            } else {
                warn!("Received Heartbeat action with unexpected payload: {payload:?}");
            }

            let response = OcppCallResult(
                CALL_RESULT_MESSAGE_TYPE_ID,
                message_id,
                OcppPayload::Heartbeat(HeartbeatKind::Response(HeartbeatResponse {
                    current_time: Utc::now(),
                })),
            );
            push_json(&response, &mut outgoing, "Heartbeat response");
            OcppOutcome::Continue(outgoing)
        }
        StatusNotification => {
            if let OcppPayload::StatusNotification(StatusNotificationKind::Request(
                status_notification,
            )) = payload
            {
                info!("CALL REQUEST:\n{status_notification:#?}");
            }
            OcppOutcome::Continue(outgoing)
        }
        StopTransaction => {
            if let OcppPayload::StopTransaction(StopTransactionKind::Request(stop_transaction)) =
                payload
            {
                info!("CALL REQUEST:\n{stop_transaction:#?}");
                let response = OcppCallResult(
                    CALL_RESULT_MESSAGE_TYPE_ID,
                    message_id,
                    OcppPayload::StopTransaction(StopTransactionKind::Response(
                        StopTransactionResponse {
                            id_tag_info: Some(rust_ocpp::v1_6::types::IdTagInfo {
                                status: rust_ocpp::v1_6::types::AuthorizationStatus::Accepted,
                                expiry_date: None,
                                parent_id_tag: None,
                            }),
                        },
                    )),
                );
                push_json(&response, &mut outgoing, "StopTransaction response");
            }
            OcppOutcome::Continue(outgoing)
        }
        _ => {
            warn!("OCPP action {action:?} not implemented");
            handle_ocpp_call_error(
                CALL_ERROR_MESSAGE_TYPE_ID,
                message_id,
                "NotSupported".to_string(),
                format!("Action {action:?} not implemented"),
                serde_json::json!({ "action": action.to_string() }),
                &mut outgoing,
            )
            .await;
            OcppOutcome::Continue(outgoing)
        }
    }
}

async fn handle_ocpp_call_result(payload: serde_json::Value) {
    match serde_json::from_value::<OcppPayload>(payload) {
        Ok(ocpp_payload) => {
            info!("Parsed OCPP Payload: {ocpp_payload:?}");
        }
        Err(err) => {
            warn!("Failed to parse OCPP Payload: {err:?}");
        }
    }
}

async fn handle_ocpp_call_error(
    message_type_id: OcppMessageTypeId,
    message_id: OcppMessageId,
    error_code: String,
    error_description: String,
    error_details: serde_json::Value,
    outgoing: &mut Vec<AxumWSMessage>,
) {
    let ocpp_call_error = OcppCallError(
        message_type_id,
        message_id,
        error_code,
        error_description,
        error_details,
    );
    push_json(&ocpp_call_error, outgoing, "OCPP CallError");
}

fn push_json<T: Serialize>(value: &T, outgoing: &mut Vec<AxumWSMessage>, label: &str) {
    match serde_json::to_string(value) {
        Ok(json) => {
            info!("{label}: {json}");
            outgoing.push(AxumWSMessage::Text(json));
        }
        Err(err) => warn!("Failed to serialize {label}: {err}"),
    }
}

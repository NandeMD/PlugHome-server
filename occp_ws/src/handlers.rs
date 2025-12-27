use std::{net::SocketAddr, str::FromStr};

use axum::extract::ws::{Message as AxumWSMessage, WebSocket};
use chrono::Utc;
use futures::StreamExt;
use rust_ocpp::v1_6::messages::{
    authorize::AuthorizeResponse, boot_notification::BootNotificationResponse,
    data_transfer::DataTransferResponse, heart_beat::HeartbeatResponse,
    stop_transaction::StopTransactionResponse,
};
use serde_json::json;
use tracing::{debug, error, info, warn};

use crate::state::load_allowed_serial_numbers;
use crate::types::*;

// OCPP 1.6 JSON framing message type identifiers
const CALL_MESSAGE_TYPE_ID: OcppMessageTypeId = 2;
const CALL_RESULT_MESSAGE_TYPE_ID: OcppMessageTypeId = 3;
const CALL_ERROR_MESSAGE_TYPE_ID: OcppMessageTypeId = 4;

pub async fn handle_socket(mut socket: WebSocket, addr: SocketAddr) {
    info!(addr = %addr, "New WebSocket connection: {addr}");

    loop {
        match socket.next().await {
            Some(Ok(msg)) => match msg {
                AxumWSMessage::Text(text) => {
                    let message = text.clone();
                    info!("\nINCOMING CALL\nFROM CHARGER\n\tMessage: {message}\n\tAddr: {addr}\n");
                    let should_continue = handle_ocpp_messages(text, &mut socket).await;
                    if !should_continue {
                        break;
                    }
                }
                AxumWSMessage::Binary(_) => warn!("Unexpected binary message"),
                AxumWSMessage::Ping(payload) => {
                    // Reply with a Pong frame carrying the same payload
                    let _ = socket.send(AxumWSMessage::Pong(payload)).await;
                }
                AxumWSMessage::Pong(_) => debug!("Received WebSocket Pong"),
                AxumWSMessage::Close(_) => {
                    info!("WebSocket connection closed");
                    if let Err(err) = socket.close().await {
                        warn!("Failed to close WebSocket connection: {err}");
                    }
                    break;
                }
            },
            Some(Err(err)) => {
                warn!(addr = %addr, "WebSocket stream error: {err}");
                break;
            }
            None => {
                info!(addr = %addr, "WebSocket stream disconnected");
                break;
            }
        }
    }
}

async fn handle_ocpp_messages(message: String, socket: &mut WebSocket) -> bool {
    match serde_json::from_str(&message) {
        Ok(ocpp_message) => match ocpp_message {
            OcppMessageType::Call(message_type_id, message_id, action, payload) => {
                if message_type_id != CALL_MESSAGE_TYPE_ID {
                    warn!(
                        expected = CALL_MESSAGE_TYPE_ID,
                        received = message_type_id,
                        "Invalid MessageTypeId for Call"
                    );
                    handle_ocpp_call_error(
                        CALL_ERROR_MESSAGE_TYPE_ID,
                        message_id,
                        "ProtocolError".to_string(),
                        "Invalid MessageTypeId for Call".to_string(),
                        json!({ "expected": CALL_MESSAGE_TYPE_ID, "received": message_type_id }),
                        socket,
                    )
                    .await;
                    return true;
                }

                let action = match OcppActionEnum::from_str(&action) {
                    Ok(action) => {
                        debug!("Parsed OCPP call action: {action:?}");
                        action
                    }
                    Err(err) => {
                        error!("Failed to parse OCPP Call Action: {err:?}");
                        handle_ocpp_call_error(
                            CALL_ERROR_MESSAGE_TYPE_ID,
                            message_id,
                            "NotSupported".to_string(),
                            format!("Unknown action: {action}"),
                            json!({ "action": action, "reason": err.to_string() }),
                            socket,
                        )
                        .await;
                        return true;
                    }
                };
                handle_ocpp_call(message_type_id, message_id, action, payload, socket).await
            }
            OcppMessageType::CallResult(message_type_id, message_id, payload) => {
                if message_type_id != CALL_RESULT_MESSAGE_TYPE_ID {
                    warn!(
                        expected = CALL_RESULT_MESSAGE_TYPE_ID,
                        received = message_type_id,
                        "Invalid MessageTypeId for CallResult"
                    );
                    return true;
                }
                handle_ocpp_call_result(message_type_id, message_id, payload, socket).await;
                true
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
                    socket,
                )
                .await;
                true
            }
        },
        Err(err) => {
            warn!("Failed to parse OCPP message: {err:?}");
            true
        }
    }
}

async fn handle_ocpp_call(
    _: OcppMessageTypeId,
    message_id: OcppMessageId,
    action: OcppActionEnum,
    payload: serde_json::Value,
    socket: &mut WebSocket,
) -> bool {
    let payload = match serde_json::from_value::<OcppPayload>(payload) {
        Ok(ocpp_payload) => ocpp_payload,
        Err(err) => {
            error!("Failed to parse OCPP Payload: {err:?}");
            return true;
        }
    };

    use OcppActionEnum::*;

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
                match serde_json::to_string(&response) {
                    Ok(response_json) => {
                        info!("CALL RESULT RESPONSE:\n{response_json}");
                        if let Err(err) = socket
                            .send(axum::extract::ws::Message::Text(response_json))
                            .await
                        {
                            warn!("Failed to send Authorize response: {err}");
                        }
                    }
                    Err(err) => warn!("Failed to serialize Authorize response: {err}"),
                }
            }
            true
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
                    match serde_json::to_string(&response) {
                        Ok(response_json) => {
                            info!("CALL RESULT RESPONSE:\n{response_json}");
                            if let Err(err) = socket
                                .send(axum::extract::ws::Message::Text(response_json))
                                .await
                            {
                                warn!("Failed to send BootNotification response: {err}");
                            }
                        }
                        Err(err) => warn!("Failed to serialize BootNotification response: {err}"),
                    }
                    true
                } else {
                    error!(
                        "Invalid Charger Serial Number. BootNotification: \
                             {boot_notification:?}"
                    );
                    if let Err(err) = socket.send(axum::extract::ws::Message::Close(None)).await {
                        warn!("Failed to send close frame: {err}");
                    }
                    false
                }
            } else {
                error!("Invalid OCPP BootNotification payload");
                true
            }
        }
        ChangeAvailability => true,
        ChangeConfiguration => true,
        ClearCache => true,
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
                match serde_json::to_string(&response) {
                    Ok(response_json) => {
                        info!("CALL RESULT RESPONSE:\n{response_json}");
                        if let Err(err) = socket
                            .send(axum::extract::ws::Message::Text(response_json))
                            .await
                        {
                            warn!("Failed to send DataTransfer response: {err}");
                        }
                    }
                    Err(err) => warn!("Failed to serialize DataTransfer response: {err}"),
                }
            }
            true
        }
        GetConfiguration => true,
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
            match serde_json::to_string(&response) {
                Ok(response_json) => {
                    info!("CALL RESULT RESPONSE:\n{response_json}");
                    if let Err(err) = socket
                        .send(axum::extract::ws::Message::Text(response_json))
                        .await
                    {
                        warn!("Failed to send Heartbeat response: {err}");
                    }
                }
                Err(err) => warn!("Failed to serialize Heartbeat response: {err}"),
            }
            true
        }
        MeterValues => true,
        RemoteStartTransaction => true,
        RemoteStopTransaction => true,
        Reset => true,
        StatusNotification => {
            if let OcppPayload::StatusNotification(StatusNotificationKind::Request(
                status_notification,
            )) = payload
            {
                info!("CALL REQUEST:\n{status_notification:#?}");
            }
            true
        }
        StartTransaction => true,
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
                match serde_json::to_string(&response) {
                    Ok(response_json) => {
                        info!("CALL RESULT RESPONSE:\n{response_json}");
                        if let Err(err) = socket
                            .send(axum::extract::ws::Message::Text(response_json))
                            .await
                        {
                            warn!("Failed to send StopTransaction response: {err}");
                        }
                    }
                    Err(err) => warn!("Failed to serialize StopTransaction response: {err}"),
                }
            }
            true
        }
        UnlockConnector => true,
        _ => {
            warn!("OCPP action {action:?} not implemented");
            handle_ocpp_call_error(
                CALL_ERROR_MESSAGE_TYPE_ID,
                message_id,
                "NotSupported".to_string(),
                format!("Action {action:?} not implemented"),
                serde_json::json!({ "action": action.to_string() }),
                socket,
            )
            .await;
            true
        }
    }
}

async fn handle_ocpp_call_result(
    _: OcppMessageTypeId,
    _: OcppMessageId,
    payload: serde_json::Value,
    _: &mut WebSocket,
) {
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
    socket: &mut WebSocket,
) {
    let ocpp_call_error = OcppCallError(
        message_type_id,
        message_id,
        error_code,
        error_description,
        error_details,
    );
    match serde_json::to_string(&ocpp_call_error) {
        Ok(ocpp_call_error_json) => {
            info!("Sending OCPP CallError: {ocpp_call_error_json}");
            if let Err(err) = socket
                .send(axum::extract::ws::Message::Text(ocpp_call_error_json))
                .await
            {
                warn!("Failed to send OCPP CallError: {err}");
            }
        }
        Err(err) => warn!("Failed to serialize OCPP CallError: {err}"),
    }
}

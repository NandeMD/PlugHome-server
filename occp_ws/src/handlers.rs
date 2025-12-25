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

use crate::types::*;

pub async fn handle_socket(mut socket: WebSocket, addr: SocketAddr) {
    info!(addr = %addr, "New WebSocket connection: {addr}");

    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            AxumWSMessage::Text(text) => {
                let message = text.clone();
                info!("\nINCOMING CALL\nFROM CHARGER\n\tMessage: {message}\n\tAddr: {addr}\n");
                handle_ocpp_messages(text, &mut socket).await;
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
        }
    }
}

async fn handle_ocpp_messages(message: String, socket: &mut WebSocket) {
    match serde_json::from_str(&message) {
        Ok(ocpp_message) => match ocpp_message {
            OcppMessageType::Call(message_type_id, message_id, action, payload) => {
                let action = match OcppActionEnum::from_str(&action) {
                    Ok(action) => {
                        debug!("Parsed OCPP call action: {action:?}");
                        action
                    }
                    Err(err) => {
                        error!("Failed to parse OCPP Call Action: {err:?}");
                        handle_ocpp_call_error(
                            4,
                            message_id,
                            "NotSupported".to_string(),
                            format!("Unknown action: {action}"),
                            json!({ "action": action, "reason": err.to_string() }),
                            socket,
                        )
                        .await;
                        return;
                    }
                };
                handle_ocpp_call(message_type_id, message_id, action, payload, socket).await;
            }
            OcppMessageType::CallResult(message_type_id, message_id, payload) => {
                handle_ocpp_call_result(message_type_id, message_id, payload, socket).await;
            }
            OcppMessageType::CallError(
                message_type_id,
                message_id,
                error_code,
                error_description,
                error_details,
            ) => {
                handle_ocpp_call_error(
                    message_type_id,
                    message_id,
                    error_code,
                    error_description,
                    error_details,
                    socket,
                )
                .await;
            }
        },
        Err(err) => warn!("Failed to parse OCPP message: {err:?}"),
    }
}

async fn handle_ocpp_call(
    _: OcppMessageTypeId,
    message_id: OcppMessageId,
    action: OcppActionEnum,
    payload: serde_json::Value,
    socket: &mut WebSocket,
) {
    let payload = match serde_json::from_value::<OcppPayload>(payload) {
        Ok(ocpp_payload) => ocpp_payload,
        Err(err) => {
            error!("Failed to parse OCPP Payload: {err:?}");
            return;
        }
    };

    use OcppActionEnum::*;

    match action {
        Authorize => {
            if let OcppPayload::Authorize(AuthorizeKind::Request(authorize)) = payload {
                info!("CALL REQUEST:\n{authorize:#?}");
                let response = OcppCallResult {
                    message_type_id: 3,
                    message_id,
                    payload: OcppPayload::Authorize(AuthorizeKind::Response(AuthorizeResponse {
                        id_tag_info: rust_ocpp::v1_6::types::IdTagInfo {
                            status: rust_ocpp::v1_6::types::AuthorizationStatus::Accepted,
                            expiry_date: None,
                            parent_id_tag: None,
                        },
                    })),
                };
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
        }
        BootNotification => {
            if let OcppPayload::BootNotification(BootNotificationKind::Request(boot_notification)) =
                payload
            {
                if boot_notification.charge_point_serial_number == Some("NKYK430037668".to_string())
                {
                    info!("CALL REQUEST:\n{boot_notification:#?}");
                    let response = OcppCallResult {
                        message_type_id: 3,
                        message_id,
                        payload: OcppPayload::BootNotification(BootNotificationKind::Response(
                            BootNotificationResponse {
                                status: rust_ocpp::v1_6::types::RegistrationStatus::Accepted,
                                current_time: Utc::now(),
                                interval: 300,
                            },
                        )),
                    };
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
                } else {
                    error!(
                        "Invalid Charger Serial Number. BootNotification: \
                             {boot_notification:?}"
                    );
                }
            } else {
                error!("Invalid OCPP BootNotification payload");
            }
        }
        ChangeAvailability => {}
        ChangeConfiguration => {}
        ClearCache => {}
        DataTransfer => {
            if let OcppPayload::DataTransfer(DataTransferKind::Request(data_transfer)) = payload {
                info!("CALL REQUEST:\n{data_transfer:#?}");
                let response = OcppCallResult {
                    message_type_id: 3,
                    message_id,
                    payload: OcppPayload::DataTransfer(DataTransferKind::Response(
                        DataTransferResponse {
                            status: rust_ocpp::v1_6::types::DataTransferStatus::Accepted,
                            data: Some("Data Transfer Accepted".to_string()),
                        },
                    )),
                };
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
        }
        GetConfiguration => {}
        Heartbeat => {
            if let OcppPayload::Heartbeat(HeartbeatKind::Request(heartbeat)) = payload {
                info!("CALL REQUEST:\n{heartbeat:#?}");
                let response = OcppCallResult {
                    message_type_id: 3,
                    message_id,
                    payload: OcppPayload::Heartbeat(HeartbeatKind::Response(HeartbeatResponse {
                        current_time: Utc::now(),
                    })),
                };
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
            }
        }
        MeterValues => {}
        RemoteStartTransaction => {}
        RemoteStopTransaction => {}
        Reset => {}
        StatusNotification => {
            if let OcppPayload::StatusNotification(StatusNotificationKind::Request(
                status_notification,
            )) = payload
            {
                info!("CALL REQUEST:\n{status_notification:#?}");
            }
        }
        StartTransaction => {}
        StopTransaction => {
            if let OcppPayload::StopTransaction(StopTransactionKind::Request(stop_transaction)) =
                payload
            {
                info!("CALL REQUEST:\n{stop_transaction:#?}");
                let response = OcppCallResult {
                    message_type_id: 3,
                    message_id,
                    payload: OcppPayload::StopTransaction(StopTransactionKind::Response(
                        StopTransactionResponse {
                            id_tag_info: Some(rust_ocpp::v1_6::types::IdTagInfo {
                                status: rust_ocpp::v1_6::types::AuthorizationStatus::Accepted,
                                expiry_date: None,
                                parent_id_tag: None,
                            }),
                        },
                    )),
                };
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
        }
        UnlockConnector => {}
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
    let ocpp_call_error = OcppCallError {
        message_type_id,
        message_id,
        error_code,
        error_description,
        error_details,
    };
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

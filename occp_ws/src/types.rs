use rust_ocpp::v1_6::messages::{
    authorize::{AuthorizeRequest, AuthorizeResponse},
    boot_notification::{BootNotificationRequest, BootNotificationResponse},
    change_availability::{ChangeAvailabilityRequest, ChangeAvailabilityResponse},
    change_configuration::{ChangeConfigurationRequest, ChangeConfigurationResponse},
    clear_cache::{ClearCacheRequest, ClearCacheResponse},
    data_transfer::{DataTransferRequest, DataTransferResponse},
    get_configuration::{GetConfigurationRequest, GetConfigurationResponse},
    heart_beat::{HeartbeatRequest, HeartbeatResponse},
    meter_values::{MeterValuesRequest, MeterValuesResponse},
    remote_start_transaction::{RemoteStartTransactionRequest, RemoteStartTransactionResponse},
    remote_stop_transaction::{RemoteStopTransactionRequest, RemoteStopTransactionResponse},
    reset::{ResetRequest, ResetResponse},
    start_transaction::{StartTransactionRequest, StartTransactionResponse},
    status_notification::{StatusNotificationRequest, StatusNotificationResponse},
    stop_transaction::{StopTransactionRequest, StopTransactionResponse},
    unlock_connector::{UnlockConnectorRequest, UnlockConnectorResponse},
};
use strum_macros::{Display, EnumString};

pub type OcppMessageTypeId = usize;
pub type OcppMessageId = String;
pub type OcppErrorCode = String;
pub type OcppErrorDescription = String;
pub type OcppErrorDetails = serde_json::Value;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, EnumString)]
#[serde(untagged)]
pub enum OcppActionEnum {
    // OCPP 1.6 JSON
    // Core
    Authorize,
    BootNotification,
    ChangeAvailability,
    ChangeConfiguration,
    DataTransfer,
    ClearCache,
    GetConfiguration,
    Heartbeat,
    MeterValues,
    RemoteStartTransaction,
    RemoteStopTransaction,
    Reset,
    StatusNotification,
    StartTransaction,
    StopTransaction,
    UnlockConnector,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum AuthorizeKind {
    Request(AuthorizeRequest),
    Response(AuthorizeResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum BootNotificationKind {
    Request(BootNotificationRequest),
    Response(BootNotificationResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum ChangeAvailabilityKind {
    Request(ChangeAvailabilityRequest),
    Response(ChangeAvailabilityResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum ChangeConfigurationKind {
    Request(ChangeConfigurationRequest),
    Response(ChangeConfigurationResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum ClearCacheKind {
    Request(ClearCacheRequest),
    Response(ClearCacheResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum DataTransferKind {
    Request(DataTransferRequest),
    Response(DataTransferResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum GetConfigurationKind {
    Request(GetConfigurationRequest),
    Response(GetConfigurationResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum HeartbeatKind {
    Request(HeartbeatRequest),
    Response(HeartbeatResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum MeterValuesKind {
    Request(MeterValuesRequest),
    Response(MeterValuesResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum RemoteStartTransactionKind {
    Request(RemoteStartTransactionRequest),
    Response(RemoteStartTransactionResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum RemoteStopTransactionKind {
    Request(RemoteStopTransactionRequest),
    Response(RemoteStopTransactionResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum ResetKind {
    Request(ResetRequest),
    Response(ResetResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum StartTransactionKind {
    Request(StartTransactionRequest),
    Response(StartTransactionResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum StopTransactionKind {
    Request(StopTransactionRequest),
    Response(StopTransactionResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum StatusNotificationKind {
    Request(StatusNotificationRequest),
    Response(StatusNotificationResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum UnlockConnectorKind {
    Request(UnlockConnectorRequest),
    Response(UnlockConnectorResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum OcppPayload {
    // OCPP 1.6 JSON
    // Core
    Authorize(AuthorizeKind),                           // Charger -> Server
    BootNotification(BootNotificationKind),             // Charger -> Server
    ChangeAvailability(ChangeAvailabilityKind),         // Server -> Charger
    ChangeConfiguration(ChangeConfigurationKind),       // Server -> Charger
    ClearCache(ClearCacheKind),                         // Server -> Charger
    DataTransfer(DataTransferKind),                     // Both directions
    GetConfiguration(GetConfigurationKind),             // Server -> Charger
    Heartbeat(HeartbeatKind),                           // Charger -> Server
    MeterValues(MeterValuesKind),                       // Charger -> Server
    RemoteStartTransaction(RemoteStartTransactionKind), // Server -> Charger
    RemoteStopTransaction(RemoteStopTransactionKind),   // Server -> Charger
    Reset(ResetKind),                                   // Server -> Charger
    StartTransaction(StartTransactionKind),             // Charger -> Server
    StatusNotification(StatusNotificationKind),         // Charger -> Server
    StopTransaction(StopTransactionKind),               // Charger -> Server
    UnlockConnector(UnlockConnectorKind),               // Server -> Charger
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
/// Call: [<MessageTypeId>, "<MessageId>", "<Action>", {<Payload>}]
pub struct OcppCall {
    pub message_type_id: OcppMessageTypeId,
    pub message_id: OcppMessageId,
    pub action: OcppActionEnum,
    pub payload: OcppPayload,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
/// CallResult: [<MessageTypeId>, "<MessageId>", {<Payload>}]
pub struct OcppCallResult {
    pub message_type_id: OcppMessageTypeId,
    pub message_id: OcppMessageId,
    pub payload: OcppPayload,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
/// CallError: [<MessageTypeId>, "<MessageId>", "<errorCode>", "<errorDescription>",
/// {<errorDetails>}]
pub struct OcppCallError {
    pub message_type_id: OcppMessageTypeId,
    pub message_id: OcppMessageId,
    pub error_code: OcppErrorCode,
    pub error_description: OcppErrorDescription,
    pub error_details: OcppErrorDetails,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum OcppMessageType {
    /// OCPP Call
    Call(usize, String, String, serde_json::Value),
    /// OCPP Result
    CallResult(usize, String, serde_json::Value),
    /// OCPP Error
    CallError(usize, String, String, String, serde_json::Value),
}

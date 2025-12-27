use rust_ocpp::v1_6::messages::{
    authorize::{AuthorizeRequest, AuthorizeResponse},
    boot_notification::{BootNotificationRequest, BootNotificationResponse},
    cancel_reservation::{CancelReservationRequest, CancelReservationResponse},
    change_availability::{ChangeAvailabilityRequest, ChangeAvailabilityResponse},
    change_configuration::{ChangeConfigurationRequest, ChangeConfigurationResponse},
    clear_cache::{ClearCacheRequest, ClearCacheResponse},
    clear_charging_profile::{ClearChargingProfileRequest, ClearChargingProfileResponse},
    data_transfer::{DataTransferRequest, DataTransferResponse},
    diagnostics_status_notification::{
        DiagnosticsStatusNotificationRequest,
        DiagnosticsStatusNotificationResponse,
    },
    firmware_status_notification::{
        FirmwareStatusNotificationRequest,
        FirmwareStatusNotificationResponse,
    },
    get_composite_schedule::{GetCompositeScheduleRequest, GetCompositeScheduleResponse},
    get_configuration::{GetConfigurationRequest, GetConfigurationResponse},
    get_diagnostics::{GetDiagnosticsRequest, GetDiagnosticsResponse},
    get_local_list_version::{GetLocalListVersionRequest, GetLocalListVersionResponse},
    heart_beat::{HeartbeatRequest, HeartbeatResponse},
    meter_values::{MeterValuesRequest, MeterValuesResponse},
    remote_start_transaction::{RemoteStartTransactionRequest, RemoteStartTransactionResponse},
    remote_stop_transaction::{RemoteStopTransactionRequest, RemoteStopTransactionResponse},
    reserve_now::{ReserveNowRequest, ReserveNowResponse},
    reset::{ResetRequest, ResetResponse},
    send_local_list::{SendLocalListRequest, SendLocalListResponse},
    set_charging_profile::{SetChargingProfileRequest, SetChargingProfileResponse},
    start_transaction::{StartTransactionRequest, StartTransactionResponse},
    status_notification::{StatusNotificationRequest, StatusNotificationResponse},
    stop_transaction::{StopTransactionRequest, StopTransactionResponse},
    trigger_message::{TriggerMessageRequest, TriggerMessageResponse},
    unlock_connector::{UnlockConnectorRequest, UnlockConnectorResponse},
    update_firmware::{UpdateFirmwareRequest, UpdateFirmwareResponse},
};
use strum_macros::{Display, EnumString};

pub type OcppMessageTypeId = usize;
pub type OcppMessageId = String;
pub type OcppErrorCode = String;
pub type OcppErrorDescription = String;
pub type OcppErrorDetails = serde_json::Value;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, EnumString, Display)]
#[serde(rename_all = "PascalCase")]
pub enum OcppActionEnum {
    // OCPP 1.6 JSON
    // Core and supported profiles
    Authorize,
    BootNotification,
    CancelReservation,
    ChangeAvailability,
    ChangeConfiguration,
    ClearCache,
    ClearChargingProfile,
    DataTransfer,
    DiagnosticsStatusNotification,
    FirmwareStatusNotification,
    GetCompositeSchedule,
    GetConfiguration,
    GetDiagnostics,
    GetLocalListVersion,
    Heartbeat,
    MeterValues,
    RemoteStartTransaction,
    RemoteStopTransaction,
    ReserveNow,
    Reset,
    SendLocalList,
    SetChargingProfile,
    StartTransaction,
    StatusNotification,
    StopTransaction,
    TriggerMessage,
    UnlockConnector,
    UpdateFirmware,
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
pub enum CancelReservationKind {
    Request(CancelReservationRequest),
    Response(CancelReservationResponse),
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
pub enum ClearChargingProfileKind {
    Request(ClearChargingProfileRequest),
    Response(ClearChargingProfileResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum DataTransferKind {
    Request(DataTransferRequest),
    Response(DataTransferResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum DiagnosticsStatusNotificationKind {
    Request(DiagnosticsStatusNotificationRequest),
    Response(DiagnosticsStatusNotificationResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum FirmwareStatusNotificationKind {
    Request(FirmwareStatusNotificationRequest),
    Response(FirmwareStatusNotificationResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum GetCompositeScheduleKind {
    Request(GetCompositeScheduleRequest),
    Response(GetCompositeScheduleResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum GetConfigurationKind {
    Request(GetConfigurationRequest),
    Response(GetConfigurationResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum GetDiagnosticsKind {
    Request(GetDiagnosticsRequest),
    Response(GetDiagnosticsResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum GetLocalListVersionKind {
    Request(GetLocalListVersionRequest),
    Response(GetLocalListVersionResponse),
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
pub enum ReserveNowKind {
    Request(ReserveNowRequest),
    Response(ReserveNowResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum ResetKind {
    Request(ResetRequest),
    Response(ResetResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum SendLocalListKind {
    Request(SendLocalListRequest),
    Response(SendLocalListResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum SetChargingProfileKind {
    Request(SetChargingProfileRequest),
    Response(SetChargingProfileResponse),
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
pub enum TriggerMessageKind {
    Request(TriggerMessageRequest),
    Response(TriggerMessageResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum UnlockConnectorKind {
    Request(UnlockConnectorRequest),
    Response(UnlockConnectorResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(untagged)]
pub enum UpdateFirmwareKind {
    Request(UpdateFirmwareRequest),
    Response(UpdateFirmwareResponse),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum OcppPayload {
    // OCPP 1.6 JSON
    // Core
    Authorize(AuthorizeKind),                                   // Charger -> Server
    BootNotification(BootNotificationKind),                     // Charger -> Server
    CancelReservation(CancelReservationKind),                   // Server -> Charger
    ChangeAvailability(ChangeAvailabilityKind),                 // Server -> Charger
    ChangeConfiguration(ChangeConfigurationKind),               // Server -> Charger
    ClearCache(ClearCacheKind),                                 // Server -> Charger
    ClearChargingProfile(ClearChargingProfileKind),             // Server -> Charger
    DataTransfer(DataTransferKind),                             // Both directions
    DiagnosticsStatusNotification(DiagnosticsStatusNotificationKind), // Charger -> Server
    FirmwareStatusNotification(FirmwareStatusNotificationKind),       // Charger -> Server
    GetCompositeSchedule(GetCompositeScheduleKind),             // Server -> Charger
    GetConfiguration(GetConfigurationKind),                     // Server -> Charger
    GetDiagnostics(GetDiagnosticsKind),                         // Server -> Charger
    GetLocalListVersion(GetLocalListVersionKind),               // Server -> Charger
    Heartbeat(HeartbeatKind),                                   // Charger -> Server
    MeterValues(MeterValuesKind),                               // Charger -> Server
    RemoteStartTransaction(RemoteStartTransactionKind),         // Server -> Charger
    RemoteStopTransaction(RemoteStopTransactionKind),           // Server -> Charger
    ReserveNow(ReserveNowKind),                                 // Server -> Charger
    Reset(ResetKind),                                           // Server -> Charger
    SendLocalList(SendLocalListKind),                           // Server -> Charger
    SetChargingProfile(SetChargingProfileKind),                 // Server -> Charger
    StartTransaction(StartTransactionKind),                     // Charger -> Server
    StatusNotification(StatusNotificationKind),                 // Charger -> Server
    StopTransaction(StopTransactionKind),                       // Charger -> Server
    TriggerMessage(TriggerMessageKind),                         // Server -> Charger
    UnlockConnector(UnlockConnectorKind),                       // Server -> Charger
    UpdateFirmware(UpdateFirmwareKind),                         // Server -> Charger
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
/// Call: [<MessageTypeId>, "<MessageId>", "<Action>", {<Payload>}]
pub struct OcppCall(
    pub OcppMessageTypeId,
    pub OcppMessageId,
    pub OcppActionEnum,
    pub OcppPayload,
);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
/// CallResult: [<MessageTypeId>, "<MessageId>", {<Payload>}]
pub struct OcppCallResult(pub OcppMessageTypeId, pub OcppMessageId, pub OcppPayload);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
/// CallError: [<MessageTypeId>, "<MessageId>", "<errorCode>", "<errorDescription>",
/// {<errorDetails>}]
pub struct OcppCallError(
    pub OcppMessageTypeId,
    pub OcppMessageId,
    pub OcppErrorCode,
    pub OcppErrorDescription,
    pub OcppErrorDetails,
);

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

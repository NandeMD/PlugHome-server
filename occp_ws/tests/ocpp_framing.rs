use chrono::{TimeZone, Utc};
use occp_ws::types::*;
use rust_ocpp::v1_6::messages::boot_notification::{BootNotificationRequest, BootNotificationResponse};
use rust_ocpp::v1_6::types::RegistrationStatus;
use serde_json::json;

#[test]
fn serializes_call_as_ocpp_array() {
    let payload = OcppPayload::BootNotification(BootNotificationKind::Request(
        BootNotificationRequest {
            charge_point_model: "ModelX".to_string(),
            charge_point_vendor: "AcmeCorp".to_string(),
            ..Default::default()
        },
    ));

    let call = OcppCall(2, "123".to_string(), OcppActionEnum::BootNotification, payload);

    let framed = serde_json::to_value(&call).expect("serialize call");

    assert_eq!(
        framed,
        json!([2, "123", "BootNotification", {"chargePointModel": "ModelX", "chargePointVendor": "AcmeCorp"}])
    );
}

#[test]
fn deserializes_call_from_ocpp_array() {
    let raw = r#"[2, "123", "BootNotification", {"chargePointModel": "ModelX", "chargePointVendor": "AcmeCorp"}]"#;

    let parsed: OcppMessageType = serde_json::from_str(raw).expect("deserialize call frame");

    match parsed {
        OcppMessageType::Call(mt, id, action, payload) => {
            assert_eq!(mt, 2);
            assert_eq!(id, "123");
            assert_eq!(action, "BootNotification");

            let payload: OcppPayload = serde_json::from_value(payload).expect("parse payload");
            match payload {
                OcppPayload::BootNotification(BootNotificationKind::Request(body)) => {
                    assert_eq!(body.charge_point_model, "ModelX");
                    assert_eq!(body.charge_point_vendor, "AcmeCorp");
                }
                other => panic!("unexpected payload: {other:?}"),
            }
        }
        other => panic!("unexpected message type: {other:?}"),
    }
}

#[test]
fn serializes_call_result_as_ocpp_array() {
    let ts = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

    let payload = OcppPayload::BootNotification(BootNotificationKind::Response(
        BootNotificationResponse {
            status: RegistrationStatus::Accepted,
            current_time: ts,
            interval: 300,
        },
    ));

    let call_result = OcppCallResult(3, "abc".to_string(), payload);

    let framed = serde_json::to_value(&call_result).expect("serialize call result");

    assert_eq!(
        framed,
        json!([3, "abc", {"status": "Accepted", "currentTime": "2024-01-01T00:00:00Z", "interval": 300}])
    );
}

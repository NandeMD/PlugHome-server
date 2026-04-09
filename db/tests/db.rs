use common::ServerConfig;
use db::{BootNotification, Db, Station, StationConnectionState, boot_notification, station};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::time::Duration;
use tokio::time::sleep;

fn test_config() -> ServerConfig {
    ServerConfig {
        addr: "127.0.0.1".to_owned(),
        port: "0".to_owned(),
        rust_log: "info".to_owned(),
        allowed_serials: Vec::new(),
        db_url: "sqlite::memory:".to_owned(),
        station_timeout: 30,
    }
}

async fn setup_test_db() -> Db {
    Db::try_new(&test_config()).await.unwrap()
}

#[tokio::test]
async fn db_connects_and_migrates() {
    let db = setup_test_db().await;

    let stations = Station::find().all(&db.conn).await.unwrap();
    let boot_notifications = BootNotification::find().all(&db.conn).await.unwrap();

    assert!(stations.is_empty());
    assert!(boot_notifications.is_empty());
}

#[tokio::test]
async fn record_boot_notification_creates_station_and_notification() {
    let db = setup_test_db().await;

    db.record_boot_notification("station-123").await.unwrap();

    let station = Station::find()
        .filter(station::Column::StationId.eq("station-123"))
        .one(&db.conn)
        .await
        .unwrap()
        .unwrap();

    let boot_notifications = BootNotification::find()
        .filter(boot_notification::Column::StationId.eq(station.id))
        .all(&db.conn)
        .await
        .unwrap();

    assert_eq!(station.connection_state, StationConnectionState::Online);
    assert_eq!(boot_notifications.len(), 1);
    assert_eq!(boot_notifications[0].station_id, station.id);
    assert_eq!(boot_notifications[0].received_at, station.last_seen);
}

#[tokio::test]
async fn record_boot_notification_reuses_station_and_adds_notification() {
    let db = setup_test_db().await;

    db.record_boot_notification("station-123").await.unwrap();

    let first_station = Station::find()
        .filter(station::Column::StationId.eq("station-123"))
        .one(&db.conn)
        .await
        .unwrap()
        .unwrap();
    let first_last_seen = first_station.last_seen;

    sleep(Duration::from_millis(5)).await;

    db.record_boot_notification("station-123").await.unwrap();

    let stations = Station::find().all(&db.conn).await.unwrap();
    let updated_station = Station::find()
        .filter(station::Column::StationId.eq("station-123"))
        .one(&db.conn)
        .await
        .unwrap()
        .unwrap();
    let boot_notifications = BootNotification::find()
        .filter(boot_notification::Column::StationId.eq(updated_station.id))
        .all(&db.conn)
        .await
        .unwrap();

    assert_eq!(stations.len(), 1);
    assert_eq!(
        updated_station.connection_state,
        StationConnectionState::Online
    );
    assert!(updated_station.last_seen > first_last_seen);
    assert_eq!(boot_notifications.len(), 2);
}

#[tokio::test]
async fn mark_station_offline_updates_existing_station() {
    let db = setup_test_db().await;

    db.record_boot_notification("station-123").await.unwrap();
    db.mark_station_offline("station-123").await.unwrap();

    let station = Station::find()
        .filter(station::Column::StationId.eq("station-123"))
        .one(&db.conn)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(station.connection_state, StationConnectionState::Offline);
}

#[tokio::test]
async fn mark_station_online_if_registered_updates_existing_station() {
    let db = setup_test_db().await;

    db.record_boot_notification("station-123").await.unwrap();
    db.mark_station_offline("station-123").await.unwrap();
    db.mark_station_online_if_registered("station-123")
        .await
        .unwrap();

    let station = Station::find()
        .filter(station::Column::StationId.eq("station-123"))
        .one(&db.conn)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(station.connection_state, StationConnectionState::Online);
}

#[tokio::test]
async fn mark_station_online_if_registered_does_not_create_station() {
    let db = setup_test_db().await;

    db.mark_station_online_if_registered("missing-station")
        .await
        .unwrap();

    let station = Station::find()
        .filter(station::Column::StationId.eq("missing-station"))
        .one(&db.conn)
        .await
        .unwrap();

    assert!(station.is_none());
}

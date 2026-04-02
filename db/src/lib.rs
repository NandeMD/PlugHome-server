pub mod entities;
pub mod migration;

use chrono::Utc;
use common::ServerConfig;
pub use entities::boot_notification::{
    self, Entity as BootNotification, Model as BootNotificationModel,
};
pub use entities::station::{
    self, Entity as Station, Model as StationModel, StationConnectionState,
};
use migration::run_migrations;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    Set, TransactionTrait,
};

pub struct Db {
    pub conn: DatabaseConnection,
}

impl Db {
    pub async fn try_new(config: &ServerConfig) -> Result<Self, DbErr> {
        let conn = Database::connect(&config.db_url).await?;

        // Migrate
        run_migrations(&conn).await?;

        Ok(Db { conn })
    }

    pub async fn record_boot_notification(&self, station_id: &str) -> Result<(), DbErr> {
        let txn = self.conn.begin().await?;
        let now = Utc::now();

        let station = match Station::find()
            .filter(station::Column::StationId.eq(station_id))
            .one(&txn)
            .await?
        {
            Some(station) => {
                let mut active_station: station::ActiveModel = station.into();
                active_station.connection_state = Set(StationConnectionState::Online);
                active_station.last_seen = Set(now.clone());
                active_station.update(&txn).await?
            }
            None => {
                station::ActiveModel {
                    station_id: Set(station_id.to_owned()),
                    connection_state: Set(StationConnectionState::Online),
                    last_seen: Set(now.clone()),
                    ..Default::default()
                }
                .insert(&txn)
                .await?
            }
        };

        boot_notification::ActiveModel {
            station_id: Set(station.id),
            received_at: Set(now),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        txn.commit().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::EntityTrait;
    use std::time::Duration;
    use tokio::time::sleep;

    async fn setup_test_db() -> Db {
        let conn = Database::connect("sqlite::memory:").await.unwrap();
        run_migrations(&conn).await.unwrap();
        Db { conn }
    }

    #[tokio::test]
    async fn test_db_connects_and_migrates() {
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
}

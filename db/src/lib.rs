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

    pub async fn mark_station_offline(&self, station_id: &str) -> Result<(), DbErr> {
        if let Some(station) = Station::find()
            .filter(station::Column::StationId.eq(station_id))
            .one(&self.conn)
            .await?
        {
            let mut active_station: station::ActiveModel = station.into();
            active_station.connection_state = Set(StationConnectionState::Offline);
            active_station.update(&self.conn).await?;
        }

        Ok(())
    }

    pub async fn mark_station_online_if_registered(&self, station_id: &str) -> Result<(), DbErr> {
        if let Some(station) = Station::find()
            .filter(station::Column::StationId.eq(station_id))
            .one(&self.conn)
            .await?
        {
            let mut active_station: station::ActiveModel = station.into();
            active_station.connection_state = Set(StationConnectionState::Online);
            active_station.update(&self.conn).await?;
        }

        Ok(())
    }

    // Updates `last_seen` field. Sorry for the bad humor.
    pub async fn still_not_dead(&self, station_id: &str) -> Result<(), DbErr> {
        if let Some(station) = Station::find()
            .filter(station::Column::StationId.eq(station_id))
            .one(&self.conn)
            .await?
        {
            let mut active_station: station::ActiveModel = station.into();
            active_station.last_seen = Set(Utc::now());
            active_station.update(&self.conn).await?;
        }

        Ok(())
    }
}

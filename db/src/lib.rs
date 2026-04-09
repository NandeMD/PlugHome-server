pub mod entities;
pub mod migration;

use chrono::{Duration, Utc};
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
    station_timeout: u32,

    pub conn: DatabaseConnection,
}

impl Db {
    pub async fn try_new(config: &ServerConfig) -> Result<Self, DbErr> {
        let conn = Database::connect(&config.db_url).await?;

        // Migrate
        run_migrations(&conn).await?;

        Ok(Db {
            conn,
            station_timeout: config.station_timeout,
        })
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
                active_station.last_seen = Set(now);
                active_station.update(&txn).await?
            }
            None => {
                station::ActiveModel {
                    station_id: Set(station_id.to_owned()),
                    connection_state: Set(StationConnectionState::Online),
                    last_seen: Set(now),
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

    pub async fn mark_station_dead(&self, station_id: &str) -> Result<(), DbErr> {
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

    // On cold starts, the connection status field has no meaning
    // So we need to think all stations as offline
    // And this helper is for this
    pub async fn kill_all(&self) -> Result<(), DbErr> {
        for st in Station::find()
            .filter(station::Column::ConnectionState.eq(StationConnectionState::Online))
            .all(&self.conn)
            .await?
        {
            let mut active_station: station::ActiveModel = st.into();
            active_station.connection_state = Set(StationConnectionState::Offline);
            active_station.update(&self.conn).await?;
        }

        Ok(())
    }

    /// Returns sweeped station ids if Ok
    pub async fn sweep(&self) -> Result<Vec<String>, DbErr> {
        let max_retention = self.station_timeout * 2 + 30;
        let cutoff = Utc::now() - Duration::seconds(max_retention as i64);
        let mut sweeped_station_ids: Vec<String> = Vec::new();

        for st in Station::find()
            .filter(station::Column::LastSeen.lte(cutoff))
            .all(&self.conn)
            .await?
        {
            let station_id = st.station_id.clone();

            let mut active_station: station::ActiveModel = st.into();
            active_station.connection_state = Set(StationConnectionState::Offline);
            active_station.update(&self.conn).await?;

            sweeped_station_ids.push(station_id);
        }

        Ok(sweeped_station_ids)
    }
}

pub mod entities;
pub mod migration;

use common::ServerConfig;
pub use entities::boot_notification::{
    self, Entity as BootNotification, Model as BootNotificationModel,
};
pub use entities::station::{
    self, Entity as Station, Model as StationModel, StationConnectionState,
};
use migration::run_migrations;
use sea_orm::{Database, DatabaseConnection, DbErr};

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_db_connects_and_migrates() {
        let _ = common::load_env();
        let conf = common::ServerConfig::from_env().unwrap();
        let _db = Db::try_new(&conf).await.unwrap();
    }
}

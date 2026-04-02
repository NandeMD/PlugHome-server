use sea_orm::{Database, DatabaseConnection, DbErr};

static DB_URL: &str = "sqlite::memory:";

pub struct Db {
    pub conn: DatabaseConnection,
}

impl Db {
    async fn connect(&mut self) -> Result<(), DbErr> {
        self.conn = Database::connect(DB_URL).await?;
        Ok(())
    }
}

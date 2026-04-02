use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BootNotifications::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BootNotifications::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(BootNotifications::StationId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BootNotifications::ReceivedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-boot_notifications-station_id")
                            .from(BootNotifications::Table, BootNotifications::StationId)
                            .to(Stations::Table, Stations::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-boot_notifications-station_id")
                    .table(BootNotifications::Table)
                    .col(BootNotifications::StationId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BootNotifications::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum BootNotifications {
    Table,
    Id,
    StationId,
    ReceivedAt,
}

#[derive(DeriveIden)]
enum Stations {
    Table,
    Id,
}

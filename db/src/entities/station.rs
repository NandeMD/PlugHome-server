use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "stations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub station_id: String,
    pub connection_state: StationConnectionState,
    pub last_seen: DateTimeUtc,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(Some(8))")]
pub enum StationConnectionState {
    #[sea_orm(string_value = "online")]
    Online,
    #[sea_orm(string_value = "offline")]
    Offline,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::boot_notification::Entity")]
    BootNotification,
}

impl Related<super::boot_notification::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BootNotification.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

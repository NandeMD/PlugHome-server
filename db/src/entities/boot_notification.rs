use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "boot_notifications")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub station_id: i32,
    pub received_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::station::Entity",
        from = "Column::StationId",
        to = "super::station::Column::Id",
        on_delete = "Cascade"
    )]
    Station,
}

impl Related<super::station::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Station.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

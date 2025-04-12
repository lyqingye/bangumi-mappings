use crate::models::enums::{Platform, ReviewStatus};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "mappings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub anilist_id: i32,
    pub platform: Platform,
    pub platform_id: Option<String>,
    pub review_status: ReviewStatus,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub score: u8,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::anime::Entity",
        from = "Column::AnilistId",
        to = "super::anime::Column::AnilistId",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Anime,
}

impl Related<super::anime::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Anime.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

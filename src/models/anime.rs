use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::enums::MediaType;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "animes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub anilist_id: i32,
    pub media_type: MediaType,
    pub titles: String,
    pub year: i32,
    pub season: Option<String>,
    pub start_date: Option<String>,
    pub episode_count: Option<i32>,
    pub season_number: Option<i32>,
    pub episode_number: Option<i32>,
    pub absolute_episode_number: Option<i32>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::mappings::Entity")]
    AnimeMapping,
}

impl Related<super::mappings::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AnimeMapping.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

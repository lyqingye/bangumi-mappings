use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "mappings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub anilist_id: i32,
    pub air_year: Option<i32>,
    pub air_month: Option<i32>,
    pub air_day: Option<i32>,
    pub bgm_id: Option<i32>,
    pub tmdb_id: Option<i32>,
    pub tmdb_season: Option<i32>,
    pub title: Option<String>,
    pub title_en: Option<String>,
    pub title_cn: Option<String>,
    pub title_romaji: Option<String>,
    pub alias: Option<String>,
    pub update_at: DateTime,
    pub bgm_tv_verify_status: VerificationStatus,
    pub tmdb_verify_status: VerificationStatus,
    pub match_count: u32,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "verification_status"
)]
pub enum VerificationStatus {
    #[sea_orm(string_value = "Accepted")]
    Accepted,
    #[sea_orm(string_value = "Rejected")]
    Rejected,
    #[sea_orm(string_value = "Dropped")]
    Dropped,
    #[sea_orm(string_value = "Ready")]
    Ready,
    #[sea_orm(string_value = "UnMatched")]
    UnMatched,
}

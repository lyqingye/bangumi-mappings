use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "review_status")]
pub enum ReviewStatus {
    #[sea_orm(string_value = "UnMatched")]
    UnMatched,
    #[sea_orm(string_value = "Ready")]
    Ready,
    #[sea_orm(string_value = "Accepted")]
    Accepted,
    #[sea_orm(string_value = "Rejected")]
    Rejected,
    #[sea_orm(string_value = "Dropped")]
    Dropped,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "platform")]
pub enum Platform {
    #[sea_orm(string_value = "BGM_TV")]
    BgmTv,
    #[sea_orm(string_value = "TMDB")]
    Tmdb,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "media_type")]
pub enum MediaType {
    #[sea_orm(string_value = "Movie")]
    Movie,
    #[sea_orm(string_value = "OVA")]
    OVA,
    #[sea_orm(string_value = "ONA")]
    ONA,
    #[sea_orm(string_value = "Special")]
    Special,
    #[sea_orm(string_value = "TV")]
    TV,
    #[sea_orm(string_value = "Unknown")]
    Unknown,
}

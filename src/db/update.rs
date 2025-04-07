use super::Db;
use super::mappings::VerificationStatus;
use crate::db::mappings::Column as MappingColumn;
use crate::db::mappings::Entity as Mappings;
use crate::db::mappings::Model as Mapping;
use anyhow::Result;
use chrono::Utc;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::sea_query::OnConflict;
use sea_orm::sea_query::SimpleExpr;
use sea_orm::{EntityTrait, IntoActiveModel};
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVerificationRequest {
    #[serde(rename = "anilistId")]
    pub anilist_id: i32,
    pub source: String,
    pub status: VerificationStatus,
}

impl Db {
    pub async fn import_anilist_animes(&self, mappings: Vec<Mapping>) -> Result<()> {
        Mappings::insert_many(mappings.into_iter().map(|m| m.into_active_model()))
            .on_conflict(
                OnConflict::column(MappingColumn::AnilistId)
                    .update_columns(vec![
                        MappingColumn::AirYear,
                        MappingColumn::AirMonth,
                        MappingColumn::AirDay,
                        MappingColumn::Title,
                        MappingColumn::TitleEn,
                        MappingColumn::TitleCn,
                        MappingColumn::TitleRomaji,
                        MappingColumn::Alias,
                        MappingColumn::UpdateAt,
                        MappingColumn::MatchCount,
                    ])
                    .to_owned(),
            )
            .exec(self.conn())
            .await?;
        Ok(())
    }

    pub async fn import_mappings(&self, mappings: Vec<Mapping>) -> Result<()> {
        Mappings::insert_many(mappings.into_iter().map(|m| m.into_active_model()))
            .on_conflict(
                OnConflict::column(MappingColumn::AnilistId)
                    .update_columns(vec![
                        MappingColumn::BgmId,
                        MappingColumn::TmdbId,
                        MappingColumn::TmdbSeason,
                        MappingColumn::BgmTvVerifyStatus,
                        MappingColumn::TmdbVerifyStatus,
                        MappingColumn::UpdateAt,
                    ])
                    .to_owned(),
            )
            .exec(self.conn())
            .await?;
        Ok(())
    }

    pub async fn update_verification_status(
        &self,
        request: UpdateVerificationRequest,
    ) -> Result<()> {
        match request.source.as_str() {
            "tmdb" => {
                self.update_tmdb_verify_status(request.anilist_id, request.status)
                    .await
            }
            "bgmtv" => {
                self.update_bgm_tv_verify_status(request.anilist_id, request.status)
                    .await
            }
            _ => Err(anyhow::anyhow!("Invalid source")),
        }
    }

    pub async fn update_bgm_tv_verify_status(
        &self,
        anilist_id: i32,
        status: VerificationStatus,
    ) -> Result<()> {
        Mappings::update_many()
            .col_expr(MappingColumn::BgmTvVerifyStatus, SimpleExpr::from(status))
            .col_expr(
                MappingColumn::UpdateAt,
                SimpleExpr::from(Utc::now().naive_utc()),
            )
            .filter(MappingColumn::AnilistId.eq(anilist_id))
            .exec(self.conn())
            .await?;
        Ok(())
    }

    pub async fn update_tmdb_verify_status(
        &self,
        anilist_id: i32,
        status: VerificationStatus,
    ) -> Result<()> {
        Mappings::update_many()
            .col_expr(MappingColumn::TmdbVerifyStatus, SimpleExpr::from(status))
            .col_expr(
                MappingColumn::UpdateAt,
                SimpleExpr::from(Utc::now().naive_utc()),
            )
            .filter(MappingColumn::AnilistId.eq(anilist_id))
            .exec(self.conn())
            .await?;
        Ok(())
    }
}

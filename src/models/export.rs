use super::db::DB;
use crate::models::anime::Column as AnimeColumn;
use crate::models::anime::Entity as AnimeEntity;
use crate::models::anime::Model as Anime;
use crate::models::mappings::Column as AnimeMappingColumn;
use crate::models::mappings::Entity as AnimeMappingEntity;
use crate::models::mappings::Model as AnimeMapping;
use anyhow::Result;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct ExportAnime {
    #[serde(flatten)]
    pub anime: Anime,
    pub mappings: Vec<AnimeMapping>,
}

impl DB {
    pub async fn export_animes(&self, year: i32) -> Result<Vec<ExportAnime>> {
        let animes = AnimeEntity::find()
            .filter(AnimeColumn::Year.eq(year))
            .all(self.conn())
            .await?;

        let anilist_ids: Vec<i32> = animes.iter().map(|a| a.anilist_id).collect();

        let mappings = AnimeMappingEntity::find()
            .filter(AnimeMappingColumn::AnilistId.is_in(anilist_ids))
            .all(self.conn())
            .await?;

        // 使用HashMap进行高效分组
        let mut mapping_groups: std::collections::HashMap<i32, Vec<AnimeMapping>> =
            std::collections::HashMap::new();

        for mapping in mappings {
            mapping_groups
                .entry(mapping.anilist_id)
                .or_insert_with(Vec::new)
                .push(mapping);
        }

        Ok(animes
            .into_iter()
            .map(|anime| {
                let anilist_id = anime.anilist_id;
                ExportAnime {
                    anime,
                    mappings: mapping_groups.get(&anilist_id).cloned().unwrap_or_default(),
                }
            })
            .collect())
    }

    pub async fn import_animes(&self, animes: Vec<ExportAnime>) -> Result<()> {
        if animes.is_empty() {
            return Ok(());
        }

        // 1. 提取所有anime记录和mapping记录
        let anime_models: Vec<Anime> = animes.iter().map(|item| item.anime.clone()).collect();
        let mut all_mappings: Vec<AnimeMapping> = Vec::new();
        for export_anime in &animes {
            all_mappings.extend(export_anime.mappings.clone());
        }

        // 2. 批量upsert anime记录
        use crate::models::anime::ActiveModel as AnimeActiveModel;
        use crate::models::mappings::ActiveModel as MappingActiveModel;
        use sea_orm::{IntoActiveModel, Set};

        let active_animes: Vec<AnimeActiveModel> = anime_models
            .into_iter()
            .map(|model| {
                let mut active_model = model.into_active_model();
                // 确保所有字段都被设置为Set值
                active_model.updated_at = Set(chrono::Utc::now());
                active_model
            })
            .collect();

        if !active_animes.is_empty() {
            AnimeEntity::insert_many(active_animes)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(AnimeColumn::AnilistId)
                        .update_columns([
                            AnimeColumn::MediaType,
                            AnimeColumn::Titles,
                            AnimeColumn::Year,
                            AnimeColumn::Season,
                            AnimeColumn::StartDate,
                            AnimeColumn::EpisodeCount,
                            AnimeColumn::SeasonNumber,
                            AnimeColumn::EpisodeNumber,
                            AnimeColumn::AbsoluteEpisodeNumber,
                            AnimeColumn::UpdatedAt,
                        ])
                        .to_owned(),
                )
                .exec(self.conn())
                .await?;
        }

        // 3. 批量upsert mapping记录
        if !all_mappings.is_empty() {
            let active_mappings: Vec<MappingActiveModel> = all_mappings
                .into_iter()
                .map(|model| {
                    let mut active_model = model.into_active_model();
                    active_model.updated_at = Set(chrono::Utc::now());
                    active_model
                })
                .collect();

            AnimeMappingEntity::insert_many(active_mappings)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::columns([
                        AnimeMappingColumn::AnilistId,
                        AnimeMappingColumn::Platform,
                    ])
                    .update_columns([
                        AnimeMappingColumn::PlatformId,
                        AnimeMappingColumn::ReviewStatus,
                        AnimeMappingColumn::Score,
                        AnimeMappingColumn::UpdatedAt,
                    ])
                    .to_owned(),
                )
                .exec(self.conn())
                .await?;
        }

        Ok(())
    }
}

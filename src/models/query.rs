use super::db::DB;
use super::enums::Platform;
use super::enums::ReviewStatus;
use crate::api::types::Pagination;
use crate::api::types::QueryAnimes;
use crate::models::anime::Column as AnimeColumn;
use crate::models::anime::Entity as AnimeEntity;
use crate::models::anime::Model as Anime;
use crate::models::mappings::Column as AnimeMappingColumn;
use crate::models::mappings::Entity as AnimeMappingEntity;
use crate::models::mappings::Model as AnimeMapping;
use anyhow::Result;
use chrono::Utc;
use sea_orm::ColumnTrait;
use sea_orm::JoinType;
use sea_orm::Order;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::QuerySelect;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, Set, TransactionTrait,
};
use std::collections::HashMap;

impl DB {
    pub async fn batch_add_animes(&self, animes: (Vec<Anime>, Vec<AnimeMapping>)) -> Result<()> {
        let now = Utc::now();
        let (anime_models, mapping_models) = animes;

        let txn = self.db.begin().await?;

        for anime in anime_models {
            let mut anime_active = anime.into_active_model();
            anime_active.created_at = Set(now);
            anime_active.updated_at = Set(now);

            anime_active.insert(&txn).await?;
        }

        for mapping in mapping_models {
            let mut mapping_active = mapping.into_active_model();
            mapping_active.created_at = Set(now);
            mapping_active.updated_at = Set(now);

            mapping_active.insert(&txn).await?;
        }

        txn.commit().await?;

        Ok(())
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.db
    }

    pub async fn get_anime(&self, anilist_id: i32) -> Result<(Option<Anime>, Vec<AnimeMapping>)> {
        let anime = AnimeEntity::find_by_id(anilist_id).one(self.conn()).await?;
        let mappings = AnimeMappingEntity::find_by_id(anilist_id)
            .all(self.conn())
            .await?;
        Ok((anime, mappings))
    }

    pub async fn review(
        &self,
        anilist_id: i32,
        platform: Platform,
        status: ReviewStatus,
    ) -> Result<()> {
        AnimeMappingEntity::update_many()
            .filter(AnimeMappingColumn::AnilistId.eq(anilist_id))
            .filter(AnimeMappingColumn::Platform.eq(platform))
            .col_expr(AnimeMappingColumn::ReviewStatus, status.into())
            .col_expr(AnimeMappingColumn::UpdatedAt, Utc::now().into())
            .exec(self.conn())
            .await?;
        Ok(())
    }

    pub async fn update_anime_mapping(
        &self,
        anilist_id: i32,
        platform: Platform,
        platform_id: String,
        score: u8,
    ) -> Result<()> {
        AnimeMappingEntity::update_many()
            .filter(AnimeMappingColumn::AnilistId.eq(anilist_id))
            .filter(AnimeMappingColumn::Platform.eq(platform))
            .col_expr(AnimeMappingColumn::ReviewStatus, ReviewStatus::Ready.into())
            .col_expr(AnimeMappingColumn::UpdatedAt, Utc::now().into())
            .col_expr(AnimeMappingColumn::PlatformId, platform_id.into())
            .col_expr(AnimeMappingColumn::Score, score.into())
            .exec(self.conn())
            .await?;
        Ok(())
    }

    pub async fn query_animes(
        &self,
        query: &QueryAnimes,
    ) -> Result<Pagination<(Anime, Vec<AnimeMapping>)>> {
        // 查询动漫ID，考虑所有筛选条件
        let mut anime_ids_query = AnimeMappingEntity::find()
            .select_only()
            .column(AnimeMappingColumn::AnilistId)
            .distinct();

        // 应用审核状态过滤条件
        if let Some(ref status) = query.status {
            anime_ids_query =
                anime_ids_query.filter(AnimeMappingColumn::ReviewStatus.eq(status.clone()));
        }

        // 通过JOIN关联动漫表
        anime_ids_query = anime_ids_query.join(
            JoinType::InnerJoin,
            AnimeMappingEntity::belongs_to(AnimeEntity)
                .from(AnimeMappingColumn::AnilistId)
                .to(AnimeColumn::AnilistId)
                .into(),
        );

        // 应用年份过滤条件
        if let Some(year) = query.year {
            anime_ids_query = anime_ids_query.filter(AnimeColumn::Year.eq(year));
        }

        // 获取符合条件的动漫IDs
        let anime_ids = anime_ids_query.into_tuple().all(self.conn()).await?;

        // 获取总记录数
        let total = anime_ids.len();

        // 应用分页
        let page = query.query.page;
        let page_size = query.query.page_size;
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(total);

        // 如果没有结果或超出范围，返回空数据
        if start >= total {
            return Ok(Pagination {
                page,
                page_size,
                total,
                data: vec![],
            });
        }

        // 获取当前页的动漫IDs
        let paged_anime_ids: Vec<i32> = anime_ids
            .iter()
            .skip(start)
            .take(end - start)
            .map(|(id,)| *id)
            .collect();

        // 构建条件查询当前页的动漫
        let mut anime_condition = sea_orm::Condition::any();
        for id in &paged_anime_ids {
            anime_condition = anime_condition.add(AnimeColumn::AnilistId.eq(*id));
        }

        // 查询动漫数据
        let animes = AnimeEntity::find()
            .filter(anime_condition)
            .order_by(AnimeColumn::AnilistId, Order::Asc)
            .all(self.conn())
            .await?;

        // 查询这些动漫关联的映射
        let mut mapping_condition = sea_orm::Condition::any();
        for id in &paged_anime_ids {
            mapping_condition = mapping_condition.add(AnimeMappingColumn::AnilistId.eq(*id));
        }

        let mut mappings_query = AnimeMappingEntity::find().filter(mapping_condition);

        // 如果指定了审核状态，继续应用过滤
        if let Some(ref status) = query.status {
            mappings_query =
                mappings_query.filter(AnimeMappingColumn::ReviewStatus.eq(status.clone()));
        }

        let mappings = mappings_query.all(self.conn()).await?;

        // 将映射按动漫ID分组
        let mut mapping_map: HashMap<i32, Vec<AnimeMapping>> = HashMap::new();
        for mapping in mappings {
            mapping_map
                .entry(mapping.anilist_id)
                .or_insert_with(Vec::new)
                .push(mapping);
        }

        // 组装最终结果
        let result = animes
            .into_iter()
            .map(|anime| {
                let anime_mappings = mapping_map.remove(&anime.anilist_id).unwrap_or_default();
                (anime, anime_mappings)
            })
            .collect();

        Ok(Pagination {
            page,
            page_size,
            total,
            data: result,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::anime::Model as Anime;
    use crate::models::enums::{MediaType, Platform, ReviewStatus};
    use crate::models::mappings::Model as AnimeMapping;

    #[tokio::test]
    async fn test_batch_add_animes() {
        let db = DB::new_for_test().await.unwrap();
        let animes = vec![Anime {
            anilist_id: 1,
            media_type: MediaType::TV,
            titles: "test".to_string(),
            year: 2024,
            season: Some("spring".to_string()),
            start_date: Some("2024-01-01".to_string()),
            episode_count: Some(12),
            season_number: Some(1),
            episode_number: Some(1),
            absolute_episode_number: Some(1),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];
        let mappings = vec![AnimeMapping {
            anilist_id: 1,
            platform: Platform::BgmTv,
            platform_id: Some("123".to_string()),
            review_status: ReviewStatus::UnMatched,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            score: 10,
        }];
        db.batch_add_animes((animes, mappings)).await.unwrap();

        let (anime, mappings) = db.get_anime(1).await.unwrap();
        assert_eq!(anime.unwrap().anilist_id, 1);
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].anilist_id, 1);

        db.review(1, Platform::BgmTv, ReviewStatus::Ready)
            .await
            .unwrap();

        let (anime, mappings) = db.get_anime(1).await.unwrap();
        assert_eq!(anime.unwrap().anilist_id, 1);
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].review_status, ReviewStatus::Ready);
    }
}

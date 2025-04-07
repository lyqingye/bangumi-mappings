use crate::db::mappings::{Column, Entity as Mappings, Model as Mapping, VerificationStatus};
use anyhow::Result;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect};
use serde::{Deserialize, Serialize};

use super::Db;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryMappings {
    pub page: u32,
    pub page_size: u32,
    pub status: Option<VerificationStatus>,
    pub year: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

impl Db {
    pub async fn get_by_anilist_id(&self, anilist_id: i32) -> Result<Option<Mapping>> {
        let mapping = Mappings::find_by_id(anilist_id).one(self.conn()).await?;
        Ok(mapping)
    }

    pub async fn list_tobe_match(&self, limit: u64, year: u32) -> Result<Vec<Mapping>> {
        let mappings = Mappings::find()
            .filter(
                Column::BgmTvVerifyStatus
                    .eq(VerificationStatus::UnMatched)
                    .or(Column::BgmTvVerifyStatus.eq(VerificationStatus::Rejected))
                    .or(Column::TmdbVerifyStatus.eq(VerificationStatus::UnMatched))
                    .or(Column::TmdbVerifyStatus.eq(VerificationStatus::Rejected)),
            )
            .filter(Column::AirYear.eq(year))
            .limit(limit)
            .all(self.conn())
            .await?;
        Ok(mappings)
    }

    pub async fn query_mappings(
        &self,
        query_params: &QueryMappings,
    ) -> Result<QueryResult<Mapping>> {
        let mut query = Mappings::find();

        if let Some(status) = &query_params.status {
            match status {
                VerificationStatus::Accepted => {
                    query = query.filter(
                        Column::BgmTvVerifyStatus
                            .eq(status.clone())
                            .or(Column::TmdbVerifyStatus.eq(status.clone())),
                    );
                }
                VerificationStatus::Rejected => {
                    query = query.filter(
                        Column::BgmTvVerifyStatus
                            .eq(status.clone())
                            .or(Column::TmdbVerifyStatus.eq(status.clone())),
                    );
                }
                VerificationStatus::Dropped => {
                    query = query.filter(
                        Column::BgmTvVerifyStatus
                            .eq(status.clone())
                            .or(Column::TmdbVerifyStatus.eq(status.clone())),
                    );
                }
                VerificationStatus::Ready => {
                    query = query.filter(
                        Column::BgmTvVerifyStatus
                            .eq(status.clone())
                            .or(Column::TmdbVerifyStatus.eq(status.clone())),
                    );
                }
                VerificationStatus::UnMatched => {
                    query = query.filter(
                        Column::BgmTvVerifyStatus
                            .eq(status.clone())
                            .or(Column::TmdbVerifyStatus.eq(status.clone())),
                    );
                }
            }
        }

        if let Some(year) = &query_params.year {
            query = query.filter(Column::AirYear.eq(*year));
        }

        // 获取总记录数
        let total = query.clone().count(self.conn()).await?;

        // 构建分页查询
        let page = query_params.page;
        let page_size = query_params.page_size;
        let offset = (page - 1) * page_size;

        let data = query
            .offset(offset as u64)
            .limit(page_size as u64)
            .all(self.conn())
            .await?;

        Ok(QueryResult {
            data,
            total,
            page,
            page_size,
        })
    }
}

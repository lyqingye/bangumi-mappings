use serde::{Deserialize, Serialize};

use crate::models::enums::{Platform, ReviewStatus};

#[derive(Debug, Serialize, Deserialize)]
pub struct Resp<T> {
    pub code: i32,
    pub msg: Option<String>,
    pub data: Option<T>,
}

impl<T> Resp<T> {
    pub fn ok(data: Option<T>) -> Self {
        Self {
            code: 0,
            msg: None,
            data,
        }
    }

    pub fn err(msg: Option<String>) -> Self {
        Self {
            code: 1,
            msg,
            data: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryAnimes {
    #[serde(flatten)]
    pub query: PageQuery,
    pub year: Option<i32>,
    pub status: Option<ReviewStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageQuery {
    pub page: usize,
    pub page_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination<T> {
    pub page: usize,
    pub page_size: usize,
    pub total: usize,
    pub data: Vec<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Anime {
    pub anilist_id: i32,
    pub titles: Vec<String>,
    pub year: i32,
    pub mappings: Vec<Mapping>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mapping {
    pub id: Option<String>,
    pub review_status: ReviewStatus,
    pub platform: Platform,
    pub score: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactMapping {
    pub id: Option<String>,
    pub platform: Platform,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactAnime {
    pub anilist_id: i32,
    pub titles: Vec<String>,
    pub year: i32,
    pub start_date: Option<String>,
    pub season_number: Option<i32>,
    pub mappings: Vec<CompactMapping>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManualMappingRequest {
    pub anilist_id: i32,
    pub platform: Platform,
    pub platform_id: String,
    pub season_number: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub total_animes: usize,
    pub total_tmdb_matched: usize,
    pub total_tmdb_unmatched: usize,
    pub total_tmdb_dropped: usize,
    pub total_bgmtv_matched: usize,
    pub total_bgmtv_unmatched: usize,
    pub total_bgmtv_dropped: usize,
}

/// 单个年份的统计数据
#[derive(Debug, Serialize, Deserialize)]
pub struct YearStatistic {
    pub year: i32,
    pub total_animes: usize,
    pub tmdb_matched: usize,
    pub tmdb_unmatched: usize,
    pub tmdb_dropped: usize,
    pub bgmtv_matched: usize,
    pub bgmtv_unmatched: usize,
    pub bgmtv_dropped: usize,
}

/// 所有年份统计数据的集合
#[derive(Debug, Serialize, Deserialize)]
pub struct YearStatistics {
    pub statistics: Vec<YearStatistic>,
}

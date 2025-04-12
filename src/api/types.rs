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

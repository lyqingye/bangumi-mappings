#![allow(unused)]

use anyhow::{Result, anyhow};
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
};
use nonzero_ext::nonzero;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fs::File, io::copy, path::Path, sync::Arc, time::Duration};
use tokio::{fs::File as TokioFile, io::AsyncWriteExt, time::sleep};
use tracing::{info, warn};

const ANILIST_API_URL: &str = "https://graphql.anilist.co";
const ANILIST_MEDIA_LIST_QUERY: &str = r#"
query($page:Int = 1, $type:MediaType, $year:String, $format:[MediaFormat]) {
  Page(page:$page,perPage:50) {
    pageInfo {
      total
      perPage
      currentPage
      lastPage
      hasNextPage
    }
    media(
      type:$type
      format_in:$format
      startDate_like:$year
    ) {
      id
      type
      season
      seasonYear
      title {
        english
        native
        romaji
      }
      startDate {
        year
        month
        day
      }
    }
  }
}
"#;

const ANILIST_MEDIA_QUERY: &str = r#"
query ($id: Int) {
  Media(id: $id, type: ANIME) {
    id
    title {
      romaji
      english
      native
      userPreferred
    }
    description(asHtml: false)
    coverImage {
      large
      medium
      color
    }
    bannerImage
    season
    seasonYear
    format
    status
    episodes
    duration
    genres
    tags {
      id
      name
      category
      rank
    }
    averageScore
    meanScore
    popularity
    studios {
      nodes {
        id
        name
        isAnimationStudio
      }
    }
    startDate {
      year
      month
      day
    }
    endDate {
      year
      month
      day
    }
    source
  }
}
"#;

// 1分钟20次请求的限流配置
const RATE_LIMIT: u32 = 20;
const RATE_LIMIT_WINDOW: u64 = 60;
const RETRY_WAIT_TIME: Duration = Duration::from_secs(60);
const MAX_RETRIES: u32 = 3;

pub struct AniListClient {
    client: reqwest::Client,
    timeout: Duration,
    rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl AniListClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            timeout: Duration::from_secs(60),
            rate_limiter: Arc::new(RateLimiter::direct(Quota::per_minute(nonzero!(RATE_LIMIT)))),
        }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            client: reqwest::Client::new(),
            timeout,
            rate_limiter: Arc::new(RateLimiter::direct(Quota::per_minute(nonzero!(RATE_LIMIT)))),
        }
    }

    async fn handle_rate_limit_and_retry<T, F, Fut>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut retries = 0;
        loop {
            match operation().await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if let Some(err) = e.downcast_ref::<reqwest::Error>() {
                        if let Some(status) = err.status() {
                            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                                if retries >= MAX_RETRIES {
                                    return Err(anyhow!("达到最大重试次数 ({MAX_RETRIES})"));
                                }
                                warn!("请求被限流，等待{}秒后重试...", RETRY_WAIT_TIME.as_secs());
                                sleep(RETRY_WAIT_TIME).await;
                                retries += 1;
                                continue;
                            }
                        }
                    }
                    return Err(e);
                }
            }
        }
    }

    async fn send_query<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T> {
        self.handle_rate_limit_and_retry(|| async {
            // 等待限流器允许请求
            self.rate_limiter.until_ready().await;

            let response = self
                .client
                .post(ANILIST_API_URL)
                .timeout(self.timeout)
                .json(&json!({
                    "query": query,
                    "variables": variables
                }))
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(anyhow!("API返回错误状态码: {}", response.status()));
            }

            let response_data = response.json().await?;
            Ok(response_data)
        })
        .await
    }

    pub async fn get_anime(&self, id: i32) -> Result<AniListMediaDetail> {
        let variables = json!({
            "id": id
        });

        let response: MediaResponse = self.send_query(ANILIST_MEDIA_QUERY, variables).await?;
        Ok(response.data.media)
    }

    pub async fn query_media_list(
        &self,
        page: i32,
        per_page: i32,
        media_type: &str,
        year: i32,
        format: &[&str],
    ) -> Result<AniListResponse> {
        let variables = json!({
            "page": page,
            "perPage": per_page,
            "type": media_type,
            "year": format!("{}%", year),
            "format": format,
        });

        self.send_query(ANILIST_MEDIA_LIST_QUERY, variables).await
    }

    pub async fn download_image(
        &self,
        anime: &AniListMediaDetail,
        path: &str,
    ) -> Result<Option<String>> {
        let image_url = anime.cover_image.large.clone().unwrap_or(
            anime
                .cover_image
                .medium
                .clone()
                .unwrap_or(anime.cover_image.color.clone().unwrap_or_default()),
        );

        if image_url.is_empty() {
            return Ok(None);
        }

        // 从URL中提取文件扩展名
        let ext = image_url
            .split('.')
            .last()
            .and_then(|ext| {
                let ext = ext.split('?').next().unwrap_or(ext);
                match ext.to_lowercase().as_str() {
                    "jpg" | "jpeg" | "png" | "gif" | "webp" => Some(ext.to_lowercase()),
                    _ => None,
                }
            })
            .unwrap_or("jpg".to_string());

        // 构建文件名
        let filename = format!("{}.{}", anime.id, ext);
        // 构建完整路径
        let full_path = Path::new(path).join(&filename);

        if full_path.exists() {
            return Ok(Some(filename));
        }

        // 创建目标路径的父目录（如果不存在）
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        self.handle_rate_limit_and_retry(|| async {
            // 等待限流器允许请求
            self.rate_limiter.until_ready().await;

            // 下载图片
            let response = self
                .client
                .get(&image_url)
                .timeout(self.timeout)
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(anyhow!("下载图片失败: {}", response.status()));
            }

            // 将图片内容写入文件
            let mut file = TokioFile::create(&full_path).await?;
            let bytes = response.bytes().await?;
            file.write_all(&bytes).await?;

            Ok(Some(filename.clone()))
        })
        .await
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AniListDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AniListTitle {
    pub english: Option<String>,
    pub native: Option<String>,
    pub romaji: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AniListMedia {
    pub id: i32,
    #[serde(rename = "type")]
    pub media_type: String,
    pub season: Option<String>,
    #[serde(rename = "seasonYear")]
    pub season_year: Option<i32>,
    pub title: AniListTitle,
    #[serde(rename = "startDate")]
    pub start_date: AniListDate,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AniListMediaListItem {
    pub media: AniListMedia,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PageInfo {
    pub total: Option<i32>,
    #[serde(rename = "perPage")]
    pub per_page: i32,
    #[serde(rename = "currentPage")]
    pub current_page: i32,
    #[serde(rename = "lastPage")]
    pub last_page: i32,
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AniListPage {
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
    pub media: Vec<AniListMedia>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AniListData {
    #[serde(rename = "Page")]
    pub page: AniListPage,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AniListResponse {
    pub data: AniListData,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CoverImage {
    pub large: Option<String>,
    pub medium: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub rank: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Studio {
    pub id: i32,
    pub name: String,
    #[serde(rename = "isAnimationStudio")]
    pub is_animation_studio: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Studios {
    pub nodes: Vec<Studio>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AniListMediaDetail {
    pub id: i32,
    pub title: AniListTitle,
    pub description: Option<String>,
    #[serde(rename = "coverImage")]
    pub cover_image: CoverImage,
    #[serde(rename = "bannerImage")]
    pub banner_image: Option<String>,
    pub season: Option<String>,
    #[serde(rename = "seasonYear")]
    pub season_year: Option<i32>,
    pub format: Option<String>,
    pub status: Option<String>,
    pub episodes: Option<i32>,
    pub duration: Option<i32>,
    pub genres: Vec<String>,
    pub tags: Vec<Tag>,
    #[serde(rename = "averageScore")]
    pub average_score: Option<i32>,
    #[serde(rename = "meanScore")]
    pub mean_score: Option<i32>,
    pub popularity: Option<i32>,
    pub studios: Studios,
    #[serde(rename = "startDate")]
    pub start_date: AniListDate,
    #[serde(rename = "endDate")]
    pub end_date: AniListDate,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MediaResponse {
    data: MediaData,
}

#[derive(Debug, Deserialize, Serialize)]
struct MediaData {
    #[serde(rename = "Media")]
    media: AniListMediaDetail,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;

    #[tokio::test]
    async fn test_get_anime() {
        let client = AniListClient::new();
        let anime = client.get_anime(1).await.unwrap();
        println!("{:?}", anime);
    }

    #[tokio::test]
    async fn test_query_media_list() {
        let client = AniListClient::new();
        let response = client
            .query_media_list(1, 50, "ANIME", 2024, &["TV"])
            .await
            .unwrap();
        println!("{:?}", response);
    }

    #[tokio::test]
    async fn test_download_image() {
        let client = AniListClient::new();
        let anime = client.get_anime(1).await.unwrap();

        let test_dir = "./assets/";
        let filename = client.download_image(&anime, test_dir).await.unwrap();

        assert!(filename.is_some());
        let filename = filename.unwrap();

        // 验证文件是否存在且大小大于0
        let full_path = Path::new(test_dir).join(filename);
        let metadata = fs::metadata(&full_path).await.unwrap();
        assert!(metadata.len() > 0);

        // 清理测试文件
        let _ = fs::remove_file(&full_path).await;
    }
}

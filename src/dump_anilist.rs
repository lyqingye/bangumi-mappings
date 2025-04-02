use anyhow::{Result, anyhow};
use governor::{Quota, RateLimiter};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashSet, fs::File, num::NonZero};
use tracing::{info, warn};

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

async fn query_anilist_media_list(
    page: i32,
    per_page: i32,
    media_type: &str,
    year: i32,
    format: &[&str],
) -> Result<AniListResponse> {
    let client = reqwest::Client::new();

    // 添加超时设置，避免无限等待
    let response = client
        .post("https://graphql.anilist.co")
        .timeout(std::time::Duration::from_secs(30))
        .json(&json!({
            "query": ANILIST_MEDIA_LIST_QUERY,
            "variables": {
                "page": page,
                "perPage": per_page,
                "type": media_type,
                "year": format!("{}%", year),
                "format": format,
            }
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!("API返回错误状态码: {}", response.status()));
    }

    let anilist_response: AniListResponse = response.json().await?;
    Ok(anilist_response)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DumpedMediaList {
    pub media_list: Vec<AniListMedia>,
    pub set: HashSet<i32>,
}

impl DumpedMediaList {
    pub fn load_from_file(year: i32) -> Result<Self> {
        let dump_file = format!("./dump/anilist_{}.json", year);
        match File::open(&dump_file) {
            Ok(file) => {
                let media_list: Vec<AniListMedia> = serde_json::from_reader(file)?;
                let mut set = HashSet::new();
                for media in media_list.iter() {
                    set.insert(media.id);
                }
                Ok(DumpedMediaList { media_list, set })
            }
            Err(_) => {
                // 如果文件不存在，创建一个新的
                Ok(DumpedMediaList {
                    media_list: Vec::new(),
                    set: HashSet::new(),
                })
            }
        }
    }

    pub fn save_to_file(&self, year: i32) -> Result<()> {
        let dump_file = format!("./dump/anilist_{}.json", year);
        // 先写入临时文件，成功后再重命名，避免文件损坏
        let temp_file = format!("{}.tmp", dump_file);
        let file = File::create(&temp_file)?;
        serde_json::to_writer(file, &self.media_list)?;
        std::fs::rename(temp_file, dump_file)?;
        Ok(())
    }

    pub fn add_media(&mut self, media: AniListMedia) {
        if self.set.contains(&media.id) {
            return;
        }
        self.set.insert(media.id);
        self.media_list.push(media);
    }
}

async fn dump_anilist_media_list(year: i32) -> Result<()> {
    let quota = Quota::per_second(NonZero::new(10).unwrap()).allow_burst(NonZero::new(1).unwrap());
    let limiter = RateLimiter::direct(quota);

    // 总是从第1页开始
    let mut page = 1;

    // 加载已有的dump数据
    let mut dumped_data = DumpedMediaList::load_from_file(year)?;

    // 定义一些常量
    const MAX_RETRIES: u32 = 100;
    const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(60); // 1分钟

    // 定义支持的格式
    let formats = &["TV", "MOVIE", "TV_SHORT", "SPECIAL", "OVA"];

    // 保存间隔设置
    let save_interval = 1; // 每处理多少页保存一次，默认1页
    let mut pages_since_last_save = 0;

    println!("开始导出{}年AniList数据", year);

    loop {
        let mut retry_count = 0;
        let mut success = false;
        let mut response = None;

        // 重试机制
        while retry_count < MAX_RETRIES && !success {
            // 限速
            let _ = limiter.until_n_ready(NonZero::new(1).unwrap()).await;

            match query_anilist_media_list(page, 50, "ANIME", year, formats).await {
                Ok(res) => {
                    response = Some(res);
                    success = true;
                }
                Err(e) => {
                    retry_count += 1;
                    warn!(
                        "查询第{}页失败: {}，重试 {}/{}...",
                        page, e, retry_count, MAX_RETRIES
                    );

                    if retry_count < MAX_RETRIES {
                        info!("等待{}秒后重试...", RETRY_DELAY.as_secs());
                        tokio::time::sleep(RETRY_DELAY).await;
                    } else {
                        return Err(anyhow!("达到最大重试次数，无法继续"));
                    }
                }
            }
        }

        let response = response.unwrap();
        let data = response.data;

        // 处理本页数据
        for media in data.page.media {
            // 添加到dump数据
            dumped_data.add_media(media);
        }

        pages_since_last_save += 1;

        // 每处理save_interval页，保存一次数据
        if pages_since_last_save >= save_interval {
            dumped_data.save_to_file(year)?;
            info!("已处理{}页，已保存数据", page);
            pages_since_last_save = 0;
        }

        // 检查是否为最后一页
        if !data.page.page_info.has_next_page {
            info!("已到达最后一页，共处理{}页", page);
            // 最后一次保存
            if pages_since_last_save > 0 {
                dumped_data.save_to_file(year)?;
            }
            break;
        }

        // 下一页
        page += 1;
    }

    // 完成后保存最终结果
    info!(
        "数据导出完成，共导出{}条{}年媒体数据",
        dumped_data.media_list.len(),
        year
    );

    Ok(())
}

// 修改主函数以接受年份参数
pub async fn run_dump_anilist(start: i32, end: i32) -> Result<()> {
    for year in start..=end {
        info!("开始导出{}年AniList数据...", year);
        dump_anilist_media_list(year).await?;
    }
    Ok(())
}

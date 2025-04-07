use std::sync::Arc;

use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tmdb_api::{
    Client,
    client::reqwest::ReqwestExecutor,
    movie::{MovieShort, search::MovieSearch},
    prelude::Command,
    tvshow::{
        TVShowShort,
        details::TVShowDetails,
        episode::groups::{TVShowEpisodeGroups, TVShowEpisodeGroupsDetails},
        search::TVShowSearch,
    },
};
use tokio_retry::{Retry, strategy::FixedInterval};
use tracing::info;

pub struct TMDBSearchTool {
    client: Arc<Client<ReqwestExecutor>>,
}

impl TMDBSearchTool {
    pub fn new() -> Self {
        let client = new_tmdb_client();
        Self {
            client: Arc::new(client),
        }
    }
}

#[derive(Deserialize)]
pub struct TMDBSearchArgs {
    query: String,
    year: Option<i16>,
}

#[derive(Debug, thiserror::Error, Serialize)]
#[error("{message}")]
pub struct TMDBError {
    message: String,
}

impl TMDBError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct TMDBSearchResult {
    #[serde(default)]
    pub data: Vec<TVShowShort>,
}

impl Tool for TMDBSearchTool {
    const NAME: &'static str = "tmdb_search_tv_show";

    type Error = TMDBError;
    type Args = TMDBSearchArgs;
    type Output = TMDBSearchResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "tmdb_search_tv_show".to_string(),
            description: "Search for TV shows on TMDB".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query for TV shows"
                    },
                    // "year": {
                    //     "type": "number",
                    //     "description": "(Optional) The year for the search, example: 2024"
                    // }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 在每次调用时创建新的client实例
        let client = self.client.clone();

        // 使用spawn_blocking来处理阻塞操作
        let query = args.query;
        tokio::spawn(async move {
            let cmd = TVShowSearch::new(query)
                .with_language(Some("zh-CN".to_string()))
                .with_year(args.year.map(|item| item as u16));

            // 使用tokio-retry实现重试逻辑
            let retry_strategy = FixedInterval::from_millis(5000).take(5);

            let result =
                Retry::spawn(retry_strategy, || async { cmd.execute(&client).await }).await;

            match result {
                Ok(result) => {
                    return Ok(TMDBSearchResult {
                        data: result.results,
                    });
                }
                Err(e) => {
                    info!("搜索失败，已重试多次: {}", e);
                    Err(TMDBError::new(format!("搜索失败，已重试多次: {}", e)))
                }
            }
        })
        .await
        .unwrap_or(Err(TMDBError::new("search not found")))
    }
}

#[derive(Deserialize, Serialize)]
pub struct TMDBSeasonArgs {
    tv_id: u64,
}

fn new_tmdb_client() -> Client<ReqwestExecutor> {
    let api_key = std::env::var("TMDB_API_KEY").expect("TMDB_API_KEY not set");
    let http_proxy = std::env::var("HTTP_PROXY");
    let https_proxy = std::env::var("HTTPS_PROXY");

    let mut client_builder = reqwest::Client::builder();

    if let Ok(http_proxy) = http_proxy {
        client_builder = client_builder.proxy(reqwest::Proxy::http(http_proxy).unwrap());
    }

    if let Ok(https_proxy) = https_proxy {
        client_builder = client_builder.proxy(reqwest::Proxy::https(https_proxy).unwrap());
    }

    let client = client_builder.build().unwrap();

    let reqwest_executor = ReqwestExecutor::from(client);
    tmdb_api::Client::builder()
        .with_api_key(api_key)
        .with_base_url("https://api.themoviedb.org/3")
        .with_executor(reqwest_executor)
        .build()
        .unwrap()
}

pub struct TMDBMovieSearchTool {
    client: Arc<Client<ReqwestExecutor>>,
}

impl TMDBMovieSearchTool {
    pub fn new() -> Self {
        let client = new_tmdb_client();
        Self {
            client: Arc::new(client),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct TMDBMovieSearchResult {
    #[serde(default)]
    pub data: Vec<MovieShort>,
}

#[derive(Deserialize, Serialize)]
pub struct TMDBMovieSearchArgs {
    query: String,
    year: Option<i16>,
}

impl Tool for TMDBMovieSearchTool {
    const NAME: &'static str = "tmdb_search_movie";

    type Error = TMDBError;
    type Args = TMDBMovieSearchArgs;
    type Output = TMDBMovieSearchResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "tmdb_search_movie".to_string(),
            description: "Search for movies on TMDB".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query for movies"
                    },
                    // "year": {
                    //     "type": "number",
                    //     "description": "(Optional) The year for the search, example: 2024"
                    // }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 在每次调用时创建新的client实例
        let client = self.client.clone();

        // 使用spawn_blocking来处理阻塞操作
        let query = args.query;
        tokio::spawn(async move {
            let cmd = MovieSearch::new(query)
                .with_language(Some("zh-CN".to_string()))
                .with_year(args.year.map(|item| item as u16));

            // 使用tokio-retry实现重试逻辑
            let retry_strategy = FixedInterval::from_millis(5000).take(5);

            let result =
                Retry::spawn(retry_strategy, || async { cmd.execute(&client).await }).await;

            match result {
                Ok(result) => {
                    return Ok(TMDBMovieSearchResult {
                        data: result.results,
                    });
                }
                Err(e) => {
                    info!("搜索失败，已重试多次: {}", e);
                    Err(TMDBError::new(format!("搜索失败，已重试多次: {}", e)))
                }
            }
        })
        .await
        .unwrap_or(Err(TMDBError::new("search not found")))
    }
}

/// ------------------------------------------------
pub struct TMDBSeasonTool {
    client: Arc<Client<ReqwestExecutor>>,
}

impl TMDBSeasonTool {
    pub fn new() -> Self {
        let client = new_tmdb_client();
        Self {
            client: Arc::new(client),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Season {
    pub id: String,
    pub name: String,
    pub number: i32,
    pub first_air_date: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct TMDBSeasonResult {
    #[serde(default)]
    pub data: Vec<Season>,
}

impl Tool for TMDBSeasonTool {
    const NAME: &'static str = "tmdb_season";

    type Error = TMDBError;
    type Args = TMDBSeasonArgs;
    type Output = TMDBSeasonResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "tmdb_season".to_string(),
            description: "Get Tv seasons detail".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "tv_id": {
                        "type": "number",
                        "description": "The TMDB ID of the TV show"
                    }
                },
                "required": ["tv_id"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 保存参数为本地变量
        let tv_id = args.tv_id;

        let client = self.client.clone();
        tokio::spawn(async move {
            let retry_strategy = FixedInterval::from_millis(5000).take(5);
            let cmd = TVShowEpisodeGroups::new(tv_id);

            let ep_groups = match Retry::spawn(retry_strategy.clone(), || async {
                cmd.execute(&client).await
            })
            .await
            {
                Ok(result) => result,
                Err(e) => return Err(TMDBError::new(format!("获取剧集组失败: {}", e))),
            };

            let mut group_id = None;
            for item in &ep_groups.results {
                if item.group_type == 6 {
                    group_id = Some(item.id.clone());
                }
            }

            if group_id.is_none() && ep_groups.results.len() > 0 {
                group_id = Some(ep_groups.results[0].id.clone());
            }

            let mut seasons = vec![];

            if let Some(group_id) = group_id {
                info!("获取季度详情: {}", group_id);
                let cmd_details = TVShowEpisodeGroupsDetails::new(group_id)
                    .with_language(Some("zh-CN".to_string()));

                let details = match Retry::spawn(retry_strategy.clone(), || async {
                    cmd_details.execute(&client).await
                })
                .await
                {
                    Ok(result) => result,
                    Err(e) => return Err(TMDBError::new(format!("获取季度详情失败: {}", e))),
                };

                if !details.groups.is_empty() {
                    seasons = details
                        .groups
                        .iter()
                        .map(|item| Season {
                            id: item.id.clone(),
                            name: item.name.clone(),
                            number: item.order as i32,
                            first_air_date: item
                                .episodes
                                .first()
                                .map(|item| item.air_date.to_string()),
                        })
                        .collect::<Vec<Season>>();
                }
            }

            let cmd_details = TVShowDetails::new(tv_id).with_language(Some("zh-CN".to_string()));

            let tv_details = match Retry::spawn(retry_strategy.clone(), || async {
                cmd_details.execute(&client).await
            })
            .await
            {
                Ok(result) => result,
                Err(e) => return Err(TMDBError::new(format!("获取TV详情失败: {}", e))),
            };

            seasons.extend(
                tv_details
                    .seasons
                    .iter()
                    .map(|item| Season {
                        id: item.inner.id.to_string(),
                        name: item.inner.name.clone(),
                        number: item.inner.season_number as i32,
                        first_air_date: item.inner.air_date.map(|item| item.to_string()),
                    })
                    .collect::<Vec<Season>>(),
            );

            Ok(TMDBSeasonResult { data: seasons })
        })
        .await
        .unwrap_or(Err(TMDBError::new("season not found")))
    }
}

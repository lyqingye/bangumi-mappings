use std::sync::Arc;

use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tmdb_api::{
    Client,
    client::reqwest::ReqwestExecutor,
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
        let api_key = std::env::var("TMDB_API_KEY").expect("TMDB_API_KEY not set");
        let client = Client::new(api_key);
        Self {
            client: Arc::new(client),
        }
    }
}

#[derive(Deserialize)]
pub struct TMDBSearchArgs {
    query: String,
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

impl Tool for TMDBSearchTool {
    const NAME: &'static str = "tmdb_search_tv_show";

    type Error = TMDBError;
    type Args = TMDBSearchArgs;
    type Output = Vec<TVShowShort>;

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
                    }
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
            let cmd = TVShowSearch::new(query).with_language(Some("zh-CN".to_string()));

            // 使用tokio-retry实现重试逻辑
            let retry_strategy = FixedInterval::from_millis(5000).take(5);

            let result =
                Retry::spawn(retry_strategy, || async { cmd.execute(&client).await }).await;

            match result {
                Ok(result) => Ok(result.results),
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

pub struct TMDBSeasonTool {
    client: Arc<Client<ReqwestExecutor>>,
}

impl TMDBSeasonTool {
    pub fn new() -> Self {
        let api_key = std::env::var("TMDB_API_KEY").expect("TMDB_API_KEY not set");
        let client = Client::new(api_key);
        Self {
            client: Arc::new(client),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Season {
    pub id: String,
    pub name: String,
    pub number: u64,
    pub first_air_date: Option<String>,
}

impl Tool for TMDBSeasonTool {
    const NAME: &'static str = "tmdb_season";

    type Error = TMDBError;
    type Args = TMDBSeasonArgs;
    type Output = Vec<Season>;

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

            if let Some(group_id) = group_id {
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

                return Ok(details
                    .groups
                    .iter()
                    .map(|item| Season {
                        id: item.id.clone(),
                        name: item.name.clone(),
                        number: item.order,
                        first_air_date: item.episodes.first().map(|item| item.air_date.to_string()),
                    })
                    .collect::<Vec<Season>>());
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

            Ok(tv_details
                .seasons
                .iter()
                .map(|item| Season {
                    id: item.inner.id.to_string(),
                    name: item.inner.name.clone(),
                    number: item.inner.season_number,
                    first_air_date: item.inner.air_date.map(|item| item.to_string()),
                })
                .collect::<Vec<Season>>())
        })
        .await
        .unwrap_or(Err(TMDBError::new("season not found")))
    }
}

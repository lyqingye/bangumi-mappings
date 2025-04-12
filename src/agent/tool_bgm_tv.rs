use chrono::NaiveDate;
use reqwest::header::USER_AGENT;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_retry::{Retry, strategy::FixedInterval};
#[derive(Deserialize, Serialize)]
pub struct BgmTVSearchArgs {
    query: String,
    start_date: Option<String>,
    end_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Pagination {
    pub total: i32,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct PageResponse<T> {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub data: Vec<T>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(default)]
pub struct Subject {
    pub id: i32,
    #[serde(rename = "type")]
    pub subject_type: i32,
    pub name: String,
    pub name_cn: Option<String>,
    pub series: bool,
    pub date: Option<NaiveDate>,
    pub eps: i32,
    pub total_episodes: i32,
    pub infobox: Vec<InfoboxItem>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct InfoboxItem {
    pub key: String,
    #[serde(default)]
    pub value: InfoboxValue,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum InfoboxValue {
    String(String),
    Array(Vec<InfoboxArrayItem>),
}

impl Default for InfoboxValue {
    fn default() -> Self {
        InfoboxValue::String(String::new())
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct InfoboxArrayItem {
    pub v: String,
}

pub struct BgmTVSearchTool {
    client: reqwest::Client,
    base_url: String,
}

impl BgmTVSearchTool {
    pub fn new() -> Self {
        let base_url = std::env::var("BGM_API_URL").unwrap_or("https://api.bgm.tv".to_string());
        let client = reqwest::Client::new();
        Self { client, base_url }
    }
}

#[derive(Debug, thiserror::Error, Serialize)]
#[error("{message}")]
pub struct BgmTVError {
    message: String,
}

impl BgmTVError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Tool for BgmTVSearchTool {
    const NAME: &'static str = "bgm_tv_search";

    type Error = BgmTVError;
    type Args = BgmTVSearchArgs;
    type Output = PageResponse<Subject>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "bgm_tv_search".to_string(),
            description: "Search for TV shows on BgmTV".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query for bgm tv"
                    },
                    "start_air_year": {
                        "type": "string",
                        "description": "The start year for the search, example: 2024"
                    },
                },
                "required": ["query", "start_air_year"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 保存参数为本地变量
        let query = args
            .query
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join("+");
        let base_url = self.base_url.clone();
        let client = self.client.clone();

        // 使用spawn_blocking来处理阻塞操作
        tokio::spawn(async move {
            let url = format!("{}/v0/search/subjects", base_url);

            let mut date_list = vec![];
            if let Some(start_date) = args.start_date {
                date_list.push(format!(">={}", start_date));
            }
            // 创建搜索过滤器
            let search_query = json!({
                "keyword": query,
                "filter": {
                    "sort": "rank",
                    "nsfw": true,
                    "air_date": date_list,
                }
            });

            let body = match serde_json::to_string(&search_query) {
                Ok(b) => b,
                Err(e) => return Err(BgmTVError::new(format!("JSON序列化错误: {}", e))),
            };

            // 使用tokio-retry实现重试逻辑
            let retry_strategy = FixedInterval::from_millis(5000).take(5);

            let response = match Retry::spawn(retry_strategy, || async {
                client
                    .post(&url)
                    .header(USER_AGENT, "lyqingye/anime-matcher-agent")
                    .query(&[("limit", "10"), ("offset", "0")])
                    .body(body.clone())
                    .send()
                    .await
            })
            .await
            {
                Ok(resp) => resp,
                Err(e) => return Err(BgmTVError::new(format!("请求错误，已重试多次: {}", e))),
            };

            let response_text = match response.text().await {
                Ok(text) => text,
                Err(e) => return Err(BgmTVError::new(format!("读取响应错误: {}", e))),
            };

            match serde_json::from_str::<PageResponse<Subject>>(&response_text) {
                Ok(mut resp) => {
                    resp.data.iter_mut().for_each(|item| {
                        item.infobox.retain(|item| {
                            item.key == "中文名" || item.key == "别名" || item.key == "英文名"
                        });
                    });
                    Ok(resp)
                }
                Err(e) => Err(BgmTVError::new(format!("解析响应错误: {}", e))),
            }
        })
        .await
        .unwrap_or(Err(BgmTVError::new("搜索失败")))
    }
}

use std::sync::Arc;

use chrono::NaiveDate;
use reqwest::header::USER_AGENT;
use rig::{
    OneOrMany,
    completion::{self, Completion, PromptError, ToolDefinition},
    message::{AssistantContent, Message, ToolCall, ToolFunction, ToolResultContent, UserContent},
    providers,
    tool::Tool,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tmdb_api::{
    Client,
    client::reqwest::ReqwestExecutor,
    prelude::Command,
    tvshow::{
        TVShowShort,
        episode::groups::{TVShowEpisodeGroups, TVShowEpisodeGroupsDetails},
        search::TVShowSearch,
    },
};

use rig::providers::deepseek::DeepSeekCompletionModel;
use tracing::info;

pub struct AnimeMatcherAgent {
    agent: MultiTurnAgent<DeepSeekCompletionModel>,
    client: providers::deepseek::Client,
}

impl AnimeMatcherAgent {
    pub fn new() -> Self {
        let client = providers::deepseek::Client::from_env();

        let model = "deepseek-chat";

        // let tmdb_client = Arc::new(Client::<ReqwestExecutor>::new(
        //     std::env::var("TMDB_API_KEY").unwrap(),
        // ));

        // let search_tools = TMDBSearchTool {
        //     client: tmdb_client.clone(),
        // };

        // let season_tool = TMDBSeasonTool {
        //     client: tmdb_client.clone(),
        // };

        let bgm_search_tool = BgmTVSearchTool::new();

        let agent = client
        .agent(model)
        .preamble("你是一个智能助手，匹配用户查询的动漫信息, 用户会输入动漫的相关信息，最终你需要找到与用户查询信息最相似的动漫")
        // .append_preamble("1. 使用tmdb_search_tv_show工具搜索动漫, 你可能需要进行多次搜索，然后找到相似度最高的动漫")
        // .append_preamble("2. 使用tmdb_season工具获取季度信息，信息中包含季度信息，你需要匹配对应的季度信息")
        .append_preamble("1. 你也可以使用bgm_tv_search工具搜索动漫，你可能需要进行多次搜索，然后找到相似度最高的动漫")
        .append_preamble("2. 以Json方式返回数据: {\"id\": \"\", \"name\": \"\"},不要携带其它与json无关的信息")
        .max_tokens(8192)
        .temperature(0.2)
        // .tool(search_tools)
        // .tool(season_tool)
        .tool(bgm_search_tool)
        .build();

        // 创建多轮对话agent
        let multi_agent = MultiTurnAgent {
            agent,
            chat_history: Vec::new(),
        };
        Self {
            agent: multi_agent,
            client,
        }
    }

    pub async fn match_anime(&mut self, query: &str) -> anyhow::Result<MatchResult> {
        let result = self.agent.multi_turn_prompt(query).await?;
        let extract_agent = self
            .client
            .extractor::<MatchResult>("deepseek-chat")
            .preamble("提取匹配结果信息")
            .build();

        let extract_result: MatchResult = extract_agent.extract(&result).await?;
        Ok(extract_result)
    }
}

struct MultiTurnAgent<M: rig::completion::CompletionModel> {
    agent: rig::agent::Agent<M>,
    chat_history: Vec<completion::Message>,
}

impl<M: rig::completion::CompletionModel> MultiTurnAgent<M> {
    async fn multi_turn_prompt(
        &mut self,
        prompt: impl Into<Message> + Send,
    ) -> Result<String, PromptError> {
        let mut current_prompt: Message = prompt.into();

        loop {
            info!("当前提示: {:?}\n", current_prompt);

            let resp = self
                .agent
                .completion(current_prompt.clone(), self.chat_history.clone())
                .await?
                .send()
                .await?;

            let mut final_text = None;

            for content in resp.choice.into_iter() {
                match content {
                    AssistantContent::Text(text) => {
                        info!("中间响应: {:?}\n", text.text);
                        final_text = Some(text.text.clone());
                        self.chat_history.push(current_prompt.clone());
                        let response_message = Message::Assistant {
                            content: OneOrMany::one(AssistantContent::text(&text.text)),
                        };
                        self.chat_history.push(response_message);
                    }
                    AssistantContent::ToolCall(content) => {
                        self.chat_history.push(current_prompt.clone());
                        let tool_call_msg = AssistantContent::ToolCall(content.clone());
                        info!("工具调用: {:?}\n", tool_call_msg);
                        self.chat_history.push(Message::Assistant {
                            content: OneOrMany::one(tool_call_msg),
                        });

                        let ToolCall {
                            id,
                            function: ToolFunction { name, arguments },
                        } = content;

                        let tool_result =
                            self.agent.tools.call(&name, arguments.to_string()).await?;

                        current_prompt = Message::User {
                            content: OneOrMany::one(UserContent::tool_result(
                                id,
                                OneOrMany::one(ToolResultContent::text(tool_result)),
                            )),
                        };

                        final_text = None;
                        break;
                    }
                }
            }

            if let Some(text) = final_text {
                return Ok(text);
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct MatchResult {
    pub id: i32,
    pub name: String,
    // pub season: u64,
}

pub struct TMDBSearchTool {
    client: Arc<Client<ReqwestExecutor>>,
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

            // 添加重试逻辑
            let mut attempts = 0;
            let max_attempts = 10;

            while attempts < max_attempts {
                match cmd.execute(&client).await {
                    Ok(result) => return Ok(result.results),
                    Err(e) => {
                        attempts += 1;
                        if attempts >= max_attempts {
                            return Err(TMDBError::new(format!(
                                "搜索失败，已重试{}次: {}",
                                attempts, e
                            )));
                        }
                        info!(
                            "搜索错误: {}，等待10秒后重试 ({}/{})",
                            e, attempts, max_attempts
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    }
                }
            }

            Err(TMDBError::new("所有重试尝试均失败"))
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

        // 使用spawn_blocking来处理阻塞操作
        let client = self.client.clone();
        tokio::spawn(async move {
            let cmd = TVShowEpisodeGroups::new(tv_id);

            // 添加重试逻辑 - 第一个请求的重试
            let mut attempts = 0;
            let max_attempts = 10;
            let mut ep_groups = None;

            while attempts < max_attempts {
                match cmd.execute(&client).await {
                    Ok(result) => {
                        ep_groups = Some(result);
                        break;
                    }
                    Err(e) => {
                        attempts += 1;
                        if attempts >= max_attempts {
                            return Err(TMDBError::new(format!(
                                "获取剧集组失败，已重试{}次: {}",
                                attempts, e
                            )));
                        }
                        info!(
                            "获取剧集组错误: {}，等待10秒后重试 ({}/{})",
                            e, attempts, max_attempts
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    }
                }
            }

            let ep_groups = match ep_groups {
                Some(result) => result,
                None => return Err(TMDBError::new("获取剧集组失败，所有重试均失败")),
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

            if group_id.is_none() {
                return Err(TMDBError::new("season not found"));
            }

            let cmd2 = TVShowEpisodeGroupsDetails::new(group_id.unwrap())
                .with_language(Some("zh-CN".to_string()));

            // 添加重试逻辑 - 第二个请求的重试
            let mut attempts = 0;
            let max_attempts = 10;

            while attempts < max_attempts {
                match cmd2.execute(&client).await {
                    Ok(result) => {
                        return Ok(result
                            .groups
                            .iter()
                            .map(|item| Season {
                                id: item.id.clone(),
                                name: item.name.clone(),
                                number: item.order,
                                first_air_date: item
                                    .episodes
                                    .first()
                                    .map(|item| item.air_date.to_string()),
                            })
                            .collect::<Vec<Season>>());
                    }
                    Err(e) => {
                        attempts += 1;
                        if attempts >= max_attempts {
                            return Err(TMDBError::new(format!(
                                "获取季度详情失败，已重试{}次: {}",
                                attempts, e
                            )));
                        }
                        info!(
                            "获取季度详情错误: {}，等待10秒后重试 ({}/{})",
                            e, attempts, max_attempts
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    }
                }
            }

            Err(TMDBError::new("获取季度详情失败，所有重试均失败"))
        })
        .await
        .unwrap_or(Err(TMDBError::new("season not found")))
    }
}

#[derive(Deserialize, Serialize)]
pub struct BgmTVSearchArgs {
    query: String,
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
    pub meta_tags: Vec<String>,
}

pub struct BgmTVSearchTool {
    client: reqwest::Client,
    base_url: String,
}

impl BgmTVSearchTool {
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            base_url: "https://api.bgm.tv".to_string(),
        }
    }
}

impl Tool for BgmTVSearchTool {
    const NAME: &'static str = "bgm_tv_search";

    type Error = TMDBError;
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
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 保存参数为本地变量
        let query = args.query;
        let base_url = self.base_url.clone();
        let client = self.client.clone();

        // 使用spawn_blocking来处理阻塞操作
        tokio::spawn(async move {
            let url = format!("{}/v0/search/subjects", base_url);

            // 创建搜索过滤器
            let search_query = json!({ "keyword": query });

            let body = match serde_json::to_string(&search_query) {
                Ok(b) => b,
                Err(e) => return Err(TMDBError::new(format!("JSON序列化错误: {}", e))),
            };

            // 添加重试逻辑
            let mut attempts = 0;
            let max_attempts = 10;
            let mut response = None;

            while attempts < max_attempts {
                match client
                    .post(&url)
                    .header(USER_AGENT, "lyqingye/anime-matcher-agent")
                    .query(&[("limit", "10"), ("offset", "0")])
                    .body(body.clone())
                    .send()
                    .await
                {
                    Ok(resp) => {
                        response = Some(resp);
                        break;
                    }
                    Err(e) => {
                        attempts += 1;
                        if attempts >= max_attempts {
                            return Err(TMDBError::new(format!(
                                "请求错误，已重试{}次: {}",
                                attempts, e
                            )));
                        }
                        info!(
                            "请求错误: {}，等待10秒后重试 ({}/{})",
                            e, attempts, max_attempts
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    }
                }
            }

            let response = match response {
                Some(resp) => resp,
                None => return Err(TMDBError::new("所有重试尝试均失败")),
            };

            let response_text = match response.text().await {
                Ok(text) => text,
                Err(e) => return Err(TMDBError::new(format!("读取响应错误: {}", e))),
            };

            match serde_json::from_str::<PageResponse<Subject>>(&response_text) {
                Ok(resp) => Ok(resp),
                Err(e) => Err(TMDBError::new(format!("解析响应错误: {}", e))),
            }
        })
        .await
        .unwrap_or(Err(TMDBError::new("搜索失败")))
    }
}

use rig::{
    OneOrMany,
    completion::{self, Completion, PromptError},
    extractor::{Extractor, ExtractorBuilder},
    message::{AssistantContent, Message, ToolCall, ToolFunction, ToolResultContent, UserContent},
    tool::Tool,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use tracing::info;

use crate::{
    agent::tool_bgm_tv::BgmTVSearchTool,
    agent::tool_submit::{SubmitBGMTool, SubmitTool},
    agent::tool_tmdb::{TMDBMovieSearchTool, TMDBSearchTool, TMDBSeasonTool},
};

pub static MATCH_BGM_PROMPT: &str = r#"You are an intelligent assistant responsible for matching anime information on Bangumi based on user queries.
Your goal is to identify the single most relevant anime entry.

1.  **Analyze User Query**: Identify potential anime titles (jp, romaji, English, etc.) and other relevant keywords provided by the user.
2.  **Primary Search**: prioritizing the most promising keyword(s) for the search (usually the jp title, if available).
3.  **Evaluate Results**: Examine the search results. If a highly relevant match is found based on the title and other available information (from the search tool's return data), proceed to step 5.
4.  **Refine Search (If Necessary)**: If the initial search results are ambiguous or low quality, you may try searching again using alternative titles (e.g., romaji, English) or extracted keywords. **Only perform additional searches if the first attempt failed to yield a likely match.**
5.  **Select Confident Match**: Evaluate the similarity between the user query and each search result (considering titles, aliases, air dates, etc.). Select the entry with the **highest similarity**, **but only if this similarity meets a high confidence threshold**. 
6.  **Submit Result**: if found confident match, submit the matched id and name, and confidence-score, otherwise submit empty result.
"#;

pub static MATCH_TMDB_PROMPT: &str = r#"You are an intelligent assistant responsible for matching anime information on TMDB based on user queries, including identifying the correct season(TV show Only).
Your goal is to identify the single most relevant anime entry and its specific season.
You can process the anime TVshow or movie.

1.  **Analyze User Query**: Identify potential anime titles (jp, romaji, English, etc.). **Critically, extract the *main title* of the anime, separating it from any season-specific identifiers or subtitles (e.g., "Season 2", "Part 3", "Arc X"). Identify these season identifiers and other relevant keywords separately.**
2.  **Primary Search**: Construct a search query prioritizing the most promising *extracted main title* (usually the jp title, if available). **Do NOT include the identified season identifiers or subtitles (like "Season 2", "第二季") in this initial search query.**
3.  **Evaluate Search Results**: Calculate the confidence score of the each search result(considering the title, air date, overview, etc.).  If no promising TV show match is found, proceed to step 8.
4.  **Fetch Season Information**: with the TMDB ID of the most likely TV show match identified in the previous step. This tool will return a list of seasons with their names, numbers, and potentially air dates.
5.  **Match Season**: Compare the season information obtained with the season details mentioned or implied in the user query. Identify the single season that best matches the user's request. Consider season numbers, names, or potentially air dates if provided.
6.  **Refine Search (If Necessary)**: If the initial search results are ambiguous or low quality, you may try searching again using alternative titles (e.g., romaji, English) or extracted keywords. **Only perform additional searches if the first attempt failed to yield a likely match.**
7.  **Select Confident Match**: Based on the TV show match (Step 3) and the specific season match (Step 5), confirm if this combination represents a high-confidence match for the user's query. 
8.  **Submit Result**: if found confident match, submit the matched tv_id and name and season number, and confidence-score, otherwise submit empty result.
"#;

pub static EXTRACT_BGM_MATCH_RESULT_PROMPT: &str = r#"extract the id and name from the input text"#;
pub static EXTRACT_TMDB_MATCH_RESULT_PROMPT: &str =
    r#"extract the id and name and season from the input text"#;

pub struct AnimeMatcherAgent<M: rig::completion::CompletionModel> {
    agent: MultiTurnAgent<M>,
    extractor: Extractor<M, MatchResult>,
}

impl<M: rig::completion::CompletionModel> AnimeMatcherAgent<M> {
    pub async fn match_anime(&mut self, query: &str) -> anyhow::Result<MatchResult> {
        let result = self.agent.multi_turn_prompt(query).await?;

        info!("模型输出: {}", result);
        match serde_json::from_str::<MatchResult>(&result) {
            Ok(result) => Ok(result),
            Err(_) => {
                let extract_result = self.extractor.extract(&result).await?;
                Ok(extract_result)
            }
        }
    }
}

pub fn new_mapping_bgm_tv_agent<M: rig::completion::CompletionModel>(
    agent: rig::agent::AgentBuilder<M>,
    extractor: ExtractorBuilder<MatchResult, M>,
) -> AnimeMatcherAgent<M> {
    let agent = agent
        .preamble(MATCH_BGM_PROMPT)
        .max_tokens(8192)
        .temperature(0.2)
        .tool(BgmTVSearchTool::new())
        .tool(SubmitBGMTool {});
    let multi_agent = MultiTurnAgent {
        agent: agent.build(),
        chat_history: Vec::new(),
    };

    let extractor = extractor.preamble(EXTRACT_BGM_MATCH_RESULT_PROMPT).build();

    AnimeMatcherAgent {
        agent: multi_agent,
        extractor,
    }
}

pub fn new_mapping_tmdb_agent<M: rig::completion::CompletionModel>(
    agent: rig::agent::AgentBuilder<M>,
    extractor: ExtractorBuilder<MatchResult, M>,
) -> AnimeMatcherAgent<M> {
    let agent = agent
        .preamble(MATCH_TMDB_PROMPT)
        .max_tokens(8192)
        .temperature(0.2)
        .tool(TMDBMovieSearchTool::new())
        .tool(TMDBSearchTool::new())
        .tool(TMDBSeasonTool::new())
        .tool(SubmitTool::new());

    let multi_agent = MultiTurnAgent {
        agent: agent.build(),
        chat_history: Vec::new(),
    };

    let extractor = extractor.preamble(EXTRACT_TMDB_MATCH_RESULT_PROMPT).build();

    AnimeMatcherAgent {
        agent: multi_agent,
        extractor,
    }
}

struct MultiTurnAgent<M: rig::completion::CompletionModel> {
    agent: rig::agent::Agent<M>,
    chat_history: Vec<completion::Message>,
}

impl<M: rig::completion::CompletionModel> MultiTurnAgent<M> {
    async fn multi_turn_prompt(
        &mut self,
        _prompt: impl Into<Message> + Send,
    ) -> Result<String, PromptError> {
        let mut prompt: Message = _prompt.into();

        loop {
            info!("当前提示: {:?}\n", prompt);

            let resp = self
                .agent
                .completion(prompt.clone(), self.chat_history.clone())
                .await?
                .send()
                .await?;

            let mut output = None;

            for content in resp.choice.into_iter() {
                match content {
                    AssistantContent::Text(text) => {
                        info!("中间响应: {:?}\n", text.text);
                        output = Some(text.text.clone());
                        self.chat_history.push(prompt.clone());
                        let response_message = Message::Assistant {
                            content: OneOrMany::one(AssistantContent::text(&text.text)),
                        };
                        self.chat_history.push(response_message);
                    }
                    AssistantContent::ToolCall(content) => {
                        info!("工具调用: {:?}\n", content);
                        self.chat_history.push(prompt.clone());
                        let tool_call_msg = AssistantContent::ToolCall(content.clone());
                        self.chat_history.push(Message::Assistant {
                            content: OneOrMany::one(tool_call_msg),
                        });

                        let ToolCall {
                            id,
                            function: ToolFunction { name, arguments },
                        } = content;

                        if name == SubmitTool::NAME {
                            let json_output = arguments.to_string();
                            info!("提交匹配结果结果: {:?}\n", json_output);
                            return Ok(json_output);
                        }

                        let tool_result =
                            self.agent.tools.call(&name, arguments.to_string()).await?;

                        info!("工具调用结果: {:?}\n", tool_result);

                        self.chat_history.push(Message::User {
                            content: OneOrMany::one(UserContent::tool_result(
                                id,
                                OneOrMany::one(ToolResultContent::text(tool_result)),
                            )),
                        });

                        prompt = Message::User {
                            content: OneOrMany::one(UserContent::text("resume")),
                        };

                        output = None;
                        break;
                    }
                }
            }

            if let Some(text) = output {
                return Ok(text);
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
pub struct MatchResult {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub season: Option<i32>,
    pub confidence_score: Option<i32>,
}

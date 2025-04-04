use rig::{
    OneOrMany,
    completion::{self, Completion, PromptError},
    extractor::{Extractor, ExtractorBuilder},
    message::{AssistantContent, Message, ToolCall, ToolFunction, ToolResultContent, UserContent},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use tracing::info;

use crate::{
    tool_bgm_tv::BgmTVSearchTool,
    tool_tmdb::{TMDBSearchTool, TMDBSeasonTool},
};

pub static MATCH_BGM_PROMPT: &str = r#"You are an intelligent assistant responsible for matching anime information on Bangumi based on user queries.
Your goal is to identify the single most relevant anime entry.

1.  **Analyze User Query**: Identify potential anime titles (native, romaji, English, etc.) and other relevant keywords provided by the user. Do not include air dates in search keywords.
2.  **Primary Search**: Use the `bgm_tv_search` tool, prioritizing the most promising keyword(s) for the search (usually the native title, if available).
3.  **Evaluate Results**: Examine the search results. If a highly relevant match is found based on the title and other available information (from the search tool's return data), proceed to step 5.
4.  **Refine Search (If Necessary)**: If the initial search results are ambiguous or low quality, you may try searching again using alternative titles (e.g., romaji, English) or extracted keywords. **Only perform additional searches if the first attempt failed to yield a likely match.**
5.  **Select Confident Match**: Evaluate the similarity between the user query and each search result (considering titles, aliases, air dates, etc.). Select the entry with the **highest similarity**, **but only if this similarity meets a high confidence threshold**. If no single entry stands out as a highly confident match, consider it "not found".
6.  **Format Output**: If a confident match is found, return the final result **only** as a JSON object: `{"id": number, "name": string}`. If no confident match is identified, return `{"id": null, "name": null}`. Do not include any explanations, introductions, or other text outside the JSON structure.
"#;

pub static MATCH_TMDB_PROMPT: &str = r#"You are an intelligent assistant responsible for matching anime information on TMDB based on user queries, including identifying the correct season.
Your goal is to identify the single most relevant anime entry and its specific season.

1.  **Analyze User Query**: Identify potential anime titles (native, romaji, English, etc.), potential season numbers (e.g., "Season 2", "S2"), and other relevant keywords. Do not include air dates in search keywords.
2.  **Primary Search**: Use the `tmdb_search_tv_show` tool, prioritizing the most promising title keyword(s) for the search (usually the native title, if available).
3.  **Evaluate Search Results**: Examine the search results. Identify the most likely TV show match based on the title and other available information. If no promising TV show match is found, proceed to step 7.
4.  **Fetch Season Information**: Use the `tmdb_season` tool with the TMDB ID of the most likely TV show match identified in the previous step. This tool will return a list of seasons with their names, numbers, and potentially air dates.
5.  **Match Season**: Compare the season information obtained from the `tmdb_season` tool with the season details mentioned or implied in the user query. Identify the single season that best matches the user's request. Consider season numbers, names, or potentially air dates if provided.
6.  **Select Confident Match**: Based on the TV show match (Step 3) and the specific season match (Step 5), confirm if this combination represents a high-confidence match for the user's query. If the TV show match was weak, or no specific season could be confidently matched, consider it "not found".
7.  **Format Output**: If a confident TV show and season match is found, return the final result **only** as a JSON object: `{"id": tv_show_id, "name": "tv_show_name", "season": matched_season_number}`. If no confident match is identified, return `{"id": null, "name": null, "season": null}`. Do not include any explanations, introductions, or other text outside the JSON structure.
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
        .temperature(0.1)
        .tool(BgmTVSearchTool::new());

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
        .temperature(0.1)
        .tool(TMDBSearchTool::new())
        .tool(TMDBSeasonTool::new());

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
    pub id: Option<i32>,
    pub name: Option<String>,
    pub season: Option<i32>,
}

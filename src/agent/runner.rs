use anyhow::Result;
use rig::providers::{deepseek, gemini, openai, openrouter, xai};
use tracing::error;

use crate::agent::agent::{MatchResult, new_mapping_bgm_tv_agent, new_mapping_tmdb_agent};

pub async fn run_mapping_bgm_tv_agent(
    keywords: &str,
    provider: &str,
    model: &str,
    retry_count: u32,
    retry_delay: u64,
) -> Result<MatchResult> {
    let mut attempts = 0;
    let max_attempts = retry_count;
    let mut result = None;
    let mut last_error = None;

    while attempts < max_attempts {
        let r = match provider {
            "xai" => {
                let client = xai::Client::from_env();
                let mut agent =
                    new_mapping_bgm_tv_agent(client.agent(model), client.extractor(model));
                agent.match_anime(&keywords).await
            }
            "gemini" => {
                let client = gemini::Client::from_env();
                let mut agent =
                    new_mapping_bgm_tv_agent(client.agent(model), client.extractor(model));
                agent.match_anime(&keywords).await
            }
            "deepseek" => {
                let client = deepseek::Client::from_env();
                let mut agent =
                    new_mapping_bgm_tv_agent(client.agent(model), client.extractor(model));
                agent.match_anime(&keywords).await
            }
            "openai" => {
                let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "".to_string());
                let api_base = std::env::var("OPENAI_API_CUSTOM_BASE")
                    .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
                let client = openai::Client::from_url(&api_key, &api_base);
                let mut agent =
                    new_mapping_bgm_tv_agent(client.agent(model), client.extractor(model));
                agent.match_anime(&keywords).await
            }
            "openrouter" => {
                let client = openrouter::Client::from_env();
                let mut agent =
                    new_mapping_bgm_tv_agent(client.agent(model), client.extractor(model));
                agent.match_anime(&keywords).await
            }
            _ => {
                return Err(anyhow::anyhow!("不支持的provider: {}", provider));
            }
        };

        match r {
            Ok(r) => {
                result = Some(r);
                break;
            }
            Err(e) => {
                error!(
                    "mapping_bgm_tv_agent 失败:{}, 等待后重试 ({}/{})",
                    e, attempts, max_attempts
                );
                last_error = Some(e);
            }
        }

        attempts += 1;
        tokio::time::sleep(std::time::Duration::from_secs(retry_delay)).await;
    }

    result.ok_or_else(|| {
        anyhow::anyhow!(
            "mapping_bgm_tv_agent 失败，尝试了 {} 次, error: {:?}",
            max_attempts,
            last_error
        )
    })
}

pub async fn run_mapping_tmdb_agent(
    keywords: &str,
    provider: &str,
    model: &str,
    retry_count: u32,
    retry_delay: u64,
) -> Result<MatchResult> {
    let mut attempts = 0;
    let max_attempts = retry_count;
    let mut result = None;
    let mut last_error = None;

    while attempts < max_attempts {
        let r = match provider {
            "xai" => {
                let client = xai::Client::from_env();
                let mut agent =
                    new_mapping_tmdb_agent(client.agent(model), client.extractor(model));
                agent.match_anime(&keywords).await
            }
            "gemini" => {
                let client = gemini::Client::from_env();
                let mut agent =
                    new_mapping_tmdb_agent(client.agent(model), client.extractor(model));
                agent.match_anime(&keywords).await
            }
            "deepseek" => {
                let client = deepseek::Client::from_env();
                let mut agent =
                    new_mapping_tmdb_agent(client.agent(model), client.extractor(model));
                agent.match_anime(&keywords).await
            }
            "openai" => {
                let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "".to_string());
                let api_base = std::env::var("OPENAI_API_CUSTOM_BASE")
                    .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
                let client = openai::Client::from_url(&api_key, &api_base);
                let mut agent =
                    new_mapping_tmdb_agent(client.agent(model), client.extractor(model));
                agent.match_anime(&keywords).await
            }
            "openrouter" => {
                let client = openrouter::Client::from_env();
                let mut agent =
                    new_mapping_tmdb_agent(client.agent(model), client.extractor(model));
                agent.match_anime(&keywords).await
            }
            _ => {
                return Err(anyhow::anyhow!("不支持的provider: {}", provider));
            }
        };

        match r {
            Ok(r) => {
                result = Some(r);
                break;
            }
            Err(e) => {
                error!(
                    "mapping_tmdb_agent 失败:{}, 等待后重试 ({}/{})",
                    e, attempts, max_attempts
                );
                last_error = Some(e);
            }
        }

        attempts += 1;
        tokio::time::sleep(std::time::Duration::from_secs(retry_delay)).await;
    }

    result.ok_or_else(|| {
        anyhow::anyhow!(
            "mapping_tmdb_agent 失败，尝试了 {} 次, error: {:?}",
            max_attempts,
            last_error
        )
    })
}

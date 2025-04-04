use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File};
use tokio::time;
use tracing::{error, info, warn};

use crate::{
    agent::{new_deepseek, new_gemini, new_openai, new_openrouter, new_xai},
    dump_anilist::DumpedMediaList,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MappingItem {
    pub anilist_id: i32,
    pub bgm_id: Option<i32>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnimeMappings {
    pub mappings: HashMap<i32, MappingItem>,
}

impl AnimeMappings {
    pub fn load_from_file(year: i32) -> Result<Self> {
        match File::open(format!("anilist_mappings_{}.json", year)) {
            Ok(file) => {
                let mappings: Vec<MappingItem> = serde_json::from_reader(file)?;
                Ok(Self {
                    mappings: mappings
                        .into_iter()
                        .map(|item| (item.anilist_id, item))
                        .collect(),
                })
            }
            Err(_) => Ok(Self {
                mappings: HashMap::new(),
            }),
        }
    }

    pub fn save_to_file(&self, year: i32) -> Result<()> {
        let file_name = format!("anilist_mappings_{}.json", year);
        let temp_file = format!("{}.tmp", file_name);
        let file = File::create(&temp_file)?;
        serde_json::to_writer(file, &self.mappings.values().collect::<Vec<_>>())?;
        std::fs::rename(temp_file, file_name)?;
        Ok(())
    }

    pub fn add_mapping(&mut self, anilist_id: i32, bgm_id: Option<i32>) {
        self.mappings
            .insert(anilist_id, MappingItem { anilist_id, bgm_id });
    }

    pub fn get_mapping(&self, anilist_id: i32) -> Option<&MappingItem> {
        self.mappings.get(&anilist_id)
    }
}

pub async fn mapping_anilist_to_bgm(
    start: i32,
    end: i32,
    provider: &str,
    model: &str,
    delay: u64,
) -> Result<()> {
    for year in start..=end {
        info!("处理年份Mappings: {}", year);
        mapping_anilist_to_bgm_by_year(year, provider, model, delay).await?;
    }
    Ok(())
}

async fn mapping_anilist_to_bgm_by_year(year: i32, provider: &str, model: &str, delay: u64) -> Result<()> {
    let media_list = DumpedMediaList::load_from_file(year)?;
    let mut mappings = AnimeMappings::load_from_file(year)?;
    for media in media_list.media_list {
        let exists = mappings.get_mapping(media.id);
        if let Some(exists) = exists {
            if exists.bgm_id.is_some() && exists.bgm_id.unwrap() != 0 {
                continue;
            }
            continue;
        }
        let mut keywords = "match anime: ".to_string();

        if let Some(native) = media.title.native {
            keywords = format!("{} native title: {}", keywords, native);
        }

        if let Some(romaji) = media.title.romaji {
            keywords = format!("{} romaji title: {}", keywords, romaji);
        }

        if let Some(english) = media.title.english {
            keywords = format!("{} english title: {}", keywords, english);
        }

        if let Some(year) = media.start_date.year {
            keywords = format!("{} year: {}", keywords, year);
        }

        if let Some(month) = media.start_date.month {
            keywords = format!("{} month: {}", keywords, month);
        }

        if let Some(day) = media.start_date.day {
            keywords = format!("{} day: {}", keywords, day);
        }
        info!("mapping {} to bgm, keywords: {}", media.id, keywords);

        let mut attempts = 0;
        let max_attempts = 3;
        let mut result = None;

        while attempts < max_attempts {
            let r = match provider {
                "xai" => {
                    let mut agent = new_xai(model);
                    agent.match_anime(&keywords).await
                }
                "gemini" => {
                    let mut agent = new_gemini(model);
                    agent.match_anime(&keywords).await
                }
                "deepseek" => {
                    let mut agent = new_deepseek(model);
                    agent.match_anime(&keywords).await
                }
                "openai" => {
                    let mut agent = new_openai(model);
                    agent.match_anime(&keywords).await
                }
                "openrouter" => {
                    let mut agent = new_openrouter(model);
                    agent.match_anime(&keywords).await
                }
                _ => {
                    return Err(anyhow::anyhow!("不支持的provider: {}", provider));
                }
            };

            match r {
                Ok(res) => {
                    result = Some(res);
                    break;
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        warn!("匹配动漫失败，已重试{}次: {}", attempts, e);
                        break;
                    }
                    error!(
                        "匹配动漫错误: {}，等待5秒后重试 ({}/{})",
                        e, attempts, max_attempts
                    );
                    time::sleep(time::Duration::from_secs(delay)).await;
                }
            }
        }

        match result {
            Some(result) => {
                info!("result: {:?}", result);
                mappings.add_mapping(media.id, result.id);
                mappings.save_to_file(year)?;
            }
            None => {
                error!("匹配动漫: {} 失败", media.id);
                mappings.add_mapping(media.id, None);
            }
        }
    }
    Ok(())
}

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File};
use tracing::{error, info, warn};

use crate::{
    dump_anilist::DumpedMediaList,
    run_agent::{run_mapping_bgm_tv_agent, run_mapping_tmdb_agent},
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MappingItem {
    pub anilist_id: i32,
    pub bgm_id: Option<i32>,
    pub tmdb_id: Option<i32>,
    pub tmdb_season: Option<i32>,
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

    pub fn add_mapping(
        &mut self,
        anilist_id: i32,
        bgm_id: Option<i32>,
        tmdb_id: Option<i32>,
        tmdb_season: Option<i32>,
    ) {
        if let Some(mapping) = self.mappings.get_mut(&anilist_id) {
            if bgm_id.is_some() {
                mapping.bgm_id = bgm_id;
            }
            if tmdb_id.is_some() {
                mapping.tmdb_id = tmdb_id;
            }
            if tmdb_season.is_some() {
                mapping.tmdb_season = tmdb_season;
            }
        } else {
            self.mappings.insert(
                anilist_id,
                MappingItem {
                    anilist_id,
                    bgm_id,
                    tmdb_id,
                    tmdb_season,
                },
            );
        }
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Input {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub romaji_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub english_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub month: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<i32>,
}

async fn mapping_anilist_to_bgm_by_year(
    year: i32,
    provider: &str,
    model: &str,
    delay: u64,
) -> Result<()> {
    let media_list = DumpedMediaList::load_from_file(year)?;
    let mut mappings = AnimeMappings::load_from_file(year)?;
    for media in media_list.media_list {
        let exists = mappings.get_mapping(media.id);
        let mut mapping_bgm = true;
        let mut mapping_tmdb = true;
        if let Some(exists) = exists {
            if exists.bgm_id.is_some() && exists.bgm_id.unwrap() != 0 {
                mapping_bgm = false;
            }
            if exists.tmdb_id.is_some() && exists.tmdb_id.unwrap() != 0 {
                mapping_tmdb = false;
            }
        }
        let input = Input {
            native_title: media.title.native.clone(),
            romaji_title: media.title.romaji.clone(),
            english_title: media.title.english.clone(),
            year: media.start_date.year,
            month: media.start_date.month,
            day: media.start_date.day,
        };

        let keywords = serde_json::to_string(&input)?;

        if mapping_bgm {
            info!("mapping {} to bgm, keywords: {}", media.id, keywords);

            let result = run_mapping_bgm_tv_agent(&keywords, provider, model, 3, delay).await;

            match result {
                Ok(result) => {
                    if result.id.is_none() {
                        warn!(
                            "匹配动漫: {:?} 失败, result: {:?}",
                            media.title.native, result
                        );
                    } else {
                        info!(
                            "匹配动漫: {:?} 成功, result: {:?}",
                            media.title.native, result
                        );
                    }
                    mappings.add_mapping(media.id, result.id, None, None);
                    mappings.save_to_file(year)?;
                }
                Err(e) => {
                    error!("匹配动漫: {} 失败, error: {:?}", media.id, e);
                    mappings.add_mapping(media.id, None, None, None);
                }
            };
        }

        if mapping_tmdb {
            info!("mapping {} to tmdb, keywords: {}", media.id, keywords);
            let result = run_mapping_tmdb_agent(&keywords, provider, model, 3, delay).await;

            match result {
                Ok(result) => {
                    if result.id.is_none() || result.season.is_none() {
                        warn!(
                            "匹配动漫: {:?} 失败, result: {:?}",
                            media.title.native, result
                        );
                    } else {
                        info!(
                            "匹配动漫: {:?} 成功, result: {:?}",
                            media.title.native, result
                        );
                    }

                    mappings.add_mapping(media.id, None, result.id, result.season);
                    mappings.save_to_file(year)?;
                }
                Err(e) => {
                    error!("匹配动漫: {:?} 失败, error: {:?}", media.title.native, e);
                    mappings.add_mapping(media.id, None, None, None);
                }
            }
        }
    }
    Ok(())
}

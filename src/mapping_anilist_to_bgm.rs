use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File};
use tracing::info;

use crate::{agent::AnimeMatcherAgent, dump_anilist::DumpedMediaList};

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
    pub fn load_from_file() -> Result<Self> {
        let file = File::open("anilist_mappings.json")?;
        let mappings: Vec<MappingItem> = serde_json::from_reader(file)?;
        Ok(Self {
            mappings: mappings
                .into_iter()
                .map(|item| (item.anilist_id, item))
                .collect(),
        })
    }

    pub fn save_to_file(&self) -> Result<()> {
        let file = File::create("anilist_mappings.json")?;
        serde_json::to_writer(file, &self.mappings.values().collect::<Vec<_>>())?;
        Ok(())
    }

    pub fn add_mapping(&mut self, anilist_id: i32, bgm_id: Option<i32>) {
        self.mappings
            .insert(anilist_id, MappingItem { anilist_id, bgm_id });
    }

    pub fn contains(&self, anilist_id: i32) -> bool {
        self.mappings.contains_key(&anilist_id)
    }
}

pub async fn mapping_anilist_to_bgm(year: i32) -> Result<i32, anyhow::Error> {
    let mut agent = AnimeMatcherAgent::new();
    let media_list = DumpedMediaList::load_from_file(year)?;
    let mut mappings = AnimeMappings::load_from_file()?;
    for media in media_list.media_list {
        if mappings.contains(media.id) {
            continue;
        }
        let mut keywords = "match anime: ".to_string();

        if let Some(native) = media.title.native {
            keywords = format!("{} nativ: {}", keywords, native);
        }

        if let Some(romaji) = media.title.romaji {
            keywords = format!("{} romiji: {}", keywords, romaji);
        }

        if let Some(english) = media.title.english {
            keywords = format!("{} english: {}", keywords, english);
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
        let result = agent
            .match_anime(&format!("{} {}", keywords, media.id))
            .await?;
        info!("result: {:?}", result);
        mappings.add_mapping(media.id, Some(result.id));
        mappings.save_to_file()?;
    }
    Ok(0)
}

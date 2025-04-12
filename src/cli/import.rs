use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tracing::info;

use crate::models::db::DB;
use crate::models::enums::{Platform, ReviewStatus};
use crate::models::mappings::Model as AnimeMapping;
use crate::models::{anime::Model as Anime, enums::MediaType};
use anyhow::{Context, Result};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AnimeJsonMapping {
    animeplanet_id: Option<String>,
    kitsu_id: Option<u32>,
    mal_id: Option<u32>,
    anilist_id: Option<i32>,
    anisearch_id: Option<u32>,
    anidb_id: Option<u32>,
    notifymoe_id: Option<String>,
    livechart_id: Option<u32>,
    thetvdb_id: Option<u32>,
    imdb_id: Option<String>,
    themoviedb_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AnimeObject {
    anilist_id: i32,
    bgm_id: Option<u32>,
    titles: Vec<String>,
    year: i32,
    season: Option<String>,
    start_date: Option<String>,
    episode_count: Option<i32>,
    #[serde(rename = "seasonNumber")]
    season_number: Option<i32>,
    #[serde(rename = "episodeNumber")]
    episode_number: Option<i32>,
    #[serde(rename = "absoluteEpisodeNumber")]
    absolute_episode_number: Option<i32>,
    #[serde(rename = "type")]
    media_type: Option<String>,
    #[serde(default)]
    mappings: AnimeJsonMapping,
}

pub async fn import_animes(dir: PathBuf) -> Result<()> {
    let files = fs::read_dir(&dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for file_result in files {
        let file_entry = file_result.with_context(|| "Failed to read directory entry")?;
        let file_path = file_entry.path();

        if file_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        info!("Processing file: {}", file_path.display());

        let file_content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        let anime_list: Vec<AnimeObject> = serde_json::from_str(&file_content)
            .with_context(|| format!("Failed to parse JSON from file: {}", file_path.display()))?;

        info!("Found {} anime entries in file", anime_list.len());

        import_anime(anime_list).await?;
    }

    Ok(())
}

fn map_media_type(media_type: &str) -> MediaType {
    match media_type.to_lowercase().as_str() {
        "tv" => MediaType::TV,
        "movie" => MediaType::Movie,
        "ova" => MediaType::OVA,
        "ona" => MediaType::ONA,
        "special" => MediaType::Special,
        _ => MediaType::Unknown,
    }
}

pub async fn import_anime(animes: Vec<AnimeObject>) -> Result<()> {
    let db = DB::new_from_env().await?;

    let mut anime_models = Vec::new();
    let mut mapping_models = Vec::new();
    for anime in animes.iter() {
        let media_type = if let Some(ref media_type) = anime.media_type {
            map_media_type(media_type)
        } else {
            MediaType::Unknown
        };
        let anime_model = Anime {
            anilist_id: anime.anilist_id,
            media_type,
            titles: json!(&anime.titles).to_string(),
            year: anime.year,
            season: anime.season.clone(),
            start_date: anime.start_date.clone(),
            episode_count: anime.episode_count,
            season_number: anime.season_number,
            episode_number: anime.episode_number,
            absolute_episode_number: anime.absolute_episode_number,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        anime_models.push(anime_model);

        let bgm_review_status = if anime.bgm_id.is_some() {
            ReviewStatus::Ready
        } else {
            ReviewStatus::UnMatched
        };

        {
            let mapping_model = AnimeMapping {
                anilist_id: anime.anilist_id,
                platform: Platform::BgmTv,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                review_status: bgm_review_status,
                score: 0,
                platform_id: anime.bgm_id.map(|id| id.to_string()),
            };
            mapping_models.push(mapping_model);
        }

        {
            let tmdb_review_status = if anime.mappings.themoviedb_id.is_some() {
                ReviewStatus::Ready
            } else {
                ReviewStatus::UnMatched
            };
            let mapping_model = AnimeMapping {
                anilist_id: anime.anilist_id,
                platform: Platform::Tmdb,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                review_status: tmdb_review_status,
                score: 0,
                platform_id: anime.mappings.themoviedb_id.clone(),
            };
            mapping_models.push(mapping_model);
        }
    }

    db.batch_add_animes((anime_models, mapping_models)).await?;

    Ok(())
}

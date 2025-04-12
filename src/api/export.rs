use std::fs::File;
use std::path::Path;

use crate::api::types::{CompactAnime, CompactMapping, Resp};
use crate::errors::Result;
use crate::models::export::ExportAnime;
use crate::server::AppState;
use actix_web::web;
use actix_web::{get, web::Json};

const EXPORT_DIR: &str = "export";
#[get("/api/export/animes/{year}")]
pub async fn export_animes(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<Json<Resp<()>>> {
    let year = path.into_inner();
    let animes = state.db.export_animes(year).await?;
    let file_path = format!("{}/{}.json", EXPORT_DIR, year);
    let file = File::create(file_path)?;
    serde_json::to_writer_pretty(file, &animes)?;
    Ok(Json(Resp::ok(None)))
}

#[get("/api/import/animes/{year}")]
pub async fn import_animes(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<Json<Resp<()>>> {
    let year = path.into_inner();
    let file_path = format!("{}/{}.json", EXPORT_DIR, year);
    let file = File::open(file_path)?;
    let animes: Vec<ExportAnime> = serde_json::from_reader(file)?;
    state.db.import_animes(animes).await?;
    Ok(Json(Resp::ok(None)))
}

#[get("/api/compact/animes/dir")]
pub async fn compact_export_dir(_: web::Data<AppState>) -> Result<Json<Resp<()>>> {
    let dir = Path::new(EXPORT_DIR);
    if !dir.exists() {
        return Ok(Json(Resp::ok(None)));
    }

    let mut all_compact_animes: Vec<CompactAnime> = Vec::new();

    let files = dir.read_dir()?;
    for file in files {
        let file = file.unwrap();
        let file_path = file.path();
        let file_name = file_path.file_name().unwrap().to_str().unwrap();

        if !file_name.ends_with(".json") || file_name.starts_with("dist") {
            continue;
        }

        let file = File::open(file_path)?;
        let animes: Vec<ExportAnime> = serde_json::from_reader(file)?;
        let compact_animes: Vec<CompactAnime> = animes
            .into_iter()
            .filter_map(|anime| match serde_json::from_str(&anime.anime.titles) {
                Ok(titles) => Some(CompactAnime {
                    anilist_id: anime.anime.anilist_id,
                    titles,
                    year: anime.anime.year,
                    start_date: anime.anime.start_date,
                    season_number: anime.anime.season_number,
                    mappings: anime
                        .mappings
                        .into_iter()
                        .map(|mapping| CompactMapping {
                            id: mapping.platform_id,
                            platform: mapping.platform,
                        })
                        .collect(),
                }),
                Err(_) => None,
            })
            .collect();

        // 将该年份的数据添加到总集合中
        all_compact_animes.extend(compact_animes);
    }

    // 将所有数据写入一个文件
    let compact_file_path = format!("{}/dist.json", EXPORT_DIR);
    let compact_file = File::create(compact_file_path)?;
    serde_json::to_writer(compact_file, &all_compact_animes)?;

    Ok(Json(Resp::ok(None)))
}

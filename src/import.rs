use crate::{
    db::{
        Db,
        mappings::{Model as Mapping, VerificationStatus},
    },
    dump_anilist::DumpedMediaList,
    mapping_anilist::AnimeMappings,
};
use anyhow::Result;
use chrono::Utc;
use tracing::info;
pub async fn import_anilist_animes(db: &Db, year: i32) -> Result<()> {
    let anilist_animes = DumpedMediaList::load_from_file(year)?;
    let mappings: Vec<Mapping> = anilist_animes
        .media_list
        .iter()
        .map(|anime| Mapping {
            anilist_id: anime.id,
            title: anime.title.native.clone(),
            title_en: anime.title.english.clone(),
            title_cn: anime.title.romaji.clone(),
            title_romaji: anime.title.romaji.clone(),
            alias: None,
            air_year: anime.start_date.year,
            air_month: anime.start_date.month,
            air_day: anime.start_date.day,
            bgm_id: None,
            tmdb_id: None,
            tmdb_season: None,
            bgm_tv_verify_status: VerificationStatus::UnMatched,
            tmdb_verify_status: VerificationStatus::UnMatched,
            match_count: 0,
            update_at: Utc::now().naive_utc(),
        })
        .collect();

    info!(
        "Importing AniList animes for year {}, size: {}",
        year,
        mappings.len()
    );

    db.import_anilist_animes(mappings).await?;
    Ok(())
}

pub async fn import_mappings(db: &Db, year: i32) -> Result<()> {
    let mappings = AnimeMappings::load_from_file(year)?;
    if mappings.mappings.is_empty() {
        info!("No mappings found for year {}, skipping", year);
        return Ok(());
    }
    let mappings = mappings
        .mappings
        .iter()
        .map(|(_, mapping)| {
            let bgm_tv_verify_status = if mapping.bgm_id.is_some() {
                VerificationStatus::Ready
            } else {
                VerificationStatus::UnMatched
            };
            let tmdb_verify_status = if mapping.tmdb_id.is_some() && mapping.tmdb_season.is_some() {
                VerificationStatus::Ready
            } else {
                VerificationStatus::UnMatched
            };
            Mapping {
                anilist_id: mapping.anilist_id,
                title: None,
                title_en: None,
                title_cn: None,
                title_romaji: None,
                alias: None,
                air_year: None,
                air_month: None,
                air_day: None,
                bgm_id: mapping.bgm_id,
                tmdb_id: mapping.tmdb_id,
                tmdb_season: mapping.tmdb_season,
                bgm_tv_verify_status,
                tmdb_verify_status,
                match_count: 0,
                update_at: Utc::now().naive_utc(),
            }
        })
        .collect();
    db.import_mappings(mappings).await?;
    Ok(())
}

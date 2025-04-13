use crate::api::types::{ManualMappingRequest, Summary, YearStatistics};
use crate::errors::Result;
use crate::{
    api::types::{Anime, Mapping, Pagination, QueryAnimes, Resp},
    server::AppState,
};
use actix_web::get;
use actix_web::{
    post,
    web::{self, Json},
};

#[post("/api/animes/page")]
pub async fn query_animes(
    state: web::Data<AppState>,
    query: web::Json<QueryAnimes>,
) -> Result<Json<Resp<Pagination<Anime>>>> {
    let query = query.into_inner();
    let query_result = state.db.query_animes(&query).await?;
    let mut animes = Vec::new();
    for (ani, mapping) in query_result.data {
        let mut anime = Anime {
            anilist_id: ani.anilist_id,
            titles: serde_json::from_str(&ani.titles)?,
            year: ani.year,
            mappings: Vec::new(),
        };
        for mapping in mapping {
            anime.mappings.push(Mapping {
                id: mapping.platform_id,
                review_status: mapping.review_status,
                platform: mapping.platform,
                score: mapping.score,
            });
        }
        animes.push(anime);
    }

    let page_query = query.query;
    Ok(Json(Resp::ok(Some(Pagination {
        page: page_query.page,
        page_size: page_query.page_size,
        total: query_result.total,
        data: animes,
    }))))
}

#[post("/api/anime/mapping/manual")]
pub async fn manual_mapping(
    state: web::Data<AppState>,
    request: web::Json<ManualMappingRequest>,
) -> Result<Json<Resp<()>>> {
    if let Some(season_number) = request.season_number {
        state
            .db
            .update_season_number(request.anilist_id, season_number)
            .await?;
    }
    state
        .db
        .update_anime_mapping(
            request.anilist_id,
            request.platform.clone(),
            request.platform_id.clone(),
            100,
        )
        .await?;
    Ok(Json(Resp::ok(None)))
}

#[get("/api/animes/summary")]
pub async fn summary(state: web::Data<AppState>) -> Result<Json<Resp<Summary>>> {
    let summary = state.db.summary().await?;
    Ok(Json(Resp::ok(Some(summary))))
}

#[get("/api/animes/year-statistics")]
pub async fn year_statistics(state: web::Data<AppState>) -> Result<Json<Resp<YearStatistics>>> {
    let statistics = state.db.get_year_statistics().await?;
    Ok(Json(Resp::ok(Some(statistics))))
}

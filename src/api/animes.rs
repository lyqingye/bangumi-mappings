use crate::errors::Result;
use crate::{
    api::types::{Anime, Mapping, Pagination, QueryAnimes, Resp},
    server::AppState,
};
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

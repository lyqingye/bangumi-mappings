use crate::api::types::Resp;
use crate::errors::Result;
use crate::{
    models::enums::{Platform, ReviewStatus},
    server::AppState,
};
use actix_web::{
    get,
    web::{self, Json},
};

#[get("/api/anime/{anilist_id}/review/{platform}/{status}")]
pub async fn review_anime(
    state: web::Data<AppState>,
    path: web::Path<(i32, Platform, ReviewStatus)>,
) -> Result<Json<Resp<()>>> {
    let (anilist_id, platform, status) = path.into_inner();
    state.db.review(anilist_id, platform, status).await?;
    Ok(Json(Resp::ok(Some(()))))
}

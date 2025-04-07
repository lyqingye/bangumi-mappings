use std::sync::Arc;

use actix_web::{
    get, post,
    web::{self, Json},
};

use crate::{db::mappings::VerificationStatus, server::AppState};
use crate::{db::query::QueryMappings, server::Resp};
use crate::{
    db::{mappings::Model as Mapping, query::QueryResult, update::UpdateVerificationRequest},
    server::ServerError,
};

#[post("/mappings")]
pub async fn query_mappings(
    state: web::Data<Arc<AppState>>,
    query: web::Json<QueryMappings>,
) -> Result<Json<Resp<QueryResult<Mapping>>>, ServerError> {
    let mappings = state.db.query_mappings(&query).await?;

    Ok(Json(Resp::ok(mappings)))
}

#[get("/mappings/{anilist_id}")]
pub async fn get_mapping(
    state: web::Data<Arc<AppState>>,
    path: web::Path<i32>,
) -> Result<Json<Resp<Mapping>>, ServerError> {
    let anilist_id = path.into_inner();
    let mapping = state.db.get_by_anilist_id(anilist_id).await?;
    Ok(Json(Resp::ok(mapping.unwrap())))
}

#[post("/mappings/update-status")]
pub async fn update_verification_status(
    state: web::Data<Arc<AppState>>,
    request: web::Json<UpdateVerificationRequest>,
) -> Result<Json<Resp<()>>, ServerError> {
    state
        .db
        .update_verification_status(request.into_inner())
        .await?;
    Ok(Json(Resp::ok_empty()))
}
#[get("/api/mapping/{anilist_id}/tmdb/verify/{status}")]
pub async fn update_tmdb_verify_status(
    state: web::Data<Arc<AppState>>,
    path: web::Path<(i32, VerificationStatus)>,
) -> Result<Json<Resp<()>>, ServerError> {
    let (anilist_id, status) = path.into_inner();
    state
        .db
        .update_tmdb_verify_status(anilist_id, status)
        .await?;
    Ok(Json(Resp::ok_empty()))
}

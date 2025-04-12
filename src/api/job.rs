use crate::job::mapping_bgm::JobDetails;
use crate::{api::types::Resp, models::enums::Platform};
use crate::errors::Result;
use crate::server::AppState;
use actix_web::{
    get,
    web::{self, Json},
};

#[get("/api/job/{platform}/create/{year}/{provider}/{model}")]
pub async fn create_job(
    state: web::Data<AppState>,
    path: web::Path<(Platform, i32, String, String)>,
) -> Result<Json<Resp<()>>> {
    let (platform, year, provider, model) = path.into_inner();
    {
        let mut job_runner = state.job_runner.lock().unwrap();
        job_runner.create_job(platform, year, provider, model).await?;
    }
    Ok(Json(Resp::ok(Some(()))))
}

#[get("/api/job/{platform}/run/{year}")]
pub async fn run_job(
    state: web::Data<AppState>,
    path: web::Path<(Platform, i32)>,
) -> Result<Json<Resp<()>>> {
    let (platform, year) = path.into_inner();
    {
        let job_runner = state.job_runner.lock().unwrap();
        job_runner.run(platform, year).await;
    }
    Ok(Json(Resp::ok(Some(()))))
}

#[get("/api/job/list")]
pub async fn list_jobs(state: web::Data<AppState>) -> Result<Json<Resp<Vec<JobDetails>>>> {
    let job_runner = state.job_runner.lock().unwrap();
    let jobs = job_runner.list_jobs().await?;
    Ok(Json(Resp::ok(Some(jobs))))
}

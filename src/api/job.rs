use crate::errors::Result;
use crate::job::mapping_bgm::JobDetails;
use crate::server::AppState;
use crate::{api::types::Resp, models::enums::Platform};
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
        job_runner
            .create_job(platform, year, provider, model)
            .await?;
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

#[get("/api/job/{platform}/pause/{year}")]
pub async fn pause_job(
    state: web::Data<AppState>,
    path: web::Path<(Platform, i32)>,
) -> Result<Json<Resp<()>>> {
    let (platform, year) = path.into_inner();
    {
        let job_runner = state.job_runner.lock().unwrap();
        job_runner.pause_job(platform, year).await?;
    }
    Ok(Json(Resp::ok(Some(()))))
}

#[get("/api/job/{platform}/resume/{year}")]
pub async fn resume_job(
    state: web::Data<AppState>,
    path: web::Path<(Platform, i32)>,
) -> Result<Json<Resp<()>>> {
    let (platform, year) = path.into_inner();
    {
        let job_runner = state.job_runner.lock().unwrap();
        job_runner.resume_job(platform, year).await?;
    }
    Ok(Json(Resp::ok(Some(()))))
}

#[get("/api/job/{platform}/remove/{year}")]
pub async fn remove_job(
    state: web::Data<AppState>,
    path: web::Path<(Platform, i32)>,
) -> Result<Json<Resp<()>>> {
    let (platform, year) = path.into_inner();
    {
        let mut job_runner = state.job_runner.lock().unwrap();
        job_runner.remove_job(platform, year).await?;
    }
    Ok(Json(Resp::ok(Some(()))))
}

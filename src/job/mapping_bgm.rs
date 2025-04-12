use std::sync::{Arc, RwLock};
use std::usize;

use crate::agent::runner::{run_mapping_bgm_tv_agent, run_mapping_tmdb_agent};
use crate::models::anime::Model as Anime;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::error;

use crate::{
    api::types::{PageQuery, QueryAnimes},
    models::{
        db::DB,
        enums::{Platform, ReviewStatus},
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDetails {
    pub year: i32,
    pub num_animes_to_match: usize,
    pub num_processed: usize,
    pub num_matched: usize,
    pub num_failed: usize,
    pub job_start_time: DateTime<Utc>,
    pub provider: String,
    pub model: String,
    pub platform: Platform,

    #[serde(skip)]
    pub animes: Vec<Anime>,
}

#[derive(Clone)]
pub struct MappingBgmJobRunner {
    db: DB,
    jobs: Vec<Arc<RwLock<JobDetails>>>,
}

impl MappingBgmJobRunner {
    pub async fn new() -> Result<Self> {
        let db = DB::new_from_env().await?;
        let jobs = Vec::new();
        Ok(Self { db, jobs })
    }

    pub async fn create_job(
        &mut self,
        platform: Platform,
        year: i32,
        provider: String,
        model: String,
    ) -> Result<()> {
        let query_result = self
            .db
            .query_animes(&QueryAnimes {
                year: Some(year),
                query: PageQuery {
                    page: 1,
                    page_size: usize::MAX,
                },
                status: Some(ReviewStatus::UnMatched),
            })
            .await?;

        let animes = query_result
            .data
            .iter()
            .filter(|(_anime, mappings)| {
                mappings.iter().any(|m| {
                    m.platform == platform && m.review_status == ReviewStatus::UnMatched
                })
            })
            .collect::<Vec<_>>();

        let job_details = Arc::new(RwLock::new(JobDetails {
            platform,
            year,
            num_animes_to_match: animes.len(),
            num_processed: 0,
            num_matched: 0,
            num_failed: 0,
            job_start_time: Utc::now(),
            animes: animes.iter().map(|(anime, _)| anime.clone()).collect(),
            provider,
            model,
        }));

        self.jobs.push(job_details.clone());

        Ok(())
    }

    pub async fn run(&self, platform: Platform, year: i32) {
        let job_details = self
            .jobs
            .iter()
            .find(|job| job.read().unwrap().platform == platform && job.read().unwrap().year == year);
        if let Some(job_details) = job_details {
            let cloned = self.clone();
            let job_details = job_details.clone();
            tokio::spawn(async move {
                cloned.run_job(job_details).await;
            });
        }
    }

    async fn run_job(&self, job_details: Arc<RwLock<JobDetails>>) {
        // 提前克隆数据并释放锁，避免锁跨越线程边界
        let details = {
            let guard = job_details.read().unwrap();
            guard.clone()
        };

        for anime in details.animes {
            let keywords = json!({
                "titles": anime.titles,
                "year": anime.year,
                "media_type": anime.media_type,
                "start_date": anime.start_date,
                "episode_number": anime.episode_number,
            });

            let result = match details.platform {
                Platform::BgmTv => run_mapping_bgm_tv_agent(
                    keywords.to_string().as_str(),
                    &details.provider,
                    &details.model,
                    3,
                    10,
                )
                .await,
                Platform::Tmdb => run_mapping_tmdb_agent(
                    keywords.to_string().as_str(),
                    &details.provider,
                    &details.model,
                    3,
                    10,
                )
                .await,
            };

            let result = match result {
                Ok(result) => result,
                Err(e) => {
                    error!("匹配Bgm失败: {}", e);
                    continue;
                }
            };

            let mut success_count = 0;
            let mut failed_count = 0;
            if let Some(id) = result.id {
                success_count += 1;
                self.db
                    .update_anime_mapping(
                        anime.anilist_id,
                        details.platform.clone(),
                        id.to_string(),
                        result.confidence_score.unwrap_or_default() as u8,
                    )
                    .await
                    .unwrap();
            } else {
                failed_count += 1;
            }

            {
                let mut guard = job_details.write().unwrap();
                guard.num_matched += success_count;
                guard.num_failed += failed_count;
                guard.num_processed += 1;
            }
        }
    }

    pub async fn list_jobs(&self) -> Result<Vec<JobDetails>> {
        Ok(self
            .jobs
            .iter()
            .map(|job| job.read().unwrap().clone())
            .collect())
    }
}

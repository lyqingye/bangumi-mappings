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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Created,
    Running,
    Paused,
    Completed,
    Failed,
}

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
    pub status: JobStatus,
    pub current_index: usize,

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
                mappings
                    .iter()
                    .any(|m| m.platform == platform && m.review_status == ReviewStatus::UnMatched)
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
            status: JobStatus::Created,
            current_index: 0,
        }));

        self.jobs.push(job_details.clone());

        Ok(())
    }

    pub async fn run(&self, platform: Platform, year: i32) {
        let job_details = self.jobs.iter().find(|job| {
            job.read().unwrap().platform == platform && job.read().unwrap().year == year
        });
        if let Some(job_details) = job_details {
            {
                let mut guard = job_details.write().unwrap();
                guard.status = JobStatus::Running;
            }

            let cloned = self.clone();
            let job_details = job_details.clone();
            tokio::spawn(async move {
                cloned.run_job(job_details).await;
            });
        }
    }

    pub async fn pause_job(&self, platform: Platform, year: i32) -> Result<bool> {
        let job_details = self.jobs.iter().find(|job| {
            job.read().unwrap().platform == platform && job.read().unwrap().year == year
        });

        if let Some(job_details) = job_details {
            let mut guard = job_details.write().unwrap();
            if guard.status == JobStatus::Running {
                guard.status = JobStatus::Paused;
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn resume_job(&self, platform: Platform, year: i32) -> Result<bool> {
        let job_details = self.jobs.iter().find(|job| {
            job.read().unwrap().platform == platform && job.read().unwrap().year == year
        });

        if let Some(job_details) = job_details {
            let mut guard = job_details.write().unwrap();
            if guard.status == JobStatus::Paused {
                guard.status = JobStatus::Running;
                drop(guard); // 释放锁，避免死锁

                let cloned = self.clone();
                let job_details_clone = job_details.clone();
                tokio::spawn(async move {
                    cloned.run_job(job_details_clone).await;
                });

                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn remove_job(&mut self, platform: Platform, year: i32) -> Result<bool> {
        // 查找任务索引
        let index = self.jobs.iter().position(|job| {
            let guard = job.read().unwrap();
            guard.platform == platform && guard.year == year
        });

        // 如果找到任务，直接从数组中移除
        if let Some(index) = index {
            self.jobs.remove(index);
            return Ok(true);
        }

        Ok(false)
    }

    async fn run_job(&self, job_details: Arc<RwLock<JobDetails>>) {
        // 提前获取需要处理的动画列表和起始索引
        let (start_index, animes) = {
            let guard = job_details.read().unwrap();
            (guard.current_index, guard.animes.clone())
        };

        // 确保只处理尚未处理的动画
        for i in start_index..animes.len() {
            // 每次循环开始时检查任务状态
            {
                let guard = job_details.read().unwrap();
                if guard.status == JobStatus::Paused {
                    // 如果任务已暂停，直接返回，不做进一步处理
                    return;
                }
            }

            let anime = &animes[i];
            let keywords = json!({
                "titles": anime.titles,
                "year": anime.year,
                "media_type": anime.media_type,
                "start_date": anime.start_date,
                "episode_number": anime.episode_number,
            });

            let platform = {
                let guard = job_details.read().unwrap();
                guard.platform.clone()
            };

            let result = match platform {
                Platform::BgmTv => {
                    let provider = job_details.read().unwrap().provider.clone();
                    let model = job_details.read().unwrap().model.clone();
                    run_mapping_bgm_tv_agent(
                        keywords.to_string().as_str(),
                        &provider,
                        &model,
                        3,
                        10,
                    )
                    .await
                }
                Platform::Tmdb => {
                    let provider = job_details.read().unwrap().provider.clone();
                    let model = job_details.read().unwrap().model.clone();
                    run_mapping_tmdb_agent(keywords.to_string().as_str(), &provider, &model, 3, 10)
                        .await
                }
            };

            let result = match result {
                Ok(result) => result,
                Err(e) => {
                    error!("匹配Bgm失败: {}", e);
                    {
                        let mut guard = job_details.write().unwrap();
                        guard.num_failed += 1;
                        guard.num_processed += 1;
                        guard.current_index = i + 1; // 更新索引以便下次从这里继续
                    }
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
                        platform.clone(),
                        id.to_string(),
                        result.confidence_score.unwrap_or_default() as u8,
                    )
                    .await
                    .unwrap();

                if let Some(season) = result.season {
                    if season > 0 {
                        self.db
                            .update_season_number(anime.anilist_id, season)
                            .await
                            .unwrap();
                    }
                }
            } else {
                failed_count += 1;
            }

            {
                let mut guard = job_details.write().unwrap();
                guard.num_matched += success_count;
                guard.num_failed += failed_count;
                guard.num_processed += 1;
                guard.current_index = i + 1; // 重要：更新当前索引为下一条记录

                // 检查是否所有动画都已处理完毕
                if guard.current_index >= animes.len() {
                    guard.status = JobStatus::Completed;
                }
            }

            // 再次检查是否应该继续执行
            {
                let guard = job_details.read().unwrap();
                if guard.status != JobStatus::Running {
                    return;
                }
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

use actix_cors::Cors;
use actix_web::web;
use actix_web::{App, HttpServer, middleware::Logger};
use anyhow::Result;
use std::sync::Mutex;
use std::{env, sync::Arc};
use tracing::info;

use crate::anilist::AniListClient;
use crate::api::animes::{manual_mapping, query_animes, summary, year_statistics};
use crate::api::export::{compact_export_dir, export_animes, import_animes};
use crate::api::job::{create_job, list_jobs, pause_job, remove_job, resume_job, run_job};
use crate::api::review::review_anime;
use crate::job::mapping_bgm::MappingBgmJobRunner;
use crate::models::db::DB;

#[derive(Clone)]
pub struct AppState {
    pub anilist: Arc<AniListClient>,
    pub job_runner: Arc<Mutex<MappingBgmJobRunner>>,
    pub db: DB,
}

/// 服务器结构体
pub struct Server {
    host: String,
    port: u16,
}

impl Server {
    /// 创建新的服务器实例
    pub async fn new() -> Result<Self> {
        // 从环境变量获取配置
        let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or(8080);

        // 创建数据库连接
        Ok(Self { host, port })
    }

    /// 启动服务器
    pub async fn serve(self) -> Result<()> {
        info!("启动服务器: {}:{}", self.host, self.port);
        let db = DB::new_from_env().await?;
        let anilist = Arc::new(AniListClient::new());
        let job_runner = Arc::new(Mutex::new(MappingBgmJobRunner::new().await?));
        let state = AppState {
            anilist,
            db,
            job_runner,
        };

        // 创建HTTP服务器
        HttpServer::new(move || {
            // 配置CORS
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600);

            App::new()
                // 添加中间件
                .app_data(web::Data::new(state.clone()))
                .service(query_animes)
                .service(review_anime)
                .service(create_job)
                .service(run_job)
                .service(list_jobs)
                .service(pause_job)
                .service(resume_job)
                .service(remove_job)
                .service(export_animes)
                .service(import_animes)
                .service(compact_export_dir)
                .service(summary)
                .service(year_statistics)
                .service(manual_mapping)
                .wrap(Logger::default())
                .wrap(cors)
        })
        .bind((self.host.clone(), self.port))?
        .run()
        .await?;

        Ok(())
    }
}

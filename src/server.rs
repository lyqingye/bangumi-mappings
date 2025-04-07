use actix_cors::Cors;
use actix_web::{App, HttpServer};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tracing::{error, info};

use crate::{
    db::{self, Db},
    router,
};

use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("actix error: {0}")]
    ActixError(#[from] actix_web::Error),

    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("db error: {0}")]
    DbError(#[from] sea_orm::DbErr),

    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Resp<T> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

impl<T> Resp<T> {
    pub fn ok(data: T) -> Self {
        Self {
            code: 0,
            msg: "success".to_string(),
            data: Some(data),
        }
    }

    pub fn ok_empty() -> Self {
        Self {
            code: 0,
            msg: "success".to_string(),
            data: None,
        }
    }

    pub fn err_msg(msg: String) -> Self {
        Self {
            code: 1,
            msg,
            data: None,
        }
    }
}

impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        let resp: Resp<()> = Resp::err_msg(self.to_string());
        HttpResponse::Ok().json(resp)
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
}

pub struct Server {
    state: Arc<AppState>,
}

impl Server {
    pub async fn new() -> Result<Self> {
        let db = db::Db::new_from_env().await?;
        Ok(Self {
            state: Arc::new(AppState { db }),
        })
    }

    pub async fn serve(&self) -> Result<()> {
        let addr = "0.0.0.0:8080".parse::<SocketAddr>()?;
        let state = self.state.clone();

        // 创建 HTTP 服务器
        let server = HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600);
            App::new()
                .wrap(cors)
                .configure(|cfg| router::configure_app(cfg, state.clone()))
        })
        .bind(addr)?
        .run();

        let server_handle = server.handle();
        let server_task = tokio::spawn(server);

        info!("服务监听: http://{}", addr);

        // 等待中断信号
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                server_handle.stop(true).await;

                match server_task.await {
                    Ok(_) => info!("HTTP 服务器已完全停止"),
                    Err(e) => error!("HTTP 服务器停止时发生错误: {}", e),
                }
                info!("服务器优雅停机完成");
            }
            Err(err) => error!("无法监听中断信号: {}", err),
        }
        Ok(())
    }
}

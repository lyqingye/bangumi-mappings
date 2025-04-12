use crate::api::types::Resp;
use actix_web::{HttpResponse, ResponseError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("actix error: {0}")]
    ActixError(#[from] actix_web::Error),

    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("file error: {0}")]
    FileError(#[from] std::io::Error),
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let resp: Resp<()> = Resp::err(Some(self.to_string()));
        HttpResponse::Ok().json(resp)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

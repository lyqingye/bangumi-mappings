use anyhow::Result;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::{sync::Arc, time::Duration};

pub mod mappings;
pub mod query;
pub mod update;

#[derive(Clone)]
pub struct Db {
    conn: Arc<DatabaseConnection>,
}

impl Db {
    /// 创建新的数据库连接
    pub async fn new(database_url: &str) -> Result<Self> {
        let mut opt = ConnectOptions::new(database_url);
        opt.max_connections(5)
            .min_connections(1)
            .connect_timeout(Duration::from_secs(5))
            .acquire_timeout(Duration::from_secs(5))
            // 设置 SQL 语句日志级别
            .sqlx_logging(true)
            .sqlx_logging_level(tracing::log::LevelFilter::Debug);

        let conn = Database::connect(opt).await?;
        Ok(Self {
            conn: Arc::new(conn),
        })
    }

    pub async fn new_from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        Self::new(&database_url).await
    }

    pub(crate) fn conn(&self) -> &DatabaseConnection {
        self.conn.as_ref()
    }
}

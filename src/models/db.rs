use anyhow::Result;
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;

use sea_orm::{Database, DatabaseConnection};

use crate::migration::Migrator;

#[derive(Clone)]
pub struct DB {
    pub(crate) db: Arc<DatabaseConnection>,
}

impl DB {
    pub async fn new_from_env() -> Result<Self> {
        let db_path = std::env::var("DATABASE_URL")?;
        let db = Database::connect(&db_path).await?;
        Migrator::up(&db, None).await?;
        Ok(Self { db: Arc::new(db) })
    }

    #[cfg(test)]
    pub async fn new_for_test() -> Result<Self> {
        let db = Database::connect("sqlite::memory:").await?;
        Migrator::up(&db, None).await?;
        Ok(Self { db: Arc::new(db) })
    }
}

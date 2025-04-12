use serde::{Deserialize, Serialize};

pub mod mapping_bgm;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    MappingBgm,
    MappingTmdb,
}

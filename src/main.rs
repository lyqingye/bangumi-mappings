use std::path::PathBuf;

use agent::runner::{run_mapping_bgm_tv_agent, run_mapping_tmdb_agent};
use anyhow::Result;
use clap::{Parser, Subcommand};
use cli::import::import_animes;
use dotenv::dotenv;

pub mod agent;
pub mod anilist;
pub mod api;
pub mod cli;
pub mod errors;
pub mod job;
pub mod migration;
pub mod models;
pub mod server;

#[derive(Debug, Parser)]
#[clap(version = "1.0", author = "Your Name")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// 匹配动漫信息
    #[command(name = "match-bgm")]
    MatchBgm {
        /// 搜索关键词
        #[arg(short, long)]
        query: String,
        #[arg(short, long, default_value = "bgm_tv")]
        provider: String,
        #[arg(short, long, default_value = "gpt-4o")]
        model: String,
    },
    /// 匹配动漫信息
    #[command(name = "match-tmdb")]
    MatchTmdb {
        /// 搜索关键词
        #[arg(short, long)]
        query: String,
        #[arg(short, long, default_value = "tmdb")]
        provider: String,
        #[arg(short, long, default_value = "gpt-4o")]
        model: String,
    },
    /// 启动服务器
    #[command(name = "server")]
    Server,
    /// 导入动漫信息
    #[command(name = "import")]
    Import {
        /// 导入动漫信息
        #[arg(short, long)]
        path: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // 加载环境变量
    dotenv().ok();

    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::MatchBgm {
            query,
            provider,
            model,
        } => {
            let result = run_mapping_bgm_tv_agent(&query, &provider, &model, 1, 5).await?;
            println!("{}", serde_json::to_string(&result).unwrap());
        }
        Commands::MatchTmdb {
            query,
            provider,
            model,
        } => {
            let result = run_mapping_tmdb_agent(&query, &provider, &model, 1, 5).await?;
            println!("{}", serde_json::to_string(&result).unwrap());
        }
        Commands::Server => {
            let server = server::Server::new().await?;
            server.serve().await?;
        }
        Commands::Import { path } => {
            let result = import_animes(PathBuf::from(path)).await?;
            println!("{}", serde_json::to_string(&result).unwrap());
        }
    }

    Ok(())
}

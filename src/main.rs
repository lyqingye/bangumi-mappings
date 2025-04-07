mod agent;
mod api;
mod db;
mod dump_anilist;
mod import;
mod mapping_anilist;
mod router;
mod run_agent;
mod server;
mod tool_bgm_tv;
mod tool_submit;
mod tool_tmdb;

use clap::{Parser, Subcommand};
use run_agent::{run_mapping_bgm_tv_agent, run_mapping_tmdb_agent};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 导出AniList数据
    #[command(name = "dump")]
    DumpAnilist {
        #[arg(short, long)]
        start: i32,
        #[arg(short, long)]
        end: i32,
    },
    /// 匹配动漫信息
    #[command(name = "match-bgm")]
    MatchBgm {
        /// 搜索关键词
        #[arg(short, long)]
        query: String,
        #[arg(short, long)]
        provider: String,
        #[arg(short, long)]
        model: String,
    },
    /// 匹配动漫信息
    #[command(name = "match-tmdb")]
    MatchTmdb {
        /// 搜索关键词
        #[arg(short, long)]
        query: String,
        #[arg(short, long)]
        provider: String,
        #[arg(short, long)]
        model: String,
    },
    /// 匹配动漫信息
    #[command(name = "mapping")]
    Mapping {
        /// 搜索关键词
        #[arg(short, long)]
        start: i32,
        #[arg(short, long)]
        end: i32,
        #[arg(short, long)]
        provider: String,
        #[arg(short, long)]
        model: String,
        #[arg(short, long)]
        delay: u64,
    },
    /// 启动服务器
    #[command(name = "server")]
    Server,
    /// 导入AniList数据
    #[command(name = "import-anilist-animes")]
    ImportAnilistAnimes {
        #[arg(short, long)]
        start: i32,
        #[arg(short, long)]
        end: i32,
    },
    /// 导入AniList数据
    #[command(name = "import-mappings")]
    ImportMappings {
        #[arg(short, long)]
        start: i32,
        #[arg(short, long)]
        end: i32,
    },
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();
    let cli = Cli::parse();

    match cli.command {
        Commands::DumpAnilist { start, end } => {
            dump_anilist::run_dump_anilist(start, end).await?;
        }
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
        Commands::Mapping {
            start,
            end,
            provider,
            model,
            delay,
        } => {
            mapping_anilist::mapping_anilist_to_bgm(start, end, &provider, &model, delay).await?;
        }
        Commands::Server => {
            let server = server::Server::new().await?;
            server.serve().await?;
        }
        Commands::ImportAnilistAnimes { start, end } => {
            let db = db::Db::new_from_env().await?;
            for year in start..=end {
                import::import_anilist_animes(&db, year).await?;
            }
        }
        Commands::ImportMappings { start, end } => {
            let db = db::Db::new_from_env().await?;
            for year in start..=end {
                import::import_mappings(&db, year).await?;
            }
        }
    }
    Ok(())
}

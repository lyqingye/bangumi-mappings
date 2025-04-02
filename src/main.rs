use agent::AnimeMatcherAgent;

mod agent;
mod dump_anilist;

use clap::{Parser, Subcommand};

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
    #[command(name = "match")]
    Match {
        /// 搜索关键词
        #[arg(short, long)]
        query: String,
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
        Commands::Match { query } => {
            let mut agent = AnimeMatcherAgent::new();
            let result = agent.match_anime(&query).await?;
            println!("{}", serde_json::to_string(&result).unwrap());
        }
    }
    Ok(())
}

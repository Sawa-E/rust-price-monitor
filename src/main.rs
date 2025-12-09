mod cli;
mod commands;
mod db;
mod scraper;
mod web;
mod scheduler;

use clap::Parser;
use cli::{Cli, Commands};
use tracing_subscriber;

fn main() -> anyhow::Result<()> {
    // 🆕 ログ設定を初期化
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .init();

    let cli = Cli::parse();
    let rt = tokio::runtime::Runtime::new()?;

    match cli.command {
        Some(Commands::Add { url }) => {
            let conn = db::init_db()?;
            rt.block_on(async {
                commands::cmd_add(&conn, &url).await
            })?;
        }
        Some(Commands::List) => {
            let conn = db::init_db()?;
            commands::cmd_list(&conn)?;
        }
        Some(Commands::Check) => {
            let conn = db::init_db()?;
            rt.block_on(async {
                commands::cmd_check(&conn).await
            })?;
        }
        Some(Commands::Export { filename }) => {
            let conn = db::init_db()?;
            commands::cmd_export(&conn, &filename)?;
        }
        None => {
            println!("🌐 Starting Web UI with auto price check...");
            rt.block_on(async {
                web::run_server().await
            })?;
        }
    }

    Ok(())
}

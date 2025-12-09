mod cli;
mod commands;
mod db;
mod scraper;
mod web;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // 🔧 Tokioランタイムを作成
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
            println!("🌐 Starting Web UI...");
            rt.block_on(async {
                web::run_server().await
            })?;
        }
    }

    Ok(())
}

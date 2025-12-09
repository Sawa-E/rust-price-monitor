mod cli;
mod db;
mod scraper;
mod commands;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use db::init_db;
use commands::{cmd_add, cmd_check, cmd_list, cmd_export};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let conn = init_db()?;
    
    match cli.command {
        Commands::Add { url } => {
            cmd_add(&conn, &url)?;
        }
        Commands::Check => {
            cmd_check(&conn)?;
        }
        Commands::List => {
            cmd_list(&conn)?;
        }
        Commands::Export { filename } => {  // ← 追加
            cmd_export(&conn, &filename)?;
        }
    }
    
    Ok(())
}

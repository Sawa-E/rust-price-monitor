use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rust-price-monitor")]
#[command(about = "Amazon商品の価格追跡ツール", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 商品を追加
    Add {
        /// Amazon商品のURL
        url: String,
    },
    /// すべての商品の価格をチェック
    Check,
    /// 登録済み商品の一覧を表示
    List,
}

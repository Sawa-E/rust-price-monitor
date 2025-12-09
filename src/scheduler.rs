use anyhow::Result;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{info, error};

use crate::db::{save_price_history, save_product};
use crate::scraper::fetch_amazon_price;

pub type SharedDb = Arc<Mutex<Connection>>;

/// 定期実行スケジューラーを起動
pub async fn start_scheduler(db: SharedDb) -> Result<()> {
    info!("🕐 スケジューラーを起動します");

    let scheduler = JobScheduler::new().await?;

    // 毎時0分に実行（例: 10:00, 11:00, 12:00...）
    // Cron形式: "秒 分 時 日 月 曜日"
    // "0 0 * * * *" = 毎時0分0秒
    let job = Job::new_async("0 0 * * * *", move |_uuid, _lock| {
        let db = db.clone();
        Box::pin(async move {
            info!("⏰ 定期価格チェックを開始します");
            if let Err(e) = check_all_prices(db).await {
                error!("❌ 定期価格チェックでエラー: {}", e);
            } else {
                info!("✅ 定期価格チェック完了");
            }
        })
    })?;

    scheduler.add(job).await?;
    scheduler.start().await?;

    info!("✅ スケジューラーが起動しました（毎時0分に実行）");

    Ok(())
}

/// 全商品の価格をチェック
async fn check_all_prices(db: SharedDb) -> Result<()> {
    let products: Vec<(i64, String, String)> = {
        let conn = db.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, url, name FROM products")?;
        
        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .filter_map(Result::ok)
            .collect()
    };

    info!("📦 {}件の商品をチェックします", products.len());

    let mut success_count = 0;
    let mut error_count = 0;

    // 🔧 &products に変更（参照でイテレート）
    for (product_id, url, name) in &products {
        info!("🔍 チェック中: {}", name);

        match fetch_amazon_price(url).await {
            Ok(product) => {
                let conn = db.lock().unwrap();
                if let Err(e) = save_product(&conn, &product) {
                    error!("❌ 商品保存エラー ({}): {}", name, e);
                    error_count += 1;
                    continue;
                }
                if let Err(e) = save_price_history(&conn, *product_id, product.price) {
                    error!("❌ 価格履歴保存エラー ({}): {}", name, e);
                    error_count += 1;
                    continue;
                }
                drop(conn);

                info!("✅ 更新成功: {} - ¥{}", name, product.price);
                success_count += 1;

                // レート制限対策: 各リクエスト間に1秒待機
                let _ = tokio::time::sleep(tokio::time::Duration::from_secs(1));
            }
            Err(e) => {
                error!("❌ スクレイピングエラー ({}): {}", name, e);
                error_count += 1;
            }
        }
    }

    info!(
        "📊 結果: 成功 {}件 / エラー {}件 / 合計 {}件",
        success_count,
        error_count,
        products.len()
    );

    Ok(())
}

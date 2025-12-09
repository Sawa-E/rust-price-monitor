use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;

fn main() -> Result<()> {
    println!("🌱 サンプルデータを投入中...");

    // データベース接続
    let conn = Connection::open("products.db")?;

    // テーブル作成
    conn.execute(
        "CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            current_price INTEGER NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS price_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            product_id INTEGER NOT NULL,
            price INTEGER NOT NULL,
            checked_at TEXT NOT NULL,
            FOREIGN KEY (product_id) REFERENCES products(id)
        )",
        [],
    )?;

    // サンプル商品データ
    let sample_products = vec![
        (
            "https://www.amazon.co.jp/dp/B08CF1RXD9",
            "コカ・コーラ 爽健美茶 ラベルレス 600ml ×24本",
            2980,
        ),
        (
            "https://www.amazon.co.jp/dp/B0D1XD1ZV3",
            "アサヒ飲料 カルピスウォーター 500ml×24本",
            2450,
        ),
        (
            "https://www.amazon.co.jp/dp/B09TQXZM3K",
            "サントリー 天然水 550ml×24本",
            1980,
        ),
        (
            "https://www.amazon.co.jp/dp/B07VXQJ8K5",
            "伊藤園 おーいお茶 緑茶 525ml×24本",
            2280,
        ),
        (
            "https://www.amazon.co.jp/dp/B08XYQWQR7",
            "キリン 午後の紅茶 ストレートティー 500ml×24本",
            2680,
        ),
    ];

    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    for (url, name, price) in sample_products {
        // 商品を挿入（既存の場合はスキップ）
        let result = conn.execute(
            "INSERT OR IGNORE INTO products (url, name, current_price, created_at) 
             VALUES (?1, ?2, ?3, ?4)",
            [url, name, &price.to_string(), &now],
        )?;

        if result > 0 {
            // 挿入された商品のIDを取得
            let product_id: i64 = conn.query_row(
                "SELECT id FROM products WHERE url = ?1",
                [url],
                |row| row.get(0),
            )?;

            println!("✅ 商品追加: {} (ID: {})", name, product_id);

            // 価格履歴を生成（過去7日分）
            use chrono::Duration;
            use rand::Rng;

            let mut rng = rand::thread_rng();
            let base_price = price;

            for days_ago in (0..7).rev() {
                let checked_at = (Utc::now() - Duration::days(days_ago))
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();

                // 価格を±10%でランダムに変動
                let variation = rng.gen_range(-10..=10);
                let history_price = base_price + (base_price * variation / 100);

                conn.execute(
                    "INSERT INTO price_history (product_id, price, checked_at) 
                     VALUES (?1, ?2, ?3)",
                    [&product_id.to_string(), &history_price.to_string(), &checked_at],
                )?;
            }

            println!("   📊 価格履歴7件を追加");
        } else {
            println!("⏭️  スキップ: {} (既に存在)", name);
        }
    }

    println!("\n🎉 サンプルデータの投入が完了しました！");
    println!("📁 データベース: products.db");

    Ok(())
}

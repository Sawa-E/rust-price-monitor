use anyhow::Result;
use rusqlite::Connection;

use crate::db::{save_price_history, save_product};
use crate::scraper::fetch_amazon_price;

// 🔧 async fnに変更
pub async fn cmd_add(conn: &Connection, url: &str) -> Result<()> {
    println!("追加中: {}", url);
    
    let product = fetch_amazon_price(url).await?;
    let product_id = save_product(conn, &product)?;
    save_price_history(conn, product_id, product.price)?;

    println!("✅ 商品を追加しました: {}", product.name);
    println!("   価格: ¥{}", product.price);

    Ok(())
}

pub fn cmd_list(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT id, url, name, current_price FROM products ORDER BY id DESC")?;

    let products = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i32>(3)?,
        ))
    })?;

    println!("\n📦 登録商品一覧:");
    println!("{}", "=".repeat(80));

    for (i, product) in products.enumerate() {
        if let Ok((id, url, name, price)) = product {
            println!("{}. [ID:{}] {}", i + 1, id, name);
            println!("   価格: ¥{}", price);
            println!("   URL: {}", url);
            println!("{}", "-".repeat(80));
        }
    }

    Ok(())
}

// 🔧 async fnに変更
pub async fn cmd_check(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT id, url, name, current_price FROM products")?;

    let products: Vec<(i64, String, String, i32)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?
        .filter_map(Result::ok)
        .collect();

    drop(stmt);

    println!("\n🔄 価格チェック中...\n");

    for (product_id, url, old_name, old_price) in products {
        println!("チェック中: {} ...", old_name);

        match fetch_amazon_price(&url).await {
            Ok(product) => {
                save_product(conn, &product)?;
                save_price_history(conn, product_id, product.price)?;

                let diff = product.price - old_price;
                let status = if diff > 0 {
                    format!("📈 +¥{}", diff)
                } else if diff < 0 {
                    format!("📉 ¥{}", diff)
                } else {
                    "➡️  変動なし".to_string()
                };

                println!("  現在価格: ¥{} {}", product.price, status);
            }
            Err(e) => {
                eprintln!("  ⚠️  エラー: {}", e);
            }
        }
        println!();
    }

    Ok(())
}

pub fn cmd_export(conn: &Connection, filename: &str) -> Result<()> {
    use std::fs::File;

    let mut wtr = csv::Writer::from_writer(File::create(filename)?);

    wtr.write_record(&["id", "name", "url", "current_price"])?;

    let mut stmt = conn.prepare("SELECT id, name, url, current_price FROM products")?;
    let products = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i32>(3)?,
        ))
    })?;

    for product in products {
        if let Ok((id, name, url, price)) = product {
            wtr.write_record(&[
                id.to_string(),
                name,
                url,
                price.to_string(),
            ])?;
        }
    }

    wtr.flush()?;
    println!("✅ エクスポート完了: {}", filename);

    Ok(())
}

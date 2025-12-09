use anyhow::Result;
use rusqlite::Connection;
use crate::scraper::fetch_amazon_price;
use crate::db::{save_product, save_price_history};

pub fn cmd_add(conn: &Connection, url: &str) -> Result<()> {
    println!("🔍 商品情報を取得中...");
    let product = fetch_amazon_price(url)?;
    
    let product_id = save_product(conn, &product)?;
    save_price_history(conn, product_id, product.price)?;
    
    println!("✅ 商品を追加しました");
    println!("📦 商品名: {}", product.name);
    println!("💰 価格: ¥{}", product.price);
    
    Ok(())
}

pub fn cmd_list(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, name, current_price, url FROM products ORDER BY id"
    )?;
    
    let products = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i32>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    
    println!("\n📋 登録済み商品一覧");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut count = 0;
    for product in products {
        let (id, name, price, url) = product?;
        count += 1;
        println!("\n[{}] {}", id, name);
        println!("    💰 ¥{}", price);
        println!("    🔗 {}", url);
    }
    
    if count == 0 {
        println!("商品が登録されていません");
        println!("'cargo run -- add <URL>' で商品を追加してください");
    } else {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("合計: {}件", count);
    }
    
    Ok(())
}

pub fn cmd_check(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, url, name, current_price FROM products ORDER BY id"
    )?;
    
    let products = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i32>(3)?,
        ))
    })?;
    
    println!("\n🔍 価格チェック開始...\n");
    
    for product in products {
        let (id, url, name, old_price) = product?;
        
        print!("チェック中: {} ... ", name);
        
        match fetch_amazon_price(&url) {
            Ok(current_product) => {
                let new_price = current_product.price;
                
                if new_price != old_price {
                    save_product(conn, &current_product)?;
                    
                    let diff = new_price - old_price;
                    if diff < 0 {
                        println!("⬇️  ¥{} → ¥{} ({}円安)", old_price, new_price, -diff);
                    } else {
                        println!("⬆️  ¥{} → ¥{} ({}円高)", old_price, new_price, diff);
                    }
                } else {
                    println!("変動なし (¥{})", new_price);
                }
                
                save_price_history(conn, id, new_price)?;
            }
            Err(e) => {
                println!("❌ エラー: {}", e);
            }
        }
    }
    
    println!("\n✅ チェック完了");
    Ok(())
}

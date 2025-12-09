use anyhow::Result;
use rusqlite::Connection;
use chrono::Utc;
use crate::scraper::Product;

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("products.db")?;
    
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
    
    Ok(conn)
}

pub fn save_product(conn: &Connection, product: &Product) -> Result<i64> {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    conn.execute(
        "INSERT INTO products (url, name, current_price, created_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(url) DO UPDATE SET
         name = excluded.name,
         current_price = excluded.current_price",
        (&product.url, &product.name, product.price, &now),
    )?;
    
    let product_id: i64 = conn.query_row(
        "SELECT id FROM products WHERE url = ?1",
        [&product.url],
        |row| row.get(0),
    )?;
    
    Ok(product_id)
}

pub fn save_price_history(conn: &Connection, product_id: i64, price: i32) -> Result<()> {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    conn.execute(
        "INSERT INTO price_history (product_id, price, checked_at)
         VALUES (?1, ?2, ?3)",
        (product_id, price, &now),
    )?;
    
    Ok(())
}

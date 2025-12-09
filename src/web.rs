use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;
use crate::scheduler;

use crate::db::{init_db, save_price_history, save_product};
use crate::scraper::fetch_amazon_price;
use tracing::info;

// 共有DB接続（スレッドセーフ）
pub type SharedDb = Arc<Mutex<Connection>>;

// APIレスポンス用の構造体
#[derive(Serialize)]
struct Product {
    id: i64,
    url: String,
    name: String,
    current_price: i32,
}

#[derive(Serialize)]
struct PriceHistory {
    price: i32,
    checked_at: String,
}

#[derive(Deserialize)]
struct AddProductRequest {
    url: String,
}

// ルーター設定
pub fn create_router(db: SharedDb) -> Router {
    Router::new()
        .route("/api/products", get(list_products).post(add_product))
        .route("/api/products/check", post(check_prices))
        .route("/api/products/:id/history", get(get_price_history))
        .route("/api/products/:id", axum::routing::delete(delete_product))  // 🆕 追加
        .with_state(db)
        .nest_service("/", ServeDir::new("static"))
}

// GET /api/products - 商品一覧取得
async fn list_products(State(db): State<SharedDb>) -> Result<Json<Vec<Product>>, StatusCode> {
    let conn = db.lock().unwrap();
    
    let mut stmt = conn
        .prepare("SELECT id, url, name, current_price FROM products ORDER BY id DESC")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let products: Vec<Product> = stmt
        .query_map([], |row| {
            Ok(Product {
                id: row.get(0)?,
                url: row.get(1)?,
                name: row.get(2)?,
                current_price: row.get(3)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(Result::ok)
        .collect();

    Ok(Json(products))
}

// POST /api/products - 商品追加
async fn add_product(
    State(db): State<SharedDb>,
    Json(req): Json<AddProductRequest>,
) -> Result<Json<Product>, StatusCode> {
    let url = req.url.clone();
    
    // スクレイピング実行
    let product = fetch_amazon_price(&url)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // DB保存
    let conn = db.lock().unwrap();
    let product_id = save_product(&conn, &product)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    save_price_history(&conn, product_id, product.price)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Product {
        id: product_id,
        url: product.url.clone(),
        name: product.name.clone(),
        current_price: product.price,
    }))
}

// POST /api/products/check - 全商品の価格チェック
async fn check_prices(State(db): State<SharedDb>) -> Result<Json<Vec<Product>>, StatusCode> {
    // 先にDB接続を取得してデータを全部読み込む
    let products: Vec<(i64, String, String, i32)> = {
        let conn = db.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, url, name, current_price FROM products")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .filter_map(Result::ok)
            .collect()
    };

    // futures::future::join_allで並列実行
    use futures::future::join_all;

    let tasks: Vec<_> = products
        .into_iter()
        .map(|(product_id, url, _name, _old_price)| {
            let db = db.clone();
            async move {
                if let Ok(product) = fetch_amazon_price(&url).await {
                    let conn = db.lock().unwrap();
                    let _ = save_product(&conn, &product);
                    let _ = save_price_history(&conn, product_id, product.price);
                    drop(conn);

                    Some(Product {
                        id: product_id,
                        url: product.url.clone(),
                        name: product.name.clone(),
                        current_price: product.price,
                    })
                } else {
                    None
                }
            }
        })
        .collect();

    let results = join_all(tasks).await;
    let updated_products: Vec<Product> = results.into_iter().filter_map(|x| x).collect();

    Ok(Json(updated_products))
}

async fn get_price_history(
    State(db): State<SharedDb>,
    axum::extract::Path(product_id): axum::extract::Path<i64>,
) -> Result<Json<Vec<PriceHistory>>, StatusCode> {
    let conn = db.lock().unwrap();

    let mut stmt = conn
        .prepare(
            "SELECT price, checked_at FROM price_history 
             WHERE product_id = ? 
             ORDER BY checked_at ASC"
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let history: Vec<PriceHistory> = stmt
        .query_map([product_id], |row| {
            Ok(PriceHistory {
                price: row.get(0)?,
                checked_at: row.get(1)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(Result::ok)
        .collect();

    Ok(Json(history))
}

async fn delete_product(
    State(db): State<SharedDb>,
    axum::extract::Path(product_id): axum::extract::Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let conn = db.lock().unwrap();

    // 価格履歴を削除
    conn.execute("DELETE FROM price_history WHERE product_id = ?", [product_id])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 商品を削除
    let deleted = conn.execute("DELETE FROM products WHERE id = ?", [product_id])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

// サーバー起動関数
pub async fn run_server() -> anyhow::Result<()> {
    let db = Arc::new(Mutex::new(init_db()?));
    let app = create_router(db.clone());

    tokio::spawn(async move {
        if let Err(e) = scheduler::start_scheduler(db).await {
            tracing::error!("❌ スケジューラー起動エラー: {}", e);
        }
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await?;

    println!("🚀 Server running at http://127.0.0.1:3000");

    axum::serve(listener, app).await?;
    Ok(())
}

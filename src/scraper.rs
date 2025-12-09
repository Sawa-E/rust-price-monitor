use anyhow::Result;
use scraper::{Html, Selector};

pub struct Product {
    pub name: String,
    pub price: i32,
    pub url: String,
}

// 🔧 async fnに変更、reqwest::blockingを削除
pub async fn fetch_amazon_price(url: &str) -> Result<Product> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36")
        .build()?;

    // 🔧 .awaitを追加
    let resp = client.get(url).send().await?;
    let body = resp.text().await?;

    // parse HTML
    let document = Html::parse_document(&body);

    // extract product title
    let title_selector = Selector::parse("#productTitle").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .ok_or_else(|| anyhow::anyhow!("Product title not found"))?
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    // extract product price
    let price_selector = Selector::parse(".a-price .a-offscreen").unwrap();
    let price_text = document
        .select(&price_selector)
        .next()
        .ok_or_else(|| anyhow::anyhow!("Product price not found"))?
        .text()
        .collect::<String>();

    // convert price to integer (remove ¥ and comma)
    let price: i32 = price_text
        .chars()
        .filter(|c| c.is_numeric())
        .collect::<String>()
        .parse()
        .map_err(|_| anyhow::anyhow!("Failed to parse price: {}", price_text))?;
    
    Ok(Product {
        name: title,
        price,
        url: url.to_string(),
    })
}

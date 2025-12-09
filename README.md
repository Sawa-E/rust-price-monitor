# 📊 rust-price-monitor

Amazon 商品の価格を自動で追跡・監視する Web ダッシュボードアプリケーション

![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)

## ✨ 特徴

- 🔍 **自動価格取得**: Amazon 商品ページから自動でスクレイピング
- 📈 **価格履歴グラフ**: Chart.js を使った視覚的な価格推移表示
- ⏰ **定期実行**: 毎時自動で価格をチェック（カスタマイズ可能）
- 🌓 **ダークモード**: 目に優しいダークテーマ対応
- 💾 **SQLite データベース**: 軽量で高速なデータ管理
- 🎨 **モダン UI**: レスポンシブデザインでスマホ対応
- 🚀 **高速**: Rust の性能を活かした爆速スクレイピング

## 🖼️ スクリーンショット

### ライトモード

_価格追跡ダッシュボード - 登録商品一覧と統計情報_

### ダークモード

_目に優しいダークテーマで夜間の作業も快適_

### 価格履歴グラフ

_過去の価格変動を折れ線グラフで可視化_

## 🎯 使用技術

### バックエンド

- **Rust 1.75+** - システムプログラミング言語
- **Axum 0.7** - 高速 Web フレームワーク
- **Tokio** - 非同期ランタイム
- **SQLite (rusqlite)** - 軽量データベース
- **reqwest** - HTTP クライアント
- **scraper** - HTML パーサー
- **tokio-cron-scheduler** - 定期実行スケジューラー

### フロントエンド

- **HTML5/CSS3** - モダンな UI 設計
- **Vanilla JavaScript** - 軽量で高速
- **Chart.js 4.4** - グラフ描画ライブラリ

## 📦 インストール

### 必要要件

- Rust 1.75 以上
- Cargo（Rust に付属）

### セットアップ

1. **リポジトリをクローン**

```bash
git clone https://github.com/yourusername/rust-price-monitor.git
cd rust-price-monitor
```

2. **依存関係をインストール**

```bash
cargo build
```

3. **サンプルデータを投入（オプション）**

```bash
cargo run --bin seed_db
```

4. **サーバーを起動**

```bash
cargo run
```

5. **ブラウザでアクセス**

```
http://127.0.0.1:3000
```

## 🚀 使い方

### Web UI（推奨）

#### 商品を追加

1. 入力欄に Amazon 商品 URL を貼り付け
2. 「➕ 追加」ボタンをクリック
3. スクレイピング完了後、商品カードが表示される

**対応 URL 形式:**

```
https://www.amazon.co.jp/dp/B08CF1RXD9
https://www.amazon.co.jp/商品名/dp/B08CF1RXD9/...
```

#### 価格チェック

- **手動**: 「🔄 価格チェック」ボタンをクリック
- **自動**: サーバー起動中は毎時 0 分に自動実行

#### 価格履歴グラフ

1. 商品カードの「📈 グラフ」ボタンをクリック
2. 過去の価格変動グラフが表示される
3. ホバーで詳細価格を確認

#### その他の機能

- **商品削除**: 「🗑️ 削除」ボタン
- **ダークモード**: 右上の「🌙/☀️」ボタン
- **統計情報**: 登録商品数、平均価格、最安値を表示

### CLI コマンド

#### 商品を追加

```bash
cargo run -- add "https://www.amazon.co.jp/dp/B08CF1RXD9"
```

#### 商品一覧を表示

```bash
cargo run -- list
```

#### 価格チェック

```bash
cargo run -- check
```

#### CSV エクスポート

```bash
cargo run -- export products.csv
```

## ⚙️ 設定

### 定期実行の頻度を変更

`src/scheduler.rs` の 28 行目を編集:

```rust
// 毎時0分（デフォルト）
let job = Job::new_async("0 0 * * * *", move |_uuid, _lock| {
```

**Cron 式の例:**

| 実行頻度   | Cron 式            |
| ---------- | ------------------ |
| 毎時 0 分  | `"0 0 * * * *"`    |
| 30 分ごと  | `"0 */30 * * * *"` |
| 15 分ごと  | `"0 */15 * * * *"` |
| 6 時間ごと | `"0 0 */6 * * *"`  |
| 毎日 9 時  | `"0 0 9 * * *"`    |
| 平日 9 時  | `"0 0 9 * * 1-5"`  |

変更後、再ビルド:

```bash
cargo build
```

### ポート番号を変更

`src/web.rs` の最後の方を編集:

```rust
let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
```

## 📁 プロジェクト構成

```
rust-price-monitor/
├── src/
│   ├── main.rs           # エントリーポイント
│   ├── cli.rs            # CLIコマンド定義
│   ├── db.rs             # データベース操作
│   ├── scraper.rs        # Webスクレイピング
│   ├── commands.rs       # CLIコマンド実装
│   ├── web.rs            # Webサーバー（Axum）
│   ├── scheduler.rs      # 定期実行スケジューラー
│   └── bin/
│       └── seed_db.rs    # サンプルデータ投入
├── static/
│   ├── index.html        # メインHTML
│   ├── css/
│   │   └── style.css     # スタイルシート
│   ├── js/
│   │   └── app.js        # JavaScript
│   └── favicon.png       # ファビコン
├── Cargo.toml            # 依存関係定義
├── products.db           # SQLiteデータベース（自動生成）
└── README.md             # このファイル
```

## 🗄️ データベーススキーマ

### products テーブル

```sql
CREATE TABLE products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    current_price INTEGER NOT NULL,
    created_at TEXT NOT NULL
);
```

### price_history テーブル

```sql
CREATE TABLE price_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_id INTEGER NOT NULL,
    price INTEGER NOT NULL,
    checked_at TEXT NOT NULL,
    FOREIGN KEY (product_id) REFERENCES products(id)
);
```

## 🛠️ トラブルシューティング

### 商品が追加できない

**原因:**

- URL が無効
- Amazon のセレクタが変更された
- ネットワークエラー

**対処法:**

```bash
# URLを確認（/dp/商品IDの形式か）
# ブラウザで直接アクセスできるか確認
```

### ポートが使用中

**エラー:**

```
Error: Address already in use (os error 10048)
```

**対処法:**

- タスクマネージャーで該当プロセスを終了
- または `src/web.rs` でポート番号を変更

### スケジューラーが動作しない

**確認方法:**

```bash
# ログに以下が表示されているか確認
✅ スケジューラーが起動しました（毎時0分に実行）
```

**テスト用に変更:**

```rust
// src/scheduler.rs の28行目を変更（毎分実行）
"0 * * * * *"
```

## 🚧 今後の拡張予定

- [ ] Discord/Slack 通知機能
- [ ] 目標価格設定とアラート
- [ ] 商品画像の表示
- [ ] フィルター・検索機能
- [ ] ユーザー認証（複数ユーザー対応）
- [ ] 価格予測（機械学習）
- [ ] クラウドデプロイ

## 📝 ライセンス

MIT License

---

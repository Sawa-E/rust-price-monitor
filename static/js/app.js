// テーマ管理
const savedTheme = localStorage.getItem("theme") || "light";
document.documentElement.setAttribute("data-theme", savedTheme);
updateThemeIcon();

function toggleTheme() {
  const current = document.documentElement.getAttribute("data-theme");
  const newTheme = current === "light" ? "dark" : "light";
  document.documentElement.setAttribute("data-theme", newTheme);
  localStorage.setItem("theme", newTheme);
  updateThemeIcon();
  showToast("テーマを変更しました", "info");
}

function updateThemeIcon() {
  const theme = document.documentElement.getAttribute("data-theme");
  const toggleBtn = document.querySelector(".theme-toggle");
  if (toggleBtn) {
    toggleBtn.textContent = theme === "light" ? "🌙" : "☀️";
  }
}

// トースト通知
function showToast(message, type = "info") {
  const toast = document.getElementById("toast");
  toast.textContent = message;
  toast.className = `toast ${type} show`;
  setTimeout(() => {
    toast.classList.remove("show");
  }, 3000);
}

// 初回読み込み
loadProducts();

async function loadProducts() {
  try {
    const res = await fetch("/api/products");
    const products = await res.json();
    displayProducts(products);
    updateStats(products);
  } catch (err) {
    document.getElementById("productList").innerHTML =
      '<div class="loading">エラーが発生しました</div>';
    showToast("商品の読み込みに失敗しました", "error");
  }
}

function updateStats(products) {
  const total = products.length;
  const prices = products.map((p) => p.current_price);
  const avg = prices.length
    ? Math.round(prices.reduce((a, b) => a + b, 0) / prices.length)
    : 0;
  const lowest = prices.length ? Math.min(...prices) : 0;

  document.getElementById("totalProducts").textContent = total;
  document.getElementById("avgPrice").textContent = "¥" + avg.toLocaleString();
  document.getElementById("lowestPrice").textContent =
    "¥" + lowest.toLocaleString();
}

async function addProduct() {
  const url = document.getElementById("urlInput").value.trim();
  if (!url) {
    showToast("URLを入力してください", "error");
    return;
  }

  document.getElementById("productList").innerHTML =
    '<div class="loading"><div class="spinner"></div>追加中...</div>';

  try {
    const res = await fetch("/api/products", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ url }),
    });

    if (res.ok) {
      document.getElementById("urlInput").value = "";
      loadProducts();
      showToast("✅ 商品を追加しました！", "success");
    } else {
      showToast("❌ 商品の追加に失敗しました", "error");
      loadProducts();
    }
  } catch (err) {
    showToast("❌ エラーが発生しました", "error");
    loadProducts();
  }
}

async function checkPrices() {
  document.getElementById("productList").innerHTML =
    '<div class="loading"><div class="spinner"></div>価格チェック中...</div>';

  try {
    const res = await fetch("/api/products/check", { method: "POST" });
    const products = await res.json();
    displayProducts(products);
    updateStats(products);
    showToast("✅ 価格チェック完了！", "success");
  } catch (err) {
    showToast("❌ 価格チェックに失敗しました", "error");
    loadProducts();
  }
}

async function deleteProduct(id, name) {
  if (!confirm(`「${name}」を削除しますか？`)) return;

  try {
    const res = await fetch(`/api/products/${id}`, { method: "DELETE" });
    if (res.ok) {
      loadProducts();
      showToast("🗑️ 商品を削除しました", "success");
    } else {
      showToast("❌ 削除に失敗しました", "error");
    }
  } catch (err) {
    showToast("❌ エラーが発生しました", "error");
  }
}

function displayProducts(products) {
  const list = document.getElementById("productList");
  if (products.length === 0) {
    list.innerHTML = '<div class="loading">📦 商品が登録されていません</div>';
    return;
  }

  list.innerHTML = products
    .map(
      (p) => `
        <div class="product-card">
            <div class="product-header">
                <div class="product-info">
                    <h3>${escapeHtml(p.name)}</h3>
                    <div class="price-container">
                        <div class="price">¥${p.current_price.toLocaleString()}</div>
                    </div>
                    <div class="url">${escapeHtml(p.url)}</div>
                </div>
                <div class="product-actions">
                    <button class="btn-graph" onclick="toggleGraph(${
                      p.id
                    })">📈 グラフ</button>
                    <button class="btn-delete btn-danger" onclick="deleteProduct(${
                      p.id
                    }, '${escapeHtml(p.name).replace(
        /'/g,
        "\\'"
      )}')">🗑️ 削除</button>
                </div>
            </div>
            <div class="chart-container" id="chart-${p.id}">
                <canvas id="canvas-${p.id}"></canvas>
            </div>
        </div>
    `
    )
    .join("");
}

function escapeHtml(text) {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

// グラフ表示
const chartInstances = {};

async function toggleGraph(productId) {
  const chartContainer = document.getElementById(`chart-${productId}`);

  if (chartContainer.classList.contains("active")) {
    chartContainer.classList.remove("active");
    if (chartInstances[productId]) {
      chartInstances[productId].destroy();
      delete chartInstances[productId];
    }
    return;
  }

  chartContainer.classList.add("active");

  try {
    const res = await fetch(`/api/products/${productId}/history`);
    const history = await res.json();

    if (history.length === 0) {
      chartContainer.innerHTML =
        '<p style="text-align:center; padding:2rem; color:var(--text-secondary);">📊 価格履歴がありません</p>';
      return;
    }

    const labels = history.map((h) => {
      const date = new Date(h.checked_at);
      return `${
        date.getMonth() + 1
      }/${date.getDate()} ${date.getHours()}:${String(
        date.getMinutes()
      ).padStart(2, "0")}`;
    });
    const prices = history.map((h) => h.price);

    const ctx = document.getElementById(`canvas-${productId}`).getContext("2d");

    if (chartInstances[productId]) {
      chartInstances[productId].destroy();
    }

    const isDark =
      document.documentElement.getAttribute("data-theme") === "dark";

    chartInstances[productId] = new Chart(ctx, {
      type: "line",
      data: {
        labels: labels,
        datasets: [
          {
            label: "価格（円）",
            data: prices,
            borderColor: "#667eea",
            backgroundColor: "rgba(102, 126, 234, 0.1)",
            borderWidth: 3,
            tension: 0.4,
            fill: true,
            pointRadius: 5,
            pointHoverRadius: 8,
            pointBackgroundColor: "#667eea",
            pointBorderColor: "#fff",
            pointBorderWidth: 2,
            pointHoverBackgroundColor: "#667eea",
            pointHoverBorderColor: "#fff",
            pointHoverBorderWidth: 3,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: true,
        interaction: {
          intersect: false,
          mode: "index",
        },
        plugins: {
          legend: {
            display: true,
            position: "top",
            labels: {
              color: isDark ? "#f7fafc" : "#2d3748",
              font: {
                size: 12,
                weight: 600,
              },
            },
          },
          tooltip: {
            backgroundColor: isDark ? "#2d3748" : "#fff",
            titleColor: isDark ? "#f7fafc" : "#2d3748",
            bodyColor: isDark ? "#f7fafc" : "#2d3748",
            borderColor: "#667eea",
            borderWidth: 2,
            padding: 12,
            displayColors: false,
            callbacks: {
              label: function (context) {
                return "¥" + context.parsed.y.toLocaleString();
              },
            },
          },
        },
        scales: {
          y: {
            beginAtZero: false,
            ticks: {
              color: isDark ? "#a0aec0" : "#718096",
              callback: function (value) {
                return "¥" + value.toLocaleString();
              },
            },
            grid: {
              color: isDark
                ? "rgba(160, 174, 192, 0.1)"
                : "rgba(226, 232, 240, 0.8)",
            },
          },
          x: {
            ticks: {
              color: isDark ? "#a0aec0" : "#718096",
            },
            grid: {
              color: isDark
                ? "rgba(160, 174, 192, 0.1)"
                : "rgba(226, 232, 240, 0.8)",
            },
          },
        },
      },
    });
  } catch (err) {
    console.error("グラフ表示エラー:", err);
    chartContainer.innerHTML =
      '<p style="text-align:center; padding:2rem; color:var(--danger-color);">❌ グラフの読み込みに失敗しました</p>';
  }
}

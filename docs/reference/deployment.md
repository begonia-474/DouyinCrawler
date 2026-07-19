# 部署文档

## 环境要求

### 系统要求

| 依赖 | 最低版本 | 说明 |
|------|----------|------|
| Node.js | 18+ | 推荐 20 LTS |
| pnpm | 8+ | 包管理器 |
| Rust | 1.77+ | Tauri 2 要求 |
| Python | 3.11+ | 推荐 3.13 |
| 系统 | Windows / macOS / Linux | Tauri 2 支持的平台 |

### 平台特定依赖

**Windows**：
- Microsoft Visual C++ Build Tools
- WebView2（Windows 10+ 自带）

**macOS**：
- Xcode Command Line Tools

**Linux**：
- 依赖 Tauri 2 的系统库，参考 [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)

---

## 安装

### 1. 克隆仓库

```bash
git clone https://github.com/<your-username>/DouyinCrawler.git
cd DouyinCrawler
```

### 2. Python 环境

准备 Python 3.11+ 环境。可以使用 Conda、venv 或其他环境管理方式：

```bash
# 安装依赖（使用 lock 文件锁定版本）
pip install -r requirements.txt -c requirements.lock
```

### 3. 前端依赖

```bash
pnpm install
```

### 4. 验证安装

```bash
# 前端构建检查
pnpm build

# Rust 编译检查
cd src-tauri && cargo check && cd ..

# Python 离线测试
python -m pytest -m offline
```

---

## 开发

### 启动完整桌面应用

```bash
pnpm tauri dev
```

此命令同时启动 Vite 前端开发服务器和 Tauri Rust 后端。前端支持 HMR 热更新。

### 仅启动前端

```bash
pnpm dev
```

仅启动 Vite 开发服务器（端口 5173），不启动 Tauri 后端。适用于纯前端开发。

### 代码检查

```bash
# ESLint
pnpm lint

# TypeScript 类型检查（包含在 pnpm build 中）
pnpm build
```

---

## 配置

### 运行时配置

配置文件位于 `config/app.json`，首次运行时自动创建。主要字段：

```json
{
  "cookie": "YOUR_DOUYIN_COOKIE",
  "download_dir": "Download",
  "db_path": "data/douyin.db",
  "max_concurrent_downloads": 3,
  "theme": "system"
}
```

### Cookie 获取

抖音 Cookie 是爬虫工作的必要条件。获取方式：

1. **浏览器手动复制**：登录抖音网页版 → F12 → Application → Cookies → 复制全部
2. **Firefox 自动提取**：应用内置 Firefox Cookie 提取功能（设置页面）

> **安全提示**：`config/` 目录已被 `.gitignore` 忽略。Cookie 属于敏感凭据，切勿提交到版本控制或公开分享。

### 数据目录

| 路径 | 说明 | 是否忽略 |
|------|------|----------|
| `config/app.json` | 运行时配置 | 是 |
| `data/douyin.db` | SQLite 数据库 | 是 |
| `Download/` | 默认下载目录 | 是 |

---

## 生产构建

### 构建桌面应用

```bash
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`：

- **Windows**：`.msi` 安装包 + `.exe`
- **macOS**：`.dmg` + `.app`
- **Linux**：`.deb` + `.AppImage`

### 仅构建前端

```bash
pnpm build
```

输出到 `dist/` 目录。

---

## 类型生成

修改 Rust 侧暴露的类型后，需要重新生成前端类型：

```bash
python scripts/gen_tauri_types.py
```

生成目标：`src/lib/tauri-types.ts`。

---

## 测试

### 离线测试（推荐提交前运行）

```bash
python -m pytest -m offline
```

### 完整集成测试

需要有效 Cookie 和网络环境：

```bash
# 使用已准备好的 Python 环境

# 运行全模式测试
PYTHONIOENCODING=utf-8 python test/test_all_modes.py

# 端到端验证
PYTHONIOENCODING=utf-8 python test/e2e_verify.py
```

### 提交前检查清单

```bash
pnpm build                              # 前端类型检查 + 构建
pnpm lint                               # ESLint
cd src-tauri && cargo check && cd ..    # Rust 编译
python -m pytest -m offline             # Python 离线测试
```

---

## 常见问题

### Python 环境激活失败

确保 Python 版本 >= 3.11：

```bash
python --version
```

### PyO3 编译错误

确保 Rust 工具链是最新的：

```bash
rustup update
```

### 前端类型不匹配

修改 Rust 类型后运行类型生成：

```bash
python scripts/gen_tauri_types.py
```

### SQLite 数据库锁定

应用使用 WAL 模式，正常情况下不会出现锁定。如遇问题，检查是否有其他进程占用 `data/douyin.db`。

### 下载失败

1. 检查 Cookie 是否过期（设置页面重新获取）
2. 检查网络连接
3. 查看应用状态栏的错误信息

# 字幕共享计划 (Subtitles DataBase) · GitHub Pages

> **轻量、优雅、纯粹**。专为 [foxofice/sub_share](https://github.com/foxofice/sub_share) 打造的现代化番剧字幕检索与导航门户。

---

## 🌟 核心特性

1. **⚡ 实时全量检索（支持 4,700+ 套字幕 / 2,100+ 部独立番剧）**
   - 毫秒级多维度匹配：支持中文译名、别名、日文原名、英文名、罗马音、字幕组、来源（BD/TV/Web）、格式（.ass/.srt）、目录路径。
   - 支持多关键词空格联合检索（例如输入 `2024 BD LoliHouse` 可精准筛选）。
   - 快捷键支持：任何位置按下 `/` 键立即自动聚焦搜索框。

2. **📦 db.json.zst 优先加载，无感多级降级**
   - **首选在线数据源**：优先从 CDN 加载 `Subtitles DataBase/Files/db.json.zst`（约 229KB，解压后约 2.1MB / 4,600+ 条），体积仅为传统 JSON 索引的约八分之一，首屏更快、流量更省。
   - **浏览器端解压**：使用轻量 `fzstd`（纯 JS，约 8KB）在浏览器内完成 Zstandard 解压，无需服务端，字段自动适配（中文/日文/罗马音、id、路径规范化）。
   - **无感降级链**：zst 下载或解压失败时，依次自动回退到本地 `subtitles.json` → 远程 `db.xml`（CDN / raw 多源），全程不阻断使用，仅在切换时给出弱提示。
   - **全局超时护栏**：单源 6s、解压组件 5s、本地 JSON 4s、XML 每源 4s，整体 12s 硬上限，杜绝长时间无反馈。

3. **🌸 封面在线优先加载（国内可靠源）**
   - **在线优先**：仓库内置封面走 **jsDelivr 国内 CDN** 在线加载，稳定、免防盗链、随仓库更新；其余番剧直连 **Bangumi 官方图域** `lain.bgm.tv`，覆盖全量条目。
   - **多级故障转移**：CDN 失败后依次尝试 `wsrv.nl` 反代、Jetpack CDN、同域本地 `covers/`，最后降级色块占位，图片极少空缺。
   - **防盗链优化**：图片加载携带 `referrerpolicy="no-referrer"`，解决国内访问图床的 403 阻断问题。
   - **智能并发限流**：视口懒加载（IntersectionObserver）与 2 并发队列，避免频繁触发 API 429 限流。
   - **双重本地持久化缓存**：已获取的海报与评分存于浏览器 `IndexedDB` / `localStorage`，再次访问瞬时呈现。
   - **预置 Seed 缓存**：内置 189 条热门/新入库番剧封面元数据（其中 128 张本地高清海报），初次进入页面秒级展现。

4. **📖 番剧详情与简介**
   - 点击卡片打开详情弹窗，展示 Bangumi 条目**具体简介**（通过 v0 条目详情接口补全，搜索接口本身不返回简介）、评分、封面与版本列表。
   - 简介获取后写回本地缓存，二次打开无需等待；快速切换番剧时自动作废旧请求，避免内容串台。
   - 内置 **一键复制路径** 与 **Git 稀疏检出（Sparse Checkout）克隆命令**，免去整库拉取 3GB 负担，秒级只拉取单部番剧字幕。

5. **📂 一键直达 GitHub 仓库位置**
   - 卡片上提供 **「仓库目录」** 按钮，点击直接跳转对应番剧在 GitHub master 分支的具体目录：
     `https://github.com/foxofice/sub_share/tree/master/subs_list/<分类>/<年份>/<番剧名>`。

6. **🔥 优先显示最近更新**
   - 依据 `db.xml` 的入库时序与 GitHub Commits 记录，自动将近期新增批次置顶。
   - 卡片展示醒目的 **「🔥 最新更新」** 标识，并在顶部显示最新 Commit 动态胶囊。
   - 支持按「最近更新优先」、「放送年份（新→旧）」、「放送年份（旧→新）」、「标题拼音 A-Z」随时切换排序。

7. **🎴 双视图模式切换**
   - **番剧聚合视图（推荐）**：将同一部番剧的多个字幕组（如 LoliHouse、冷番补完、今晚月色真美等）聚合为一张精美海报卡片，清晰整洁。
   - **单条字幕视图**：完整罗列全部明细条目，适合精细核对。

8. **📥 本地数据即时导入**
   - 设置弹窗支持拖拽 **db.json.zst** 或 **db.xml** 到本地，立即在浏览器中解析并预览检索，无需上传服务器。
   - 支持小样本/部分数据集，导入后自动完成字段适配与 R18 过滤。

9. **🔒 安全的 GitHub Token 设计**
   - 支持在网页中输入 GitHub Personal Access Token（PAT），仅保存在用户本地浏览器 `localStorage` 中，API 配额从 60 次/小时提升至 5,000 次/小时。
   - **杜绝硬编码**：不在公开前端仓库代码中暴露明文 Token，防止被 GitHub Secret Scanning 自动检测并吊销。

10. **♿ 响应式与无障碍**
    - 手机 / 平板 / 桌面自适应，避免横向滚动、元素重叠与文字溢出；统一 44px 触控命中区。
    - 完整键盘可达与 `:focus-visible` 焦点环，加载状态通过 `aria-live` 播报，支持 `prefers-reduced-motion` 降级。

---

## 📁 目录文件结构

```text
├── index.html          # 主页面 HTML5（语义化、无外部繁重框架、极致轻量）
├── styles.css          # 核心样式（深色/浅色双主题、Glassmorphism 毛玻璃卡片、响应式）
├── app.js              # 核心交互、zst 解压加载器与 Bangumi 浏览器端抓取器
├── subtitles.json      # 由 db.xml 预编译的精简 JSON 索引（约 1.7MB），作为 zst 失败时的本地降级源
├── seed_covers.json    # 189 条番剧封面与元数据预热缓存（含本地高清封面路径）
├── covers/             # 128 张本地高清海报（jsDelivr CDN / 同域兜底使用）
├── update-data.js      # （维护者脚本）当 db.xml 有新增字幕时，一键更新 json 与封面
├── .nojekyll           # 确保 GitHub Pages 正确托管静态资源
└── README.md           # 使用与部署文档
```

在线主数据源（无需随站点部署，由 app.js 直接按需加载）：

```text
Subtitles DataBase/Files/db.json.zst   # 压缩字幕索引，浏览器端 fzstd 解压后使用
```

---

## 🚀 部署与同步指南

> **当前线上状态提示**：截至本次更新，仓库的 GitHub Pages 设置仍指向 `master` 分支根目录。若要让本 `gh-pages` 分支的页面生效，请先在 **Settings -> Pages** 将 Source 切换为 `gh-pages` / `/ (root)`。切换后本分支即为线上页面源。

### 方案 A：直接推送到 `gh-pages` 分支（推荐）

1. 克隆或拉取当前仓库的 `gh-pages` 分支：
   ```bash
   git clone -b gh-pages https://github.com/foxofice/sub_share.git sub_share_ghpages
   cd sub_share_ghpages
   ```

2. 确认分支根目录包含以下文件（即本 README 的「目录文件结构」）：
   - `index.html`
   - `styles.css`
   - `app.js`
   - `subtitles.json`
   - `seed_covers.json`
   - `covers/`
   - `update-data.js`
   - `.nojekyll`

3. 提交并推送到 GitHub：
   ```bash
   git add .
   git commit -m "feat: 升级 GitHub Pages 检索门户"
   git push origin gh-pages
   ```

4. 访问 GitHub 仓库的 **Settings -> Pages**，将 Source 设置为 `Deploy from a branch`，Branch 选择 `gh-pages` / `/ (root)` 并保存。等待几十秒重新部署后即可访问。

---

### 方案 B：部署在 `master` 分支下的 `docs/` 目录

如果 Pages 保持在 `master` 分支，可将同样文件放入 `docs/`：

1. 在 `master` 分支根目录下创建 `docs/` 文件夹。
2. 将上述所有文件放入 `docs/` 中。
3. 提交并推送到 `master`。
4. 在仓库 **Settings -> Pages** 中将 Branch 设为 `master` / `/docs`。

---

## 🔄 数据更新方式

### 方式一：更新在线 zst 主数据源（推荐）

页面优先加载 `Subtitles DataBase/Files/db.json.zst`。当 master 分支的 `db.xml` 有新增条目后，重新生成对应的 `db.json.zst` 并提交到 master 分支即可，页面会在下次加载时自动使用最新数据，无需改动 gh-pages 分支的前端文件。

### 方式二：更新本地降级索引

若需要刷新本地降级用的 `subtitles.json` 与封面缓存，在项目目录下执行：

```bash
node update-data.js
```

脚本将自动重新生成最新的 `subtitles.json` 并增量补充最新番剧的封面信息，之后将变更提交推送即可完成网页更新。

### 方式三：浏览器内即时预览

无需推送，可在设置弹窗中直接拖拽本地 **db.json.zst** 或 **db.xml**，即时在浏览器中预览检索结果，适合数据维护前的快速验证。

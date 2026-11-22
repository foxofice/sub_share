# 字幕共享计划 (Subtitles DataBase) · GitHub Pages

> **轻量、优雅、纯粹**。专为 [foxofice/sub_share](https://github.com/foxofice/sub_share) 打造的全新现代化番剧字幕检索与导航门户。

---

## 🌟 核心特性

1. **⚡ 实时全量检索（支持 4,750+ 套字幕 / 2,100+ 部独立番剧）**
   - 毫秒级多维度匹配：支持中文译名、别名、日文原名、英文名、罗马音、字幕组、来源（BD/TV/Web）、格式（.ass/.srt）、目录路径。
   - 支持多关键词空格联合检索（例如输入 `2024 BD LoliHouse` 可精准筛选）。
   - 快捷键支持：任何位置按下 `/` 键立即自动聚焦搜索框。

2. **🌸 浏览器直连抓取番剧封面与元数据（优先国内高速直连）**
   - **国内免翻直连**：前端采用原生开放的 **Bangumi (bgm.tv)** API，原生支持 CORS 跨域。
   - **防盗链优化**：图片加载携带 `referrerpolicy="no-referrer"`，彻底解决国内访问 `lain.bgm.tv` 403 阻断问题。
   - **高可用与智能并发限流**：内置视口懒加载（IntersectionObserver）与 2 并发队列，避免频繁触发 API 429 限流。
   - **双重本地持久化缓存**：已获取的番剧海报与评分存储于浏览器本地 `IndexedDB` 与 `localStorage`，后续访问 0 延时瞬时呈现。
   - **预置 Seed 缓存**：首屏预载最新 60+ 部热门/新入库番剧封面，初次进入页面秒级展现海报。

3. **📂 一键直达 GitHub 仓库位置**
   - 卡片上提供 **「仓库目录」** 按钮，点击直接跳转对应番剧在 GitHub master 分支的具体目录：
     `https://github.com/foxofice/sub_share/tree/master/subs_list/<分类>/<年份>/<番剧名>`。
   - 详情弹窗内置 **一键复制路径** 与 **Git 稀疏检出（Sparse Checkout）克隆命令**，免去整库拉取 3GB 负担，秒级只拉取单部番剧字幕。

4. **🔥 优先显示最近更新**
   - 依据 `db.xml` 的入库时序与 GitHub Commits 记录，自动将近期新增批次（如近期提交的数十部番剧）置顶。
   - 卡片展示醒目的 **「🔥 最新更新」** 标识，并在顶部显示最新 Commit 动态胶囊。
   - 支持按「最近更新优先」、「放送年份（新→旧）」、「放送年份（旧→新）」、「标题拼音 A-Z」随时切换排序。

5. **🎴 双视图模式切换**
   - **番剧聚合视图（推荐）**：将同一部番剧的多个字幕组（如 LoliHouse、冷番补完、今晚月色真美等）聚合为一张精美海报卡片，清晰整洁。
   - **单条字幕视图**：完整罗列 4,750 条明细条目，适合精细核对。

6. **🔒 安全的 GitHub Token 设计**
   - 支持在网页中输入 GitHub Personal Access Token（PAT），仅保存在用户本地浏览器 `localStorage` 中，API 配额从 60次/小时 提升至 5,000次/小时。
   - **杜绝硬编码**：不在公开前端仓库代码中暴露明文 Token，防止被 GitHub Secret Scanning 自动检测并吊销。

---

## 📁 目录文件结构

```text
├── index.html          # 主页面 HTML5（语义化、无外部繁重框架、极致轻量）
├── styles.css          # 核心样式（支持深色/浅色模式、Glassmorphism 毛玻璃卡片）
├── app.js              # 核心交互与 Bangumi 浏览器端抓取器
├── subtitles.json      # 由 db.xml 预编译的精简 JSON 索引（约 1.7MB，Gzip 后仅 ~250KB）
├── seed_covers.json    # 最新 60 部番剧的封面与元数据预热缓存
├── update-data.js      # （维护者脚本）当 db.xml 有新增字幕时，一键更新 json 与封面
├── .nojekyll           # 确保 GitHub Pages 正确托管静态资源
└── README.md           # 使用与部署文档
```

---

## 🚀 部署与同步指南

### 方案 A：直接推送到 `gh-pages` 分支（推荐）

1. 克隆或拉取当前仓库的 `gh-pages` 分支：
   ```bash
   git clone -b gh-pages https://github.com/foxofice/sub_share.git sub_share_ghpages
   cd sub_share_ghpages
   ```

2. 将本项目中的文件复制到该分支根目录下：
   - `index.html`
   - `styles.css`
   - `app.js`
   - `subtitles.json`
   - `seed_covers.json`
   - `update-data.js`
   - `.nojekyll`

3. 提交并推送到 GitHub：
   ```bash
   git add .
   git commit -m "feat: 升级 GitHub Pages 为现代化番剧检索与封面直连系统"
   git push origin gh-pages
   ```

4. 访问 GitHub 仓库的 **Settings -> Pages**，将 Source 设置为 `Deploy from a branch`，Branch 选择 `gh-pages` / `/ (root)` 即可！

---

### 方案 B：部署在 `master` 分支下的 `docs/` 目录

如果习惯在主分支统一维护：
1. 在 `master` 分支根目录下创建 `docs/` 文件夹。
2. 将上述所有文件放入 `docs/` 中。
3. 提交并推送到 `master`。
4. 在仓库 **Settings -> Pages** 中将 Branch 设为 `master` / `/docs`。

---

## 🔄 当仓库添加新字幕后如何更新？

每次向 `Subtitles DataBase/Files/db.xml` 添加了新条目后，只需在项目目录下执行：

```bash
node update-data.js
```

脚本将自动重新生成最新的 `subtitles.json` 并增量补充最新番剧的封面信息，之后将变更提交推送即可完成网页更新！

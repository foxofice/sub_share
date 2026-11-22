# CI 说明：字幕数据库自动同步与 Pages 发布

本目录的 `workflows/pages.yml` 与 `scripts/sync-db.mjs` 负责两件事：**自动维护字幕数据库**、**稳定发布 Pages**。

## 工作流做了什么

1. `subs_list/**` 或 `Subtitles DataBase/Files/db.xml` 有变化时触发；
2. 用 `git ls-tree` + `--filter=blob:none` 只读目录树，**不下载任何字幕文件**；
3. 增量补齐 `db.xml`：只追加「新番剧 / 新字幕组」的记录，已存在记录原样保留；
4. 生成 `db.json.zst`（客户端用）与 `subtitles.json`（站点用）；
5. 为新番自动检索 Bangumi 封面，写入 `seed_covers.json`（每轮最多 20 部，取不到不阻塞）；
5. 分别提交到 `master` 与 `gh-pages`，再由本工作流统一部署 Pages。

提交信息带 `[skip ci]`，且 `GITHUB_TOKEN` 的推送不会触发新工作流，不会形成循环。

## 仓库设置要求

| 设置 | 位置 | 说明 |
| --- | --- | --- |
| Pages Source = **GitHub Actions** | Settings → Pages | 必须。分支构建与 Actions 部署会争抢 Pages 发布锁，导致互相取消；本工作流检测到仍为分支构建时会跳过发布并给出 warning |
| Workflow permissions = **Read and write** | Settings → Actions → General | 需要把生成的 `db.xml` / `db.json.zst` / `subtitles.json` 推回仓库；只读时工作流会给出 warning 而不失败 |
| `gh-pages` 分支存在 | Branches | 站点文件所在分支 |

## 脚本行为

- **封面**：只补 `seed_covers.json` 中不存在的条目，保留已有 `localCover`；结果按「名称完全匹配 + 放送年份接近」打分挑选，避免同名续作错配；接口不可达时本轮跳过，下次重试；
- **只追加不删除**：`db.xml` 中有、但目录已不存在的记录只在报告里列出，需人工确认后删除，避免误删；
- 名称优先读番剧目录下的 `info.txt`（`chs` / `cht` / `jp` / `en` / `rome`），缺失时回退为目录名；
- 来源与字幕组按固定层级识别：`番剧/<source>/<字幕组>/文件`；无 source 目录时默认为 `BD`，第一层即字幕组；
- 幂等：按 `path + source + sub_name` 去重，重复运行不会重复追加；
- `--check` 模式在发现待新增记录或失效记录时返回退出码 1，可用于 PR 校验。

## 本地运行

```bash
git clone --filter=blob:none --sparse https://github.com/foxofice/sub_share.git
cd sub_share
git sparse-checkout set --no-cone /scripts/ "/Subtitles DataBase/Files/db.xml"

node scripts/sync-db.mjs \
  --db "Subtitles DataBase/Files/db.xml" \
  --json site/subtitles.json \
  --dbjson db.json \
  --report report.md
```

同步报告会输出到 Actions 的 Step Summary，包含：扫描文件数、番剧数、字幕组数、新增记录、失效记录、缺少 `info.txt` 的新番、层级异常文件。

## 实测基准

以上游实际目录树为样本：

- 全量扫描：127,405 个文件 → 2,433 个番剧目录、5,393 条字幕组记录；
- 现有 `db.xml` 有 5,732 条记录，脚本判定「待新增 6 条、失效 74 条」，说明数据库与目录基本一致；
- 用 1970 / 2005 / 2010 三年共 5,451 个文件回放，生成结果的 `(path, source, sub_name, extension)` 与现有 `db.xml` 一致率 99.54%，这三年新增记录为 0。

## 已知边界

- 嵌套超过 6 层的目录会在报告中提示，不阻塞；
- `zstd` 由工作流通过 `apt-get` 安装（runner 不保证预装）；
- 工作流使用 Node 24 运行时（`actions/checkout@v7`、`setup-node@v7`、`upload-artifact@v7`、`download-artifact@v8`、`configure-pages@v6`、`upload-pages-artifact@v5`、`deploy-pages@v5`），不再触发 Node 20 弃用提示；
- 若之后调整目录规范，请同步修改 `scripts/sync-db.mjs` 的层级识别规则。

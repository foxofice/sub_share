// Multi-language support: Files/Languages/*.txt plus an embedded zh-CN table.
use std::collections::BTreeMap;
use std::path::Path;

pub const LANGUAGE_DIR: &str = "Files\\Languages";

/// One language: name (file stem) -> id -> text.
#[derive(Debug, Clone, Default)]
pub struct LangStore {
    pub languages: BTreeMap<String, BTreeMap<i32, String>>,
    pub current: String,
}

/// Embedded fallback so the exe works even without Files/Languages.
const BUILTIN_ZH: &[(i32, &str)] = &[
    (0, "服务器不支持范围请求或 Content-Length 不可用"),
    (1, "下载已被取消"),
    (10, "最后更新数据库的时间："),
    (11, "保存配置文件 {0:s}"),
    (20, "字幕路径"),
    (21, "最后更新数据库的时间"),
    (22, "窗口大小"),
    (23, "窗口位置"),
    (24, "数据网格"),
    (100, "关于"),
    (101, "更新日志："),
    (102, "官网"),
    (103, "确定"),
    (150, "日志"),
    (151, "时间"),
    (152, "日志"),
    (200, "高级查找"),
    (201, "番剧名称："),
    (202, "字幕名称："),
    (203, "字幕文件后缀："),
    (204, "提供者/字幕组："),
    (205, "描述："),
    (206, "类型："),
    (207, "片源："),
    (208, "放送日期："),
    (209, "重置搜索条件"),
    (210, "搜索"),
    (250, "是否重置搜索条件？"),
    (300, "更新数据库"),
    (301, "本地字幕数据 -> db.xml"),
    (302, "远程 db.xml -> db.xml"),
    (310, "本地扫描设置"),
    (311, "远程下载设置"),
    (320, "全部扫描"),
    (321, "部分扫描"),
    (322, "部分扫描设置"),
    (323, "类型"),
    (324, "年份"),
    (330, "下载线程数："),
    (350, "开始扫描"),
    (351, "停止扫描"),
    (352, "开始下载"),
    (353, "停止下载"),
    (360, "执行数据库更新……"),
    (361, "找不到本地字幕路径 {0:s}"),
    (362, "缺少 {0:s}"),
    (363, "用户停止更新数据库（耗时 {0:s}）"),
    (364, "更新数据库完成（扫描 {0:d} 条数据，全部 {1:d} 条数据，耗时 {2:s}）"),
    (365, "写入数据到文件 {0:s} 完成"),
    (366, "正在从远程服务器同步数据库（优先 db.json.zst）……"),
    (367, "下载 db.xml 完成"),
    (368, "下载 db.xml 失败（{0:s}）"),
    (400, "更新数据库"),
    (401, "打开本地文件夹"),
    (402, "打开远程链接"),
    (403, "高级查找"),
    (404, "日志"),
    (405, "语言(Languages)"),
    (406, "关于"),
    (420, "字幕路径："),
    (421, "根据名称查找"),
    (422, "使用查询语句查找"),
    (450, "本地没有任何数据，是否要下载最新的 db.xml？"),
    (451, "{0:d} 条记录"),
    (452, "{0:d} 部番剧"),
    (453, "是否要退出程序？"),
    (454, "找不到文件夹 {0:s}"),
    // Extra ids used only by this rewrite.
    (500, "搜索名称…"),
    (501, "搜索"),
    (502, "重置"),
    (503, "全部"),
    (504, "共 {0:d} 条记录 · {1:d} 部番剧"),
    (505, "上次更新：{0:s}"),
    (506, "双击行打开本地文件夹"),
    (507, "复制路径"),
    (508, "已复制到剪贴板"),
    (509, "类型"),
    (510, "片源"),
    (511, "在浏览器打开字幕目录…"),
    (512, "停止"),
    (513, "正在下载… {0:s}"),
    (514, "正在扫描… {0:s}"),
    (515, "已就绪"),
    (516, "年份（如 2001, 2008-2012, 2020）"),
    (517, "SubAtlas · 字幕库 —— foxofice/sub_db 的 Rust + GPUI 现代化重写版"),
    (518, "打开 GitHub 仓库"),
    (519, "服务器"),
    (520, "下载失败：{0:s}"),
    (521, "扫描失败：{0:s}"),
    (522, "字幕路径"),
    (523, "确认退出"),
    (524, "要退出 SubAtlas 吗？"),
    (525, "退出"),
    (526, "取消"),
    (527, "打开链接失败"),
    (560, "仪表盘"),
    (561, "字幕库"),
    (562, "快捷操作"),
    (563, "类型数"),
    (564, "记录总数"),
    (566, "链接"),
    (567, "在线字幕站"),
    (570, "导出 JSON+zstd"),
    (571, "已导出 {0:s}"),
    (572, "导出失败：{0:s}"),
    (573, "数据库已保存（JSON + zstd）"),
    (574, "远程不存在 db.json.zst，已回退下载 db.xml"),
    (575, "数据源"),
    (576, "本地扫描"),
    (577, "远程同步"),
    (578, "字幕库根目录"),
    (579, "优先下载 db.json.zst（JSON + zstd），失败自动回退 db.xml"),
    (580, "下载引擎"),
    (581, "ufetch-core · 自适应多连接 · 断点续传 · 自动重试"),
    (582, "更新完成后自动保存为 db.json.zst 与 db.xml"),
    (583, "开始同步"),
    (584, "相对路径以程序所在目录为基准，例如 ..\\subs_list"),
    (585, "更新完成"),
    (586, "更新失败"),
    (587, "扫描 {0:s} 个目录"),
    (588, "条记录"),
    (589, "扫描范围"),
    (590, "同步完成：{0:s}"),
    (591, "筛选中"),
    (565, "番剧数"),
    (530, "时间"),
    (531, "名称(简)"),
    (532, "名称(繁)"),
    (533, "名称(日)"),
    (534, "名称(英)"),
    (535, "罗马音"),
    (536, "类型"),
    (537, "片源"),
    (538, "字幕名称"),
    (539, "后缀"),
    (540, "字幕组"),
    (541, "描述"),
    (542, "路径"),
];

const BUILTIN_EN: &[(i32, &str)] = &[
    (0, "Server does not support range requests or Content-Length is missing"),
    (1, "Download cancelled"),
    (10, "Last database update time:"),
    (11, "Saved config file {0:s}"),
    (20, "Subs path"),
    (21, "Last update time"),
    (22, "Window size"),
    (23, "Window position"),
    (24, "Data grid"),
    (100, "About"),
    (101, "Changelog:"),
    (102, "Website"),
    (103, "OK"),
    (150, "Log"),
    (151, "Time"),
    (152, "Log"),
    (200, "Advanced Search"),
    (201, "Series name:"),
    (202, "Subtitle name:"),
    (203, "Subtitle extension:"),
    (204, "Providers:"),
    (205, "Description:"),
    (206, "Type:"),
    (207, "Source:"),
    (208, "Date:"),
    (209, "Reset search filters"),
    (210, "Search"),
    (250, "Reset search filters?"),
    (300, "Update Database"),
    (301, "Local data -> db.xml"),
    (302, "Remote db.xml -> db.xml"),
    (310, "Local scan settings"),
    (311, "Remote download settings"),
    (320, "Scan all"),
    (321, "Partial scan"),
    (322, "Partial scan settings"),
    (323, "Types"),
    (324, "Years"),
    (330, "Download threads:"),
    (350, "Start scan"),
    (351, "Stop scan"),
    (352, "Start download"),
    (353, "Stop download"),
    (360, "Updating database…"),
    (361, "Local subs path not found: {0:s}"),
    (362, "Missing {0:s}"),
    (363, "Update cancelled by user (took {0:s})"),
    (364, "Database updated (scanned {0:d} rows, {1:d} rows total, took {2:s})"),
    (365, "Wrote data to {0:s}"),
    (366, "Syncing database from remote (prefers db.json.zst)…"),
    (367, "Download db.xml finished"),
    (368, "Download db.xml failed ({0:s})"),
    (400, "Update DB"),
    (401, "Open Local Folder"),
    (402, "Open Remote Link"),
    (403, "Advanced Search"),
    (404, "Log"),
    (405, "Language"),
    (406, "About"),
    (420, "Subs path:"),
    (421, "Search by name"),
    (422, "Search by query"),
    (450, "No local data. Download the latest db.xml?"),
    (451, "{0:d} records"),
    (452, "{0:d} series"),
    (453, "Quit the program?"),
    (454, "Folder not found: {0:s}"),
    (500, "Search name…"),
    (501, "Search"),
    (502, "Reset"),
    (503, "All"),
    (504, "{0:d} records · {1:d} series"),
    (505, "Last update: {0:s}"),
    (506, "Double-click a row to open its folder"),
    (507, "Copy path"),
    (508, "Copied to clipboard"),
    (509, "Type"),
    (510, "Source"),
    (511, "Open subtitle folder on GitHub…"),
    (512, "Stop"),
    (513, "Downloading… {0:s}"),
    (514, "Scanning… {0:s}"),
    (515, "Ready"),
    (516, "Years (e.g. 2001, 2008-2012, 2020)"),
    (517, "SubAtlas — a modern Rust + GPUI rewrite of foxofice/sub_db"),
    (518, "Open GitHub repository"),
    (519, "Server"),
    (520, "Download failed: {0:s}"),
    (521, "Scan failed: {0:s}"),
    (522, "Subs path"),
    (523, "Confirm quit"),
    (524, "Quit SubAtlas?"),
    (525, "Quit"),
    (526, "Cancel"),
    (527, "Failed to open link"),
    (560, "Dashboard"),
    (561, "Database"),
    (562, "Quick actions"),
    (563, "Types"),
    (564, "Total records"),
    (566, "Links"),
    (567, "Subtitle site"),
    (570, "Export JSON+zstd"),
    (571, "Exported {0:s}"),
    (572, "Export failed: {0:s}"),
    (573, "Database saved (JSON + zstd)"),
    (574, "db.json.zst not available remotely, fell back to db.xml"),
    (575, "Source"),
    (576, "Local scan"),
    (577, "Remote sync"),
    (578, "Subtitle library root"),
    (579, "Prefers db.json.zst (JSON + zstd), falls back to db.xml"),
    (580, "Download engine"),
    (581, "ufetch-core · adaptive connections · resume · auto retry"),
    (582, "Saves both db.json.zst and db.xml after updating"),
    (583, "Start sync"),
    (584, "Relative paths are based on the program directory, e.g. ..\\subs_list"),
    (585, "Update complete"),
    (586, "Update failed"),
    (587, "scanned {0:s} folders"),
    (588, "rows"),
    (589, "Scan scope"),
    (590, "Synced: {0:s}"),
    (591, "Filtering"),
    (565, "Series"),
    (530, "Time"),
    (531, "Name (CHS)"),
    (532, "Name (CHT)"),
    (533, "Name (JP)"),
    (534, "Name (EN)"),
    (535, "Romaji"),
    (536, "Type"),
    (537, "Source"),
    (538, "Subtitle"),
    (539, "Ext"),
    (540, "Group"),
    (541, "Desc"),
    (542, "Path"),
];

fn builtin_table(entries: &[(i32, &str)]) -> BTreeMap<i32, String> {
    entries
        .iter()
        .map(|(k, v)| (*k, v.to_string()))
        .collect()
}

impl LangStore {
    pub fn new() -> Self {
        let mut this = Self {
            languages: BTreeMap::new(),
            current: String::new(),
        };
        this.languages.insert("zh-cn".into(), builtin_table(&BUILTIN_ZH));
        this.languages.insert("en-us".into(), builtin_table(&BUILTIN_EN));
        this.current = "zh-cn".into();
        this
    }

    /// Load extra language files from Files/Languages (same `id: text` format).
    pub fn load_dir(&mut self, dir: &Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("txt") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let mut table: BTreeMap<i32, String> = self
                .languages
                .get(&stem.to_lowercase())
                .cloned()
                .unwrap_or_default();
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let mut added = 0usize;
            for line in text.lines() {
                if line.len() < 2 || line.starts_with("//") {
                    continue;
                }
                let Some(idx) = line.find(':') else { continue };
                let Ok(id) = line[..idx].trim().parse::<i32>() else { continue };
                table.insert(id, line[idx + 1..].trim().to_string());
                added += 1;
            }
            // Files like changelog.txt carry no `id: text` entries; never
            // register them as a language.
            if added == 0 {
                continue;
            }
            self.languages.insert(stem.to_lowercase(), table);
        }
    }

    /// Pick the language matching `culture_name` (e.g. `zh-CN`), else en-US, else zh-CN.
    pub fn set_language(&mut self, culture_name: &str) {
        let want = culture_name.to_lowercase();
        self.current = if self.languages.contains_key(&want) {
            want
        } else if let Some((lang, _)) = want.split_once('-') {
            match self.languages.keys().find(|k| k.starts_with(lang)) {
                Some(k) => k.clone(),
                None => "zh-cn".into(),
            }
        } else {
            "zh-cn".into()
        };
    }

    pub fn language_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.languages.keys().cloned().collect();
        // Chinese first, then English, everything else follows.
        names.sort_by_key(|n| match n.as_str() {
            "zh-cn" => 0,
            "en-us" => 1,
            _ => 2,
        });
        names
    }

    pub fn txt(&self, id: i32) -> String {
        self.languages
            .get(&self.current)
            .and_then(|t| t.get(&id))
            .cloned()
            .unwrap_or_else(|| {
                self.languages
                    .get("zh-cn")
                    .and_then(|t| t.get(&id))
                    .cloned()
                    .unwrap_or_else(|| "(no msg)".into())
            })
    }
}

/// Replace `{0:s}` / `{1:d}` style placeholders (matches the original's
/// `string.Format` usage). Values are passed in positional order.
pub fn fmt(template: &str, args: &[String]) -> String {
    let mut out = template.to_string();
    for (i, arg) in args.iter().enumerate() {
        let mut repl = |tag: &str| {
            let from = format!("{{{i}:{tag}}}");
            while let Some(pos) = out.find(&from) {
                out.replace_range(pos..pos + from.len(), arg);
            }
        };
        repl("s");
        repl("d");
    }
    out
}

pub fn fmt1(template: &str, a: impl Into<String>) -> String {
    fmt(template, &[a.into()])
}

pub fn fmt2(template: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    fmt(template, &[a.into(), b.into()])
}

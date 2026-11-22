// Page views rendered inside the main shell (single-window, FlClash style).
use crate::app::{self, Globals, LogLevel};
use crate::dl::{DownloadStatus, Downloader};
use crate::search::Filters;
use gpui::{
    div, px, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _, Window,
};
use gpui_component::{
    button::Button,
    checkbox::Checkbox,
    input::{Input, InputState},
    radio::{Radio, RadioGroup},
    ActiveTheme as _, button::ButtonVariants as _, Icon, Sizable as _, StyledExt as _, h_flex,
    v_flex,
};
use gpui_kit::assets::IconName;
use gpui::prelude::FluentBuilder as _;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Called when the database changed in the update page: ask the shell to
/// reload itself during its next render (where a Window is available).
fn notify_main_reload(cx: &mut gpui::App) {
    let mv = Globals::global(cx).main_view.clone();
    if let Some(m) = mv.upgrade() {
        m.update(cx, |m, cx| {
            m.mark_pending_reload();
            cx.notify();
        });
    }
}


// ---------------------------------------------------------------------------
// Log window
// ---------------------------------------------------------------------------


fn level_color(cx: &gpui::App, level: LogLevel) -> gpui::Hsla {
    let theme = cx.theme();
    match level {
        LogLevel::Info => theme.muted_foreground,
        LogLevel::Success => theme.success,
        LogLevel::Warning => theme.warning,
        LogLevel::Error => theme.danger,
    }
}

/// Log page (stateless; re-rendered when the shell is notified).
pub fn log_page(cx: &gpui::App) -> gpui::AnyElement {
    let g = Globals::global(cx);
    let entries = g.log.read(cx).entries.clone();
    let theme_fg = cx.theme().muted_foreground;

    v_flex()
        .size_full()
        .bg(cx.theme().background)
        .text_color(cx.theme().foreground)
        .child(
            h_flex()
                .px_2()
                .py_1()
                .gap_2()
                .border_b_1()
                .border_color(cx.theme().border)
                .text_size(px(12.))
                .text_color(theme_fg)
                .child(div().w(px(140.)).child(app::t(cx, 151)))
                .child(div().child(app::t(cx, 152))),
        )
        .child(
            div()
                .id("log-list")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .px_2()
                .py_1()
                .children(entries.into_iter().rev().map(|e| {
                    h_flex()
                        .gap_2()
                        .text_size(px(12.))
                        .py(px(1.))
                        .child(
                            div()
                                .w(px(140.))
                                .flex_shrink_0()
                                .text_color(theme_fg)
                                .child(SharedString::from(
                                    e.time.format("%Y-%m-%d %H:%M:%S").to_string(),
                                )),
                        )
                        .child(
                            div()
                                .text_color(level_color(cx, e.level))
                                .child(SharedString::from(e.text)),
                        )
                })),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// About window
// ---------------------------------------------------------------------------

pub const VERSION: &str = "v0.24 · SubAtlas";

/// Changelog for this Rust + GPUI rewrite (shown above the upstream log).
pub const REWRITE_CHANGELOG: &str = "v0.24 · SubAtlas（Rust + GPUI） - 2026.11.22\n\
\t[ ! ] 使用 Rust + GPUI 完全重写，单文件 exe\n\
\t[ ! ] 全新现代化界面：FlClash 风格侧边栏 + 卡片式页面\n\
\t[ + ] 深色 / 浅色主题一键切换（默认浅色）\n\
\t[ + ] 界面动效：页面切换过渡、侧栏选中动画、卡片渐入\n\
\t[ + ] 仪表盘：记录总数 / 番剧数 / 类型数 / 上次更新 + 快捷操作\n\
\t[ + ] 在线字幕站入口：https://foxofice.github.io/sub_share/\n\
\t[ * ] 名称搜索：简繁自动转换匹配（保留原版逻辑）\n\
\t[ * ] 更新数据库：多线程分段下载 / 本地扫描 / 部分扫描（类型 + 年份）\n\
\t[ * ] 高级查找：番剧名 / 字幕名 / 后缀 / 片源 / 字幕组 / 描述 / 放送日期\n\
\t[ * ] 日志页、关于页改为内嵌页面\n\
\t[ - ] 移除“SQL 查询语句”搜索模式，请使用高级查找字段\n\
\t[ * ] 快捷键：F5 更新数据库 / F3 高级查找 / F4 日志\n\
\t[ + ] 数据库新增 JSON + zstd 压缩格式（优先加载，体积更小）\n\
\t[ + ] 数据库页支持“导出 JSON+zstd”\n\
\t[ * ] 远程更新优先下载 db.json.zst，不可用时自动回退 db.xml\n\
\t[ * ] 远程同步接入 ufetch-core 下载引擎（自适应多连接 / 断点续传 / 自动重试）\n\
\t[ ! ] 语言菜单仅显示真实语言，默认优先中文\n\
\t[ ! ] 关于页/语言/更新日志全部内嵌，除数据库外无外部文件依赖\n\n";

pub const SITE_URL: &str = "https://foxofice.github.io/sub_share/";

/// Export the current database as JSON + zstd via a native save dialog.
pub fn export_json_zstd(cx: &mut gpui::App) {
    let subs = Globals::global(cx).db.0.read().unwrap().subs.clone();
    if subs.is_empty() {
        return;
    }
    let Some(path) = rfd::FileDialog::new()
        .set_title("导出 JSON + zstd")
        .set_file_name("db.json.zst")
        .add_filter("zstd compressed json", &["zst"])
        .save_file()
    else {
        return;
    };
    match crate::db::write_db_json_zst(&path, &subs) {
        Ok(()) => {
            let msg = crate::i18n::fmt1(&app::t(cx, 571), path.display().to_string());
            app::log(cx, msg, LogLevel::Success);
        }
        Err(e) => {
            let msg = crate::i18n::fmt1(&app::t(cx, 572), e.to_string());
            app::log(cx, msg, LogLevel::Error);
        }
    }
}

/// A FlClash-style link row: tinted icon tile + title + subtitle + chevron.
fn link_row(
    cx: &gpui::App,
    id: &'static str,
    icon: IconName,
    title: String,
    sub: String,
    url: &'static str,
    dark: bool,
) -> gpui::AnyElement {
    let sf = crate::theme::surfaces(dark);
    h_flex()
        .id(id)
        .h(px(56.))
        .px_4()
        .gap_3()
        .items_center()
        .cursor_pointer()
        .hover(|s| s.bg(gpui::rgb(sf.hover)))
        .on_click(move |_, _, _| {
            let _ = open::that(url);
        })
        .child(
            div()
                .size(px(34.))
                .rounded(px(10.))
                .flex()
                .items_center()
                .justify_center()
                .bg(cx.theme().accent)
                .text_color(cx.theme().accent_foreground)
                .child(Icon::new(icon).size(px(17.))),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap(px(1.))
                .child(
                    div()
                        .text_size(px(13.))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .child(SharedString::from(title)),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(cx.theme().muted_foreground)
                        .child(SharedString::from(sub)),
                ),
        )
        .child(
            Icon::new(IconName::ExternalLink)
                .size(px(15.))
                .text_color(gpui::rgb(sf.chevron)),
        )
        .into_any_element()
}

fn section_label(cx: &gpui::App, text: String) -> impl IntoElement {
    div()
        .mt_2()
        .mb_1()
        .text_size(px(12.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(cx.theme().muted_foreground)
        .child(SharedString::from(text))
}

fn cards(cx: &gpui::App) -> gpui::Div {
    let dark = cx.theme().mode.is_dark();
    let sf = crate::theme::surfaces(dark);
    div()
        .rounded(px(16.))
        .bg(cx.theme().popover)
        .border_1()
        .border_color(gpui::rgb(sf.card_border))
        .shadow(vec![gpui::BoxShadow {
            color: gpui::hsla(0., 0., 0., if dark { 0.35 } else { 0.05 }),
            offset: gpui::point(px(0.), px(6.)),
            blur_radius: px(20.),
            spread_radius: px(-6.),
            inset: false,
        }])
        .overflow_hidden()
}

/// About page (stateless), FlClash-style sections.
pub fn about_page(cx: &gpui::App) -> gpui::AnyElement {
    let dark = cx.theme().mode.is_dark();
    let sf = crate::theme::surfaces(dark);
    // Fully embedded: the binary needs no external text files at all.
    const UPSTREAM_CHANGELOG: &str = include_str!("../assets/upstream_changelog.txt");
    let changelog = if UPSTREAM_CHANGELOG.contains("v0.24 (Rust+GPUI)") {
        UPSTREAM_CHANGELOG.to_string()
    } else {
        format!("{REWRITE_CHANGELOG}{UPSTREAM_CHANGELOG}")
    };

    v_flex()
        .gap_3()
        .text_color(cx.theme().foreground)
        // ---- Hero card -------------------------------------------------
        .child(
            cards(cx)
                .p_6()
                .child(
                    h_flex()
                        .gap_4()
                        .items_center()
                        .child(
                            div()
                                .size(px(64.))
                                .rounded(px(18.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(cx.theme().primary)
                                .text_color(cx.theme().primary_foreground)
                                .child(Icon::new(IconName::Database).size(px(30.))),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_2()
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .items_center()
                                        .child(
                                            div()
                                                .text_size(px(22.))
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .child(SharedString::from(
                                                    "SubAtlas",
                                                )),
                                        )
                                        .child(
                                            div()
                                                .px_2()
                                                .py(px(2.))
                                                .rounded_full()
                                                .bg(cx.theme().accent)
                                                .text_size(px(11.))
                                                .text_color(cx.theme().accent_foreground)
                                                .child(SharedString::from(VERSION)),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .text_color(cx.theme().muted_foreground)
                                        .child(SharedString::from(app::t(cx, 517))),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(cx.theme().muted_foreground)
                                        .child(SharedString::from(
                                            "下载引擎 ufetch-core · LGPL-3.0-or-later · github.com/LianXia233/ufetch-core",
                                        )),
                                ),
                        ),
                ),
        )
        // ---- Links -----------------------------------------------------
        .child(section_label(cx, app::t(cx, 566)))
        .child(
            cards(cx)
                .child(link_row(
                    cx,
                    "about-site-online",
                    IconName::Globe,
                    app::t(cx, 567),
                    "foxofice.github.io/sub_share".into(),
                    SITE_URL,
                    dark,
                ))
                .child(div().h(px(1.)).bg(cx.theme().border.opacity(0.6)))
                .child(link_row(
                    cx,
                    "about-share",
                    IconName::Github,
                    "sub_share".to_string(),
                    "github.com/foxofice/sub_share".into(),
                    "https://github.com/foxofice/sub_share",
                    dark,
                ))
                .child(div().h(px(1.)).bg(cx.theme().border.opacity(0.6)))
                .child(link_row(
                    cx,
                    "about-repo",
                    IconName::Github,
                    "sub_db".to_string(),
                    "github.com/foxofice/sub_db".into(),
                    "https://github.com/foxofice/sub_db",
                    dark,
                ))
                .child(div().h(px(1.)).bg(cx.theme().border.opacity(0.6)))
                .child(link_row(
                    cx,
                    "about-web",
                    IconName::Link,
                    app::t(cx, 102),
                    "www.acgdev.com".into(),
                    "https://www.acgdev.com",
                    dark,
                )),
        )
        // ---- Changelog -------------------------------------------------
        .child(section_label(cx, app::t(cx, 101)))
        .child(
            cards(cx).p_4().child(
                div()
                    .text_size(px(12.))
                    .text_color(gpui::rgb(sf.sidebar_text))
                    .child(SharedString::from(changelog)),
            ),
        )
        .into_any_element()
}

const TIME_OPS: [&str; 5] = ["=", "<", "<=", ">", ">="];

pub struct SearchView {
    name: Entity<InputState>,
    sub_name: Entity<InputState>,
    extension: Entity<InputState>,
    providers: Entity<InputState>,
    desc: Entity<InputState>,
    source: Entity<InputState>,
    date: Entity<InputState>,
    time_op_ix: usize,
    /// Reset requested from a click; applied during render (needs a Window).
    pending_reset: bool,
}

impl SearchView {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let lang_entity = Globals::global(cx).lang.clone();
        cx.observe(&lang_entity, |_, _, cx| cx.notify())
            .detach();
        let cur = Globals::global(cx).filters.read(cx).filters.clone();

        let name = cx.new(|cx| InputState::new(window, cx));
        name.update(cx, |s, cx| s.set_value(cur.name.clone(), window, cx));
        let sub_name = cx.new(|cx| InputState::new(window, cx));
        sub_name
            .update(cx, |s, cx| s.set_value(cur.sub_name.clone(), window, cx));
        let extension = cx.new(|cx| InputState::new(window, cx));
        extension
            .update(cx, |s, cx| s.set_value(cur.extension.clone(), window, cx));
        let providers = cx.new(|cx| InputState::new(window, cx));
        providers
            .update(cx, |s, cx| s.set_value(cur.providers.clone(), window, cx));
        let desc = cx.new(|cx| InputState::new(window, cx));
        desc.update(cx, |s, cx| s.set_value(cur.desc.clone(), window, cx));
        let source = cx.new(|cx| InputState::new(window, cx));
        source.update(cx, |s, cx| s.set_value(cur.source.clone(), window, cx));
        let date = cx.new(|cx| InputState::new(window, cx));
        date.update(cx, |s, cx| {
            s.set_value(
                cur.time_op
                    .as_ref()
                    .map(|(_, d)| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default(),
                window,
                cx,
            )
        });

        let time_op_ix = cur
            .time_op
            .as_ref()
            .map(|(op, _)| TIME_OPS.iter().position(|o| o == op).unwrap_or(0))
            .unwrap_or(0);

        Self {
            name,
            sub_name,
            extension,
            providers,
            desc,
            source,
            date,
            time_op_ix,
            pending_reset: false,
        }
    }

    fn field(
        &self,
        label: String,
        input: &Entity<InputState>,
    ) -> impl IntoElement {
        h_flex()
            .gap_2()
            .items_center()
            .child(
                div()
                    .w(px(120.))
                    .text_size(px(13.))
                    .child(SharedString::from(label)),
            )
            .child(
                div()
                    .flex_1()
                    .child(Input::new(input).with_size(gpui_component::Size::Small)),
            )
    }

    fn collect(&self, cx: &Context<Self>) -> Filters {
        let mut f = Filters::default();
        f.name = self.name.read(cx).value().trim().to_string();
        f.sub_name = self.sub_name.read(cx).value().trim().to_string();
        f.extension = self.extension.read(cx).value().trim().to_string();
        f.providers = self.providers.read(cx).value().trim().to_string();
        f.desc = self.desc.read(cx).value().trim().to_string();
        f.source = self.source.read(cx).value().trim().to_string();
        let date_text = self.date.read(cx).value().trim().to_string();
        let op = TIME_OPS[self.time_op_ix].to_string();
        if !date_text.is_empty() {
            if let Ok(d) = chrono::NaiveDate::parse_from_str(&date_text, "%Y-%m-%d") {
                f.time_op = Some((op, d));
            }
        }
        f
    }
}

impl Render for SearchView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.pending_reset {
            self.pending_reset = false;
            for f in [
                &self.name,
                &self.sub_name,
                &self.extension,
                &self.providers,
                &self.desc,
                &self.source,
                &self.date,
            ] {
                f.update(cx, |s, cx| s.set_value("", window, cx));
            }
            self.time_op_ix = 0;
            let store = Globals::global(cx).filters.clone();
            store.update(cx, |store, cx| store.set(Filters::default(), cx));
        }
        let this = cx.entity().downgrade();
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .p_4()
            .gap_2()
            .child(self.field(app::t(cx, 201), &self.name))
            .child(self.field(app::t(cx, 202), &self.sub_name))
            .child(self.field(app::t(cx, 203), &self.extension))
            .child(self.field(app::t(cx, 207), &self.source))
            .child(self.field(app::t(cx, 204), &self.providers))
            .child(self.field(app::t(cx, 205), &self.desc))
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .w(px(120.))
                            .text_size(px(13.))
                            .child(SharedString::from(app::t(cx, 208))),
                    )
                    .child(
                        div().child(
                            RadioGroup::horizontal("time-op")
                                .selected_index(Some(self.time_op_ix))
                                .on_click({
                                    let this = this.clone();
                                    move |ix, _, cx| {
                                        let _ = this.update(cx, |v, cx| {
                                            v.time_op_ix = *ix;
                                            cx.notify();
                                        });
                                    }
                                })
                                .child(Radio::new("op0").label("="))
                                .child(Radio::new("op1").label("<"))
                                .child(Radio::new("op2").label("<="))
                                .child(Radio::new("op3").label(">"))
                                .child(Radio::new("op4").label(">=")),
                        ),
                    )
                    .child(
                        div()
                            .w(px(180.))
                            .child(Input::new(&self.date).with_size(gpui_component::Size::Small)),
                    ),
            )
            .child(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("search-reset")
                            .label(app::t(cx, 209))
                            .with_size(gpui_component::Size::Small)
                            .on_click({
                                let this = this.clone();
                                move |_, _, cx| {
                                    let _ = this.update(cx, |v, cx| {
                                        v.pending_reset = true;
                                        cx.notify();
                                    });
                                }
                            }),
                    )
                    .child(
                        Button::new("search-go")
                            .label(app::t(cx, 210))
                            .icon(Icon::new(IconName::Search))
                            .primary()
                            .with_size(gpui_component::Size::Small)
                            .on_click({
                                let this = this.clone();
                                move |_, _, cx| {
                                    let _ = this.update(cx, |v, cx| {
                                        let filters = v.collect(cx);
                                        let store = Globals::global(cx).filters.clone();
                                        store.update(cx, |store, cx| store.set(filters, cx));
                                    });
                                }
                            }),
                    ),
            )
    }
}

// ---------------------------------------------------------------------------
// Update database window
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct JobState {
    running: bool,
    is_scan: bool,
    text: String,
    progress: Option<f32>,
    /// Which file the sync actually used, e.g. `db.json.zst · 229 KB`.
    done_note: Option<String>,
    /// Set by the worker when the job ends: Ok(summary lines) / Err(message).
    result: Option<Result<(usize, Vec<String>, f64), String>>,
}

pub struct UpdateView {
    mode_scan: bool,
    scan_all: bool,
    types: Arc<Mutex<Vec<(String, bool)>>>,
    years: Entity<InputState>,
    subs_path: Entity<InputState>,
    job: Arc<Mutex<JobState>>,
    stop: Arc<AtomicBool>,
    downloader: Arc<Downloader>,
    scan_progress: Arc<Mutex<crate::scan::ScanProgress>>,
    running: Arc<AtomicBool>,
    /// Result banner shown after a job finishes: Ok(detail) / Err(message).
    last: Option<Result<String, String>>,
}

impl UpdateView {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let lang_entity = Globals::global(cx).lang.clone();
        cx.observe(&lang_entity, |_, _, cx| cx.notify())
            .detach();

        let cfg = Globals::global(cx).config.lock().unwrap().clone();
        let subs_path = cx.new(|cx| InputState::new(window, cx));
        subs_path.update(cx, |s, cx| s.set_value(cfg.subs_path.clone(), window, cx));
        let years = cx.new(|cx| {
            InputState::new(window, cx).placeholder(app::t(cx, 516))
        });

        let types: Vec<(String, bool)> = crate::scan::local_types(&cfg.subs_path)
            .into_iter()
            .map(|t| (t, false))
            .collect();

        Self {
            mode_scan: true,
            scan_all: true,
            types: Arc::new(Mutex::new(types)),
            years,
            subs_path,
            job: Arc::new(Mutex::new(JobState {
                running: false,
                is_scan: true,
                text: String::new(),
                progress: None,
                done_note: None,
                result: None,
            })),
            stop: Arc::new(AtomicBool::new(false)),
            downloader: Arc::new(Downloader::new()),
            scan_progress: Arc::new(Mutex::new(Default::default())),
            running: Arc::new(AtomicBool::new(false)),
            last: None,
        }
    }

    fn save_subs_path(&self, cx: &Context<Self>) {
        let path = self.subs_path.read(cx).value().trim().to_string();
        let g = Globals::global(cx);
        let mut cfg = g.config.lock().unwrap().clone();
        cfg.subs_path = path;
        *g.config.lock().unwrap() = cfg.clone();
        let lang = g.lang.read(cx).clone();
        let _ = crate::config::save_config(&app::config_path(), &cfg, &lang);
    }

    fn parse_years(text: &str) -> Option<Vec<i32>> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let mut years = Vec::new();
        for part in text.split([',', ';', ' ']) {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            if let Some((a, b)) = part.split_once('-') {
                if let (Ok(a), Ok(b)) = (a.trim().parse::<i32>(), b.trim().parse::<i32>()) {
                    for y in a..=b {
                        years.push(y);
                    }
                }
            } else if let Ok(y) = part.parse::<i32>() {
                years.push(y);
            }
        }
        if years.is_empty() {
            None
        } else {
            Some(years)
        }
    }

    /// Start a remote db download (used when no local database exists).
    pub(crate) fn start_remote_download(&mut self, cx: &mut Context<Self>) {
        // First-run auto download: show the fresh data when it lands.
        self.mode_scan = false;
        self.start_job_ex(cx, true);
    }

    fn start_job(&mut self, cx: &mut Context<Self>) {
        self.start_job_ex(cx, false);
    }

    fn start_job_ex(&mut self, cx: &mut Context<Self>, jump: bool) {
        if self.running.load(Ordering::Relaxed) {
            return;
        }
        Globals::global(cx)
            .jump_after_reload
            .store(jump, Ordering::Relaxed);
        self.save_subs_path(cx);
        self.last = None;

        // No local subtitle tree? Fetch the remote database instead.
        if self.mode_scan {
            let path = self.subs_path.read(cx).value().trim().to_string();
            if !std::path::Path::new(&path).is_dir() {
                let msg = crate::i18n::fmt1(&app::t(cx, 361), path.clone());
                app::log(cx, msg, LogLevel::Warning);
                let msg2 = app::t(cx, 574);
                app::log(cx, msg2, LogLevel::Info);
                self.mode_scan = false;
                cx.notify();
                return self.start_job(cx);
            }
        }
        self.stop.store(false, Ordering::Relaxed);
        self.running.store(true, Ordering::Relaxed);

        let lang_ids = if self.mode_scan { (360, 350) } else { (366, 352) };
        let msg = app::t(cx, lang_ids.0);
        app::log(cx, msg, LogLevel::Info);

        if self.mode_scan {
            // Local scan -----------------------------------------------------
            let subs_path = self.subs_path.read(cx).value().trim().to_string();
            let selected_types: Option<Vec<String>> = if self.scan_all {
                None
            } else {
                let sel: Vec<String> = self
                    .types
                    .lock().unwrap()
                    .iter()
                    .filter(|(_, on)| *on)
                    .map(|(t, _)| t.clone())
                    .collect();
                Some(sel)
            };
            let years = if self.scan_all {
                None
            } else {
                Self::parse_years(&self.years.read(cx).value())
            };
            let job = self.job.clone();
            let stop = self.stop.clone();
            let progress = self.scan_progress.clone();
            let db_path = app::db_xml_path();
            let offset = app::local_offset();

            *job.lock().unwrap() = JobState {
                running: true,
                is_scan: true,
                text: app::t(cx, 360),
                progress: Some(0.),
                done_note: None,
                result: None,
            };

            std::thread::spawn(move || {
                let mut warnings = Vec::new();
                let result = crate::scan::run_scan(
                    &crate::scan::ScanParams {
                        subs_path,
                        types: selected_types,
                        years,
                    },
                    &stop,
                    &progress,
                    |msg| warnings.push(msg),
                );
                let mut st = job.lock().unwrap();
                match result {
                    Ok(r) => {
                        match crate::scan::write_scan_result(r.subs, &db_path, offset) {
                            Ok(_) => {
                                st.result = Some(Ok((r.total, warnings, r.elapsed.as_secs_f64())));
                            }
                            Err(e) => {
                                st.result = Some(Err(format!("write db.xml: {e}")));
                            }
                        }
                    }
                    Err(e) => {
                        st.result = Some(Err(e.to_string()));
                    }
                }
                st.running = false;
            });
        } else {
            // Remote download ------------------------------------------------
            let job = self.job.clone();
            let downloader = self.downloader.clone();
            let dest = app::db_xml_path();

            *job.lock().unwrap() = JobState {
                running: true,
                is_scan: false,
                text: app::t(cx, 366),
                progress: Some(0.),
                done_note: None,
                result: None,
            };

            let json_dest = app::db_json_path();
            std::thread::spawn(move || {
                // Prefer the compressed JSON + zstd file; fall back to XML.
                let mut result =
                    downloader.download(crate::dl::DB_JSON_URL, &json_dest, None);
                let mut used = Some("db.json.zst");
                if result.is_err() {
                    used = None;
                    result = downloader.download(crate::dl::DB_URL, &dest, None);
                    if result.is_ok() {
                        used = Some("db.xml");
                    }
                }
                let mut st = job.lock().unwrap();
                st.running = false;
                match result {
                    Ok(()) => {
                        let file = used.unwrap_or("db.json.zst");
                        let path = if file == "db.xml" { &dest } else { &json_dest };
                        let bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                        st.done_note = Some(format!("{file} · {}", UpdateView::human(bytes)));
                        st.result = Some(Ok((0, Vec::new(), 0.)));
                    }
                    Err(e) => st.result = Some(Err(e)),
                }
            });
        }

        // Poll loop --------------------------------------------------------
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(150))
                    .await;
                let mut done = false;
                if this
                    .update(cx, |v, cx| {
                        v.poll(cx);
                        done = !v.job.lock().unwrap().running;
                    })
                    .is_err()
                {
                    break;
                }
                if done {
                    break;
                }
            }
        })
        .detach();
    }

    /// Runs on the main thread while a job is active.
    fn poll(&mut self, cx: &mut Context<Self>) {
        let has_result;
        let running;
        {
            let job = self.job.lock().unwrap();
            has_result = job.result.is_some();
            running = job.running;
        }
        if running || !has_result {
            // Update progress display
            if self.mode_scan {
                let p = self.scan_progress.lock().unwrap().clone();
                let pct = if p.total > 0 {
                    p.done as f32 / p.total as f32
                } else {
                    0.
                };
                let mut job = self.job.lock().unwrap();
                job.progress = Some(pct);
                job.text = format!("{}/{}", p.done, p.total);
            } else {
                let st = self.downloader.status();
                if let DownloadStatus::Progress { received, total } = st {
                    let pct = if total > 0 {
                        received as f32 / total as f32
                    } else {
                        0.
                    };
                    let mut job = self.job.lock().unwrap();
                    job.progress = Some(pct);
                    job.text = if total > 0 {
                        format!("{} / {}", Self::human(received), Self::human(total))
                    } else {
                        Self::human(received)
                    };
                }
            }
            cx.notify();
            return;
        }

        // Job finished ------------------------------------------------------
        self.running.store(false, Ordering::Relaxed);
        let result = self.job.lock().unwrap().result.take();
        match result {
            Some(Ok((scanned, warnings, elapsed))) => {
                let secs = format!("{:.1}s", elapsed);
                if self.mode_scan {
                    for w in &warnings {
                        app::log(cx, w.clone(), LogLevel::Warning);
                    }
                }
                let note = self.job.lock().unwrap().done_note.clone();
                let rows = self.finish_reload(cx, secs.clone());
                let detail = if self.mode_scan {
                    let scanned_txt = crate::i18n::fmt1(&app::t(cx, 587), scanned.to_string());
                    format!("{scanned_txt} · {rows} {} · {secs}", app::t(cx, 588))
                } else {
                    let file = note.clone().unwrap_or_else(|| "db.json.zst".to_string());
                    format!("{rows} {} · {file} · {secs}", app::t(cx, 588))
                };
                if !self.mode_scan {
                    let msg = crate::i18n::fmt1(&app::t(cx, 590), note.unwrap_or_default());
                    app::log(cx, msg, LogLevel::Success);
                }
                self.last = Some(Ok(detail));
            }
            Some(Err(e)) => {
                let msg = crate::i18n::fmt1(&app::t(cx, 368), e.clone());
                app::log(cx, msg, LogLevel::Error);
                self.last = Some(Err(e));
            }
            None => {}
        }
        cx.notify();
    }

    fn finish_reload(&mut self, cx: &mut Context<Self>, secs: String) -> usize {
        let (db, config, lang) = {
            let g = Globals::global(cx);
            let lang = g.lang.read(cx).clone();
            (g.db.clone(), g.config.clone(), lang)
        };
        let mut store = db.0.write().unwrap();
        match store.load_from_file(&app::db_xml_path()) {
            Ok((n, _src)) => {
                // Keep both formats in sync (JSON+zstd primary, XML legacy).
                let offset = app::local_offset();
                let _ = store.save_to_file(&app::db_xml_path(), offset);
                drop(store);
                let mut cfg = config.lock().unwrap().clone();
                cfg.last_update_time = Some(chrono::Local::now().naive_local());
                let _ = crate::config::save_config(&app::config_path(), &cfg, &lang);
                *config.lock().unwrap() = cfg;
                if self.mode_scan {
                    let msg = crate::i18n::fmt(
                        &app::t(cx, 364),
                        &[n.to_string(), n.to_string(), secs],
                    );
                    app::log(cx, msg, LogLevel::Success);
                    let msg2 = crate::i18n::fmt1(
                        &app::t(cx, 365),
                        app::db_xml_path().display().to_string(),
                    );
                    app::log(cx, msg2, LogLevel::Success);
                }
                notify_main_reload(cx);
                return n;
            }
            Err(e) => {
                app::log(cx, format!("reload db.xml: {e}"), LogLevel::Error);
            }
        }
        0
    }

    fn stop_job(&mut self, cx: &mut Context<Self>) {
        self.stop.store(true, Ordering::Relaxed);
        self.downloader.cancel();
        let msg = app::t(cx, 1);
        app::log(cx, msg, LogLevel::Warning);
    }

    fn human(n: u64) -> String {
        if n >= 1024 * 1024 {
            format!("{:.1} MB", n as f64 / (1024.0 * 1024.0))
        } else if n >= 1024 {
            format!("{:.0} KB", n as f64 / 1024.0)
        } else {
            format!("{n} B")
        }
    }

    /// A small muted section label.
    fn label(cx: &gpui::App, id: i32) -> impl IntoElement {
        div()
            .text_size(px(12.))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(cx.theme().muted_foreground)
            .child(SharedString::from(app::t(cx, id)))
    }

    /// FlClash-style segmented control (pill container, filled selection).
    fn segmented(
        cx: &gpui::App,
        items: Vec<(bool, String)>,
        selected: bool,
        kind: u8,
        view: gpui::WeakEntity<UpdateView>,
    ) -> impl IntoElement {
        h_flex()
            .gap_1()
            .p(px(3.))
            .rounded_full()
            .bg(cx.theme().muted)
            .children(items.into_iter().map(move |(value, text)| {
                let on = value == selected;
                let view = view.clone();
                div()
                    .id(format!("seg-{kind}-{}", value as u8))
                    .px_4()
                    .py(px(5.))
                    .rounded_full()
                    .cursor_pointer()
                    .bg(if on {
                        cx.theme().primary
                    } else {
                        gpui::rgba(0x00000000).into()
                    })
                    .text_size(px(13.))
                    .text_color(if on {
                        cx.theme().primary_foreground
                    } else {
                        cx.theme().muted_foreground
                    })
                    .on_click(move |_, _, cx| {
                        let _ = view.update(cx, |v, cx| {
                            if kind == 0 {
                                v.mode_scan = value;
                            } else {
                                v.scan_all = value;
                            }
                            cx.notify();
                        });
                    })
                    .child(SharedString::from(text))
            }))
    }

    fn render_local(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let scan_all = self.scan_all;
        let types = self.types.lock().unwrap().clone();
        let dark = cx.theme().mode.is_dark();
        let sf = crate::theme::surfaces(dark);

        v_flex()
            .gap_2()
            .child(Self::label(cx, 578))
            .child(
                Input::new(&self.subs_path)
                    .w_full()
                    .with_size(gpui_component::Size::Small),
            )
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(cx.theme().muted_foreground)
                    .child(SharedString::from(app::t(cx, 584))),
            )
            .child(div().h(px(4.)))
            .child(Self::label(cx, 589))
            .child(Self::segmented(
                cx,
                vec![
                    (true, app::t(cx, 320)),
                    (false, app::t(cx, 321)),
                ],
                scan_all,
                1,
                view.clone(),
            ))
            .when_some(if scan_all { None } else { Some(()) }, |el, _| {
                el.child(
                    v_flex()
                        .mt_1()
                        .p_3()
                        .gap_2()
                        .rounded(px(12.))
                        .bg(gpui::rgb(sf.hover))
                        .child(Self::label(cx, 323))
                        .child(
                            div().flex_wrap().gap_2().children(
                                types.iter().enumerate().map(|(i, (name, on))| {
                                    let view = view.clone();
                                    let name = name.clone();
                                    Checkbox::new(format!("type-{i}"))
                                        .label(name)
                                        .checked(*on)
                                        .on_click(move |checked, _, cx| {
                                            let _ = view.update(cx, |v, cx| {
                                                if let Some(entry) =
                                                    v.types.lock().unwrap().get_mut(i)
                                                {
                                                    entry.1 = *checked;
                                                }
                                                cx.notify();
                                            });
                                        })
                                }),
                            ),
                        )
                        .child(Self::label(cx, 324))
                        .child(
                            Input::new(&self.years)
                                .w_full()
                                .with_size(gpui_component::Size::Small),
                        ),
                )
            })
    }

    fn render_remote(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let dark = cx.theme().mode.is_dark();
        let sf = crate::theme::surfaces(dark);
        let row = |icon: IconName, title: String, sub: String| {
            h_flex()
                .gap_3()
                .items_center()
                .child(
                    div()
                        .size(px(30.))
                        .rounded(px(9.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(cx.theme().accent)
                        .text_color(cx.theme().accent_foreground)
                        .child(Icon::new(icon).size(px(15.))),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .gap(px(1.))
                        .child(
                            div()
                                .text_size(px(13.))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(SharedString::from(title)),
                        )
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(cx.theme().muted_foreground)
                                .child(SharedString::from(sub)),
                        ),
                )
        };

        v_flex()
            .gap_2()
            .child(Self::label(cx, 575))
            .child(
                v_flex()
                    .p_3()
                    .gap_3()
                    .rounded(px(12.))
                    .bg(gpui::rgb(sf.hover))
                    .child(row(
                        IconName::Github,
                        "GitHub · foxofice/sub_share".to_string(),
                        app::t(cx, 579),
                    ))
                    .child(row(
                        IconName::Download,
                        app::t(cx, 580),
                        app::t(cx, 581),
                    )),
            )
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(cx.theme().muted_foreground)
                    .child(SharedString::from(app::t(cx, 582))),
            )
    }

    fn render_progress_panel(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let job = self.job.lock().unwrap().clone();
        let running = self.running.load(Ordering::Relaxed) || job.running;

        // Finished: show a clear result banner instead of the raw bytes line.
        if !running {
            if let Some(last) = &self.last {
                return match last {
                    Ok(detail) => v_flex()
                        .mt_2()
                        .p_3()
                        .gap_1()
                        .rounded(px(12.))
                        .bg(cx.theme().success.opacity(0.12))
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(
                                    Icon::new(IconName::CircleCheck)
                                        .size(px(16.))
                                        .text_color(cx.theme().success),
                                )
                                .child(
                                    div()
                                        .text_size(px(13.))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(cx.theme().success)
                                        .child(SharedString::from(app::t(cx, 585))),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(cx.theme().muted_foreground)
                                .child(SharedString::from(detail.clone())),
                        )
                        .into_any_element(),
                    Err(e) => h_flex()
                        .mt_2()
                        .p_3()
                        .gap_2()
                        .items_center()
                        .rounded(px(12.))
                        .bg(cx.theme().danger.opacity(0.1))
                        .child(
                            Icon::new(IconName::CircleAlert)
                                .size(px(16.))
                                .text_color(cx.theme().danger),
                        )
                        .child(
                            v_flex()
                                .gap(px(1.))
                                .child(
                                    div()
                                        .text_size(px(13.))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(cx.theme().danger)
                                        .child(SharedString::from(app::t(cx, 586))),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(cx.theme().muted_foreground)
                                        .child(SharedString::from(e.clone())),
                                ),
                        )
                        .into_any_element(),
                };
            }
        }

        let speed = self.downloader.speed.load(Ordering::Relaxed);
        let pct = job.progress.unwrap_or(0.);

        if !running && self.last.is_none() {
            return div()
                .mt_2()
                .text_size(px(12.))
                .text_color(cx.theme().muted_foreground)
                .child(SharedString::from(app::t(cx, 515)))
                .into_any_element();
        }

        v_flex()
            .mt_2()
            .gap_2()
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(13.))
                            .child(SharedString::from(if job.text.is_empty() {
                                app::t(cx, 515)
                            } else {
                                job.text.clone()
                            })),
                    )
                    .children(if running && speed > 0 {
                        vec![
                            div()
                                .text_size(px(12.))
                                .text_color(cx.theme().muted_foreground)
                                .child(SharedString::from(format!(
                                    "{}/s",
                                    Self::human(speed)
                                )))
                                .into_any_element(),
                        ]
                    } else {
                        Vec::new()
                    })
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(cx.theme().muted_foreground)
                            .child(SharedString::from(format!("{:.1}%", pct * 100.))),
                    ),
            )
            .child(
                div()
                    .h(px(6.))
                    .rounded_full()
                    .bg(cx.theme().muted)
                    .overflow_hidden()
                    .child(
                        div()
                            .h_full()
                            .rounded_full()
                            .bg(cx.theme().primary)
                            .w(gpui::relative(pct.clamp(0., 1.))),
                    ),
            )
            .into_any_element()
    }
}

impl Render for UpdateView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let mode_scan = self.mode_scan;
        let running = self.running.load(Ordering::Relaxed);

        v_flex()
            .size_full()
            .p_4()
            .gap_3()
            .text_color(cx.theme().foreground)
            .child(Self::segmented(
                cx,
                vec![
                    (true, app::t(cx, 576)),
                    (false, app::t(cx, 577)),
                ],
                mode_scan,
                0,
                view.clone(),
            ))
            .child(div().h(px(1.)).bg(cx.theme().border.opacity(0.6)))
            .child(if mode_scan {
                self.render_local(cx).into_any_element()
            } else {
                self.render_remote(cx).into_any_element()
            })
            .child(self.render_progress_panel(cx))
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        Button::new("update-start")
                            .label(app::t(cx, if mode_scan { 350 } else { 583 }))
                            .primary()
                            .with_size(gpui_component::Size::Small)
                            .on_click({
                                let view = view.clone();
                                move |_, _, cx| {
                                    let _ = view.update(cx, |v, cx| v.start_job(cx));
                                }
                            }),
                    )
                    .when(running, |el| {
                        el.child(
                            Button::new("update-stop")
                                .label(app::t(cx, 353))
                                .icon(Icon::new(IconName::X))
                                .ghost()
                                .with_size(gpui_component::Size::Small)
                                .on_click({
                                    let view = view.clone();
                                    move |_, _, cx| {
                                        let _ = view.update(cx, |v, cx| v.stop_job(cx));
                                    }
                                }),
                        )
                    }),
            )
    }
}

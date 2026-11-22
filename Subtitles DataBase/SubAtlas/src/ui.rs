// Application shell — FlClash-style layout:
// left navigation sidebar + page header + card-based pages, dark/light theme.
use crate::app::{self, Globals, LogLevel};
use crate::db::{SubInfo, COLUMNS};
use crate::scan;
use crate::search::{self, Filters};
use chrono::Datelike as _;
use gpui::{
    div, ease_out_quint, hsla, point, px, size, Animation, AnimationExt as _, AppContext as _,
    BoxShadow, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, SharedString, StatefulInteractiveElement as _, Styled as _, Window, WindowBounds,
    WindowOptions,
};
use gpui_component::{
    button::Button,
    input::{Input, InputEvent, InputState},
    select::{Select, SelectEvent, SelectState},
    table::{Column, ColumnSort, DataTable, TableDelegate, TableEvent, TableState},
    IndexPath, Icon, Root, Sizable as _, Theme, ThemeMode, TitleBar, ActiveTheme as _, h_flex,
    v_flex,
};
use gpui::prelude::FluentBuilder as _;
use gpui_component::button::ButtonVariants as _;
use gpui_component::searchable_list::SearchableVec;
use gpui_kit::assets::IconName;

// ---------------------------------------------------------------------------
// Page model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Database,
    Update,
    Search,
    Log,
    About,
}

impl Page {
    fn title_id(&self) -> i32 {
        match self {
            Page::Dashboard => 560,
            Page::Database => 561,
            Page::Update => 300,
            Page::Search => 200,
            Page::Log => 150,
            Page::About => 100,
        }
    }
}

// ---------------------------------------------------------------------------
// Table delegate
// ---------------------------------------------------------------------------

fn default_width(col: crate::db::Column) -> f32 {
    match col {
        crate::db::Column::Time => 110.,
        crate::db::Column::NameChs => 200.,
        crate::db::Column::NameCht => 180.,
        crate::db::Column::NameJp => 200.,
        crate::db::Column::NameEn => 180.,
        crate::db::Column::NameRome => 180.,
        crate::db::Column::Type => 90.,
        crate::db::Column::Source => 100.,
        crate::db::Column::SubName => 160.,
        crate::db::Column::Extension => 110.,
        crate::db::Column::Providers => 130.,
        crate::db::Column::Desc => 200.,
        crate::db::Column::Path => 320.,
    }
}

fn localized_header(cx: &gpui::App, col: crate::db::Column) -> SharedString {
    let id = match col {
        crate::db::Column::Time => 530,
        crate::db::Column::NameChs => 531,
        crate::db::Column::NameCht => 532,
        crate::db::Column::NameJp => 533,
        crate::db::Column::NameEn => 534,
        crate::db::Column::NameRome => 535,
        crate::db::Column::Type => 536,
        crate::db::Column::Source => 537,
        crate::db::Column::SubName => 538,
        crate::db::Column::Extension => 539,
        crate::db::Column::Providers => 540,
        crate::db::Column::Desc => 541,
        crate::db::Column::Path => 542,
    };
    let txt = app::t(cx, id);
    if txt == "(no msg)" {
        col.key().into()
    } else {
        txt.into()
    }
}

fn cell_text(sub: &SubInfo, col: crate::db::Column) -> String {
    match col {
        crate::db::Column::Time => sub.time.format("%Y-%m-%d").to_string(),
        crate::db::Column::NameChs => sub.name_chs.clone(),
        crate::db::Column::NameCht => sub.name_cht.clone(),
        crate::db::Column::NameJp => sub.name_jp.clone(),
        crate::db::Column::NameEn => sub.name_en.clone(),
        crate::db::Column::NameRome => sub.name_rome.clone(),
        crate::db::Column::Type => sub.r#type.clone(),
        crate::db::Column::Source => sub.source.clone(),
        crate::db::Column::SubName => sub.sub_name.clone(),
        crate::db::Column::Extension => sub.extension.clone(),
        crate::db::Column::Providers => sub.providers.clone(),
        crate::db::Column::Desc => sub.desc.clone(),
        crate::db::Column::Path => sub.path.clone(),
    }
}

pub struct SubTableDelegate {
    pub rows: Vec<SubInfo>,
    pub sort: Option<(usize, bool)>,
}

impl SubTableDelegate {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            sort: None,
        }
    }

    fn sort_rows(&mut self, col_ix: usize, desc: bool) {
        let col = COLUMNS[col_ix];
        self.rows.sort_by(|a, b| {
            let ord = match col {
                crate::db::Column::Time => a.time.cmp(&b.time),
                _ => {
                    let (x, y) = (cell_text(a, col), cell_text(b, col));
                    scan::natural_cmp(&x, &y)
                }
            };
            if desc {
                ord.reverse()
            } else {
                ord
            }
        });
    }
}

impl TableDelegate for SubTableDelegate {
    fn columns_count(&self, _: &gpui::App) -> usize {
        COLUMNS.len()
    }

    fn rows_count(&self, _: &gpui::App) -> usize {
        self.rows.len()
    }

    fn column(&self, col_ix: usize, cx: &gpui::App) -> Column {
        let col = COLUMNS[col_ix];
        let cfg = Globals::global(cx).config.lock().unwrap().clone();
        let w = cfg.column_widths[col_ix];
        let mut c = Column::new(col.key(), localized_header(cx, col))
            .resizable(true)
            .sortable();
        c = if w > 0 {
            c.width(px(w as f32))
        } else {
            c.width(px(default_width(col)))
        };
        if let Some((sorted_col, desc)) = self.sort {
            if sorted_col == col_ix {
                c = if desc {
                    c.descending()
                } else {
                    c.ascending()
                };
            }
        }
        c
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let col = COLUMNS[col_ix];
        let text: SharedString = cell_text(&self.rows[row_ix], col).into();
        div().overflow_hidden().whitespace_nowrap().child(text)
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        let desc = matches!(sort, ColumnSort::Descending);
        self.sort_rows(col_ix, desc);
        self.sort = Some((col_ix, desc));
        cx.notify();
    }
}

// ---------------------------------------------------------------------------
// Main shell
// ---------------------------------------------------------------------------

pub struct MainView {
    page: Page,
    // Data page state.
    table: Entity<TableState<SubTableDelegate>>,
    input: Entity<InputState>,
    type_sel: Entity<SelectState<SearchableVec<String>>>,
    source_sel: Entity<SelectState<SearchableVec<String>>>,
    lang_sel: Entity<SelectState<SearchableVec<String>>>,
    selected_row: Option<usize>,
    filtered: Vec<SubInfo>,
    /// Set from click handlers; executed during render where a Window exists.
    pending_clear: bool,
    pending_reload: bool,
    /// Cached dashboard stats: (total rows, series, types, last update text).
    stats: Option<(usize, usize, usize, String)>,
    /// Bumped on every page switch / theme switch, used as animation key.
    page_epoch: u64,
    theme_epoch: u64,
    // Embedded pages.
    update_view: Entity<crate::windows::UpdateView>,
    search_view: Entity<crate::windows::SearchView>,
}

impl MainView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx).placeholder(app::t(cx, 500))
        });

        let g = Globals::global(cx);
        let db_subs = g.db.0.read().unwrap().subs.clone();

        let (types, sources) = {
            let mut t: std::collections::BTreeSet<String> = Default::default();
            let mut s: std::collections::BTreeSet<String> = Default::default();
            for sub in &db_subs {
                if !sub.r#type.is_empty() {
                    t.insert(sub.r#type.clone());
                }
                if !sub.source.is_empty() {
                    s.insert(sub.source.clone());
                }
            }
            (t, s)
        };

        let all_label = app::t(cx, 503);
        let mut type_items = vec![all_label.clone()];
        type_items.extend(types.into_iter());
        let mut source_items = vec![all_label.clone()];
        source_items.extend(sources.into_iter());

        let type_sel = cx.new(|cx| {
            SelectState::new(
                SearchableVec::new(type_items),
                Some(IndexPath::default().row(0)),
                window,
                cx,
            )
        });
        let source_sel = cx.new(|cx| {
            SelectState::new(
                SearchableVec::new(source_items),
                Some(IndexPath::default().row(0)),
                window,
                cx,
            )
        });

        let lang = Globals::global(cx).lang.clone();
        let lang_names = lang.read(cx).language_names();
        let current = lang.read(cx).current.clone();
        let cur_ix = lang_names.iter().position(|n| *n == current).unwrap_or(0);
        let lang_sel = cx.new(|cx| {
            SelectState::new(
                SearchableVec::new(lang_names),
                Some(IndexPath::default().row(cur_ix)),
                window,
                cx,
            )
        });

        let table = cx.new(|cx| TableState::new(SubTableDelegate::new(), window, cx));

        let update_view = cx.new(|cx| crate::windows::UpdateView::new(window, cx));
        let search_view = cx.new(|cx| crate::windows::SearchView::new(window, cx));

        let mut this = Self {
            page: Page::Dashboard,
            table,
            input,
            type_sel,
            source_sel,
            lang_sel,
            selected_row: None,
            filtered: Vec::new(),
            pending_clear: false,
            pending_reload: false,
            stats: None,
            page_epoch: 1,
            theme_epoch: 1,
            update_view,
            search_view,
        };

        // Subscriptions -----------------------------------------------------
        cx.subscribe(&this.input, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change | InputEvent::PressEnter { .. }) {
                let value = this.input.read(cx).value().trim().to_string();
                let store = Globals::global(cx).filters.clone();
                store.update(cx, |store, cx| {
                    store.filters.name = value;
                    store.version += 1;
                    cx.notify();
                });
                this.apply_filters(cx);
            }
        })
        .detach();

        cx.subscribe(&this.type_sel, |this, _, event: &SelectEvent<_>, cx| {
            if let SelectEvent::Confirm(Some(v)) = event {
                this.set_filter(|f| f.r#type = normalize_all(&v), cx);
            }
        })
        .detach();

        cx.subscribe(&this.source_sel, |this, _, event: &SelectEvent<_>, cx| {
            if let SelectEvent::Confirm(Some(v)) = event {
                this.set_filter(|f| f.source = normalize_all(&v), cx);
            }
        })
        .detach();

        cx.subscribe(&this.lang_sel, |this, _, event: &SelectEvent<_>, cx| {
            if let SelectEvent::Confirm(Some(v)) = event {
                let lang = Globals::global(cx).lang.clone();
                lang.update(cx, |l, cx| {
                    l.set_language(&v);
                    cx.notify();
                });
                this.table.update(cx, |st, cx| st.refresh(cx));
                cx.notify();
            }
        })
        .detach();

        cx.subscribe(&this.table, |this, _, event: &TableEvent, cx| {
            match event {
                TableEvent::SelectRow(ix) => {
                    this.selected_row = Some(*ix);
                }
                TableEvent::DoubleClickedRow(ix) => {
                    this.selected_row = Some(*ix);
                    this.open_local_folder(cx);
                }
                TableEvent::ColumnWidthsChanged(widths) => {
                    let g = Globals::global(cx);
                    let mut cfg = g.config.lock().unwrap().clone();
                    for (i, w) in widths.iter().enumerate() {
                        if i < cfg.column_widths.len() {
                            cfg.column_widths[i] = w.as_f32() as i32;
                        }
                    }
                    *g.config.lock().unwrap() = cfg.clone();
                    let lang = g.lang.read(cx).clone();
                    let _ = crate::config::save_config(&app::config_path(), &cfg, &lang);
                }
                _ => {}
            }
        })
        .detach();

        // React to filter changes from the advanced search page: jump to the
        // database page to show the results.
        let filters_entity = Globals::global(cx).filters.clone();
        cx.observe(&filters_entity, |this, _, cx| {
            this.apply_filters(cx);
            // Advanced search (and any filter change) lands on the database
            // page so the results are visible immediately.
            let active = Globals::global(cx).filters.read(cx).filters.is_active();
            if active && this.page != Page::Database {
                this.set_page(Page::Database, cx);
            }
            cx.notify();
        })
        .detach();

        // React to language changes.
        let lang_entity = Globals::global(cx).lang.clone();
        cx.observe(&lang_entity, |this, _, cx| {
            this.table.update(cx, |st, cx| st.refresh(cx));
            cx.notify();
        })
        .detach();

        // Re-render (log page) when new log entries arrive.
        let log_entity = Globals::global(cx).log.clone();
        cx.observe(&log_entity, |_, _, cx| cx.notify()).detach();

        this.refresh_stats(cx);
        this.apply_filters(cx);

        // No local database at all: fetch the remote one automatically.
        let empty = Globals::global(cx).db.0.read().unwrap().subs.is_empty();
        if empty {
            let uv = this.update_view.clone();
            uv.update(cx, |v, cx| v.start_remote_download(cx));
        }
        this
    }

    fn refresh_stats(&mut self, cx: &mut Context<Self>) {
        let g = Globals::global(cx);
        let subs = g.db.0.read().unwrap().subs.clone();
        let cfg = g.config.lock().unwrap().clone();
        let mut types: std::collections::BTreeSet<&str> = Default::default();
        let mut series: std::collections::BTreeSet<&str> = Default::default();
        for sub in &subs {
            if !sub.r#type.is_empty() {
                types.insert(&sub.r#type);
            }
            series.insert(&sub.path);
        }
        let last = cfg
            .last_update_time
            .filter(|d| d.year() >= 1970)
            .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "-".into());
        self.stats = Some((subs.len(), series.len(), types.len(), last));
    }

    fn set_filter(&mut self, f: impl FnOnce(&mut Filters), cx: &mut Context<Self>) {
        let filters = Globals::global(cx).filters.clone();
        filters.update(cx, |store, cx| {
            f(&mut store.filters);
            store.version += 1;
            cx.notify();
        });
        self.apply_filters(cx);
    }

    /// Switch page with a transition animation.
    pub fn set_page(&mut self, page: Page, cx: &mut Context<Self>) {
        if self.page != page {
            self.page = page;
            self.page_epoch += 1;
        }
        cx.notify();
    }

    /// Ask the shell to reload the database during its next render.
    pub fn mark_pending_reload(&mut self) {
        self.pending_reload = true;
    }

    pub fn apply_filters(&mut self, cx: &mut Context<Self>) {
        let g = Globals::global(cx);
        let filters = g.filters.read(cx).filters.clone();

        let subs = g.db.0.read().unwrap().subs.clone();
        let ix = search::filter_all(&subs, &filters);
        let rows: Vec<SubInfo> = ix.into_iter().map(|i| subs[i].clone()).collect();

        self.filtered = rows.clone();
        let keep_sort = self.table.read(cx).delegate().sort;
        self.table.update(cx, |st, cx| {
            st.delegate_mut().rows = rows;
            if let Some((c, desc)) = keep_sort {
                st.delegate_mut().sort_rows(c, desc);
            }
            cx.notify();
        });
        cx.notify();
    }

    /// Called when the database was replaced (update page finished).
    pub fn on_db_reloaded(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let g = Globals::global(cx);
        let subs = g.db.0.read().unwrap().subs.clone();

        let (mut types, mut sources) = (
            std::collections::BTreeSet::new(),
            std::collections::BTreeSet::new(),
        );
        for sub in &subs {
            if !sub.r#type.is_empty() {
                types.insert(sub.r#type.clone());
            }
            if !sub.source.is_empty() {
                sources.insert(sub.source.clone());
            }
        }
        let all_label = app::t(cx, 503);
        let mut type_items = vec![all_label.clone()];
        type_items.extend(types.into_iter());
        let mut source_items = vec![all_label.clone()];
        source_items.extend(sources.into_iter());

        self.type_sel.update(cx, |s, cx| {
            let cur = s.selected_index(cx).map(|ix| ix.row);
            *s = SelectState::new(
                SearchableVec::new(type_items),
                Some(IndexPath::default().row(cur.unwrap_or(0))),
                window,
                cx,
            );
        });
        self.source_sel.update(cx, |s, cx| {
            let cur = s.selected_index(cx).map(|ix| ix.row);
            *s = SelectState::new(
                SearchableVec::new(source_items),
                Some(IndexPath::default().row(cur.unwrap_or(0))),
                window,
                cx,
            );
        });
        self.attach_select_subscriptions(cx);

        self.filtered = subs;
        self.table.update(cx, |st, cx| {
            st.delegate_mut().sort = None;
            st.refresh(cx);
            cx.notify();
        });
        let jump = Globals::global(cx)
            .jump_after_reload
            .load(std::sync::atomic::Ordering::Relaxed);
        if jump && self.page != Page::Database {
            self.page = Page::Database;
            self.page_epoch += 1;
        }
        self.refresh_stats(cx);
        self.apply_filters(cx);
    }

    fn attach_select_subscriptions(&mut self, cx: &mut Context<Self>) {
        cx.subscribe(&self.type_sel, |this, _, event: &SelectEvent<_>, cx| {
            if let SelectEvent::Confirm(Some(v)) = event {
                this.set_filter(|f| f.r#type = normalize_all(&v), cx);
            }
        })
        .detach();
        cx.subscribe(&self.source_sel, |this, _, event: &SelectEvent<_>, cx| {
            if let SelectEvent::Confirm(Some(v)) = event {
                this.set_filter(|f| f.source = normalize_all(&v), cx);
            }
        })
        .detach();
    }

    fn selected_sub(&self, cx: &Context<Self>) -> Option<SubInfo> {
        self.selected_row.and_then(|ix| self.filtered.get(ix).cloned())
    }

    pub fn open_local_folder(&mut self, cx: &mut Context<Self>) {
        let Some(sub) = self.selected_sub(cx) else { return };
        let cfg = Globals::global(cx).config.lock().unwrap().clone();
        let path = std::path::Path::new(&cfg.subs_path)
            .join(&sub.path)
            .join(&sub.source)
            .join(&sub.sub_name);
        if !path.is_dir() {
            let msg = app::fmt1(cx, 454, path.display().to_string());
            app::log(cx, msg, LogLevel::Error);
            return;
        }
        let msg = format!("explorer \"{}\"", path.display());
        app::log(cx, msg, LogLevel::Info);
        let _ = open::that(path);
    }

    pub fn open_remote_link(&mut self, cx: &mut Context<Self>) {
        let Some(sub) = self.selected_sub(cx) else { return };
        let joined = format!(
            "{}/{}/{}",
            sub.path.replace('\\', "/"),
            sub.source,
            sub.sub_name
        );
        let url = format!(
            "https://github.com/foxofice/sub_share/tree/master/subs_list/{}",
            percent_encode(&joined)
        );
        app::log(cx, url.clone(), LogLevel::Info);
        if let Err(e) = open::that(&url) {
            app::log(cx, format!("{e}"), LogLevel::Error);
        }
    }

    // -----------------------------------------------------------------------
    // Rendering helpers
    // -----------------------------------------------------------------------

    /// FlClash-style rounded card: white surface, soft shadow, 16px radius.
    fn card(&self, cx: &Context<Self>) -> gpui::Div {
        let dark = cx.theme().mode.is_dark();
        let sf = crate::theme::surfaces(dark);
        div()
            .rounded(px(16.))
            .bg(cx.theme().popover)
            .border_1()
            .border_color(gpui::rgb(sf.card_border))
            .shadow(vec![BoxShadow {
                color: hsla(0., 0., 0., if dark { 0.35 } else { 0.05 }),
                offset: point(px(0.), px(6.)),
                blur_radius: px(20.),
                spread_radius: px(-6.),
                inset: false,
            }])
    }

    /// FlClash-style pill chip.
    fn chip(&self, cx: &Context<Self>, icon: IconName, text: String) -> impl IntoElement {
        h_flex()
            .rounded_full()
            .bg(cx.theme().muted)
            .px_3()
            .py(px(5.))
            .gap_1p5()
            .text_size(px(12.))
            .text_color(cx.theme().muted_foreground)
            .child(Icon::new(icon).size(px(13.)))
            .child(SharedString::from(text))
    }

    /// Primary-filled pill (selected state, like FlClash's active tab).
    fn chip_active(&self, cx: &Context<Self>, icon: IconName, text: String) -> impl IntoElement {
        h_flex()
            .rounded_full()
            .bg(cx.theme().primary)
            .px_3()
            .py(px(5.))
            .gap_1p5()
            .text_size(px(12.))
            .text_color(cx.theme().primary_foreground)
            .child(Icon::new(icon).size(px(13.)))
            .child(SharedString::from(text))
    }

    /// FlClash-style sidebar: gray active pill, blue left indicator bar.
    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let dark = cx.theme().mode.is_dark();
        let sf = crate::theme::surfaces(dark);
        let items = [
            (Page::Dashboard, IconName::LayoutDashboard, 560),
            (Page::Database, IconName::Database, 561),
            (Page::Update, IconName::RefreshCw, 300),
            (Page::Search, IconName::SlidersHorizontal, 200),
            (Page::Log, IconName::FileText, 150),
            (Page::About, IconName::Info, 100),
        ];

        let nav = v_flex().px_2().gap_1().children(items.iter().map(
            |(page, icon, title_id)| {
                let page = *page;
                let icon = *icon;
                let title_id = *title_id;
                let active = self.page == page;
                let shell = cx.entity();
                let epoch = self.page_epoch;
                let mut item = h_flex()
                    .id(("nav", title_id as usize))
                    .relative()
                    .h(px(38.))
                    .px_3()
                    .gap_2p5()
                    .items_center()
                    .rounded(px(10.))
                    .cursor_pointer()
                    .hover(|s| s.bg(gpui::rgb(sf.hover)))
                    .on_click(move |_, _, cx| {
                        shell.update(cx, |shell, cx| {
                            shell.set_page(page, cx);
                        });
                    });
                if active {
                    item = item.child(
                        div()
                            .absolute()
                            .inset_0()
                            .rounded(px(10.))
                            .bg(gpui::rgb(sf.sidebar_pill))
                            .with_animation(
                                ("nav-pill", epoch),
                                Animation::new(std::time::Duration::from_millis(220))
                                    .with_easing(ease_out_quint()),
                                |el, t| el.opacity(t),
                            ),
                    );
                }
                item = item
                    .child(
                        div()
                            .w(px(18.))
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .text_color(if active {
                                cx.theme().primary
                            } else {
                                gpui::rgb(sf.chevron).into()
                            })
                            .child(Icon::new(icon).size(px(17.))),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(if active {
                                cx.theme().foreground
                            } else {
                                gpui::rgb(sf.sidebar_text).into()
                            })
                            .font_weight(if active {
                                gpui::FontWeight::SEMIBOLD
                            } else {
                                gpui::FontWeight::NORMAL
                            })
                            .child(SharedString::from(app::t(cx, title_id))),
                    );
                if active {
                    item = item.child(
                        div()
                            .absolute()
                            .left(px(-8.))
                            .top(px(10.))
                            .h(px(18.))
                            .w(px(3.))
                            .rounded_full()
                            .bg(cx.theme().primary)
                            .with_animation(
                                ("nav-bar", epoch),
                                Animation::new(std::time::Duration::from_millis(280))
                                    .with_easing(ease_out_quint()),
                                |el, t| el.opacity(t).h(px(6. + 12. * t)),
                            ),
                    );
                }
                item
            },
        ));

        // Theme toggle (light <-> dark).
        let shell_for_theme = cx.entity().downgrade();
        let theme_btn = Button::new("theme-toggle")
            .icon(Icon::new(if dark {
                IconName::Sun
            } else {
                IconName::Moon
            }))
            .ghost()
            .with_size(gpui_component::Size::Small)
            .on_click(move |_, _, cx| {
                let dark = cx.theme().mode.is_dark();
                crate::theme::apply(
                    cx,
                    if dark {
                        gpui_component::ThemeMode::Light
                    } else {
                        gpui_component::ThemeMode::Dark
                    },
                );
                let _ = shell_for_theme.update(cx, |m, cx| {
                    m.theme_epoch += 1;
                    cx.notify();
                });
            });

        v_flex()
            .w(px(200.))
            .flex_shrink_0()
            .h_full()
            .justify_between()
            .bg(gpui::rgb(sf.sidebar))
            .border_r_1()
            .border_color(cx.theme().border.opacity(0.6))
            .child(
              v_flex().gap_2().child(
                // Logo header
                h_flex()
                    .px_3()
                    .pt_4()
                    .pb_3()
                    .gap_2p5()
                    .items_center()
                    .child(
                        div()
                            .size(px(34.))
                            .rounded(px(10.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(cx.theme().primary)
                            .text_color(cx.theme().primary_foreground)
                            .child(Icon::new(IconName::Database).size(px(18.))),
                    )
                    .child(
                        v_flex()
                            .min_w_0()
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child(SharedString::from("SubAtlas")),
                            )
                            .child(
                                div()
                                    .text_size(px(10.))
                                    .text_color(cx.theme().muted_foreground)
                                    .child(SharedString::from(crate::windows::VERSION)),
                            ),
                    ),
            )
              .child(nav),
            )
            .child(
                // Footer: theme toggle + language
                h_flex()
                    .px_3()
                    .py_3()
                    .gap_2()
                    .items_center()
                    .border_t_1()
                    .border_color(cx.theme().border.opacity(0.6))
                    .child(theme_btn)
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                Select::new(&self.lang_sel)
                                    .menu_width(px(150.))
                                    .with_size(gpui_component::Size::Small),
                            ),
                    ),
            )
    }

    /// Page header: big title (FlClash style).
    fn render_page_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let title = app::t(cx, self.page.title_id());
        h_flex()
            .items_center()
            .justify_between()
            .px_6()
            .pt_4()
            .pb_3()
            .child(
                div()
                    .text_size(px(20.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(SharedString::from(title)),
            )
            .child(self.render_page_actions(cx))
    }

    fn render_page_actions(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let shell = cx.entity();
        h_flex().gap_2().items_center().children(match self.page {
            Page::Database => vec![
                Button::new("act-export")
                    .label(app::t(cx, 570))
                    .icon(Icon::new(IconName::Download))
                    .ghost()
                    .with_size(gpui_component::Size::Small)
                    .on_click(|_, _, cx| crate::windows::export_json_zstd(cx))
                    .into_any_element(),
                Button::new("act-folder")
                    .label(app::t(cx, 401))
                    .icon(Icon::new(IconName::FolderOpen))
                    .ghost()
                    .with_size(gpui_component::Size::Small)
                    .on_click({
                        let shell = shell.clone();
                        move |_, _, cx| {
                            shell.update(cx, |m, cx| m.open_local_folder(cx));
                        }
                    })
                    .into_any_element(),
                Button::new("act-url")
                    .label(app::t(cx, 402))
                    .icon(Icon::new(IconName::Link))
                    .ghost()
                    .with_size(gpui_component::Size::Small)
                    .on_click(move |_, _, cx| {
                        shell.update(cx, |m, cx| m.open_remote_link(cx));
                    })
                    .into_any_element(),
            ],
            _ => Vec::new(),
        })
    }

    // -----------------------------------------------------------------------
    // Pages
    // -----------------------------------------------------------------------

    /// Dashboard: FlClash stat cards + quick action list.
    fn render_dashboard(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let dark = cx.theme().mode.is_dark();
        let sf = crate::theme::surfaces(dark);
        let (total, n_series, n_types, last) = self
            .stats
            .clone()
            .unwrap_or_else(|| (0, 0, 0, "-".into()));

        // FlClash stat card: small header (icon + muted label), big value.
        let epoch = self.page_epoch;
        let stat = |ix: usize, label: String, value: String, big: bool, icon: IconName| {
            v_flex()
                .flex_1()
                .min_w(px(150.))
                .p_4()
                .gap_3()
                .rounded(px(16.))
                .bg(cx.theme().popover)
                .border_1()
                .border_color(gpui::rgb(sf.card_border))
                .shadow(vec![BoxShadow {
                    color: hsla(0., 0., 0., if dark { 0.35 } else { 0.05 }),
                    offset: point(px(0.), px(6.)),
                    blur_radius: px(20.),
                    spread_radius: px(-6.),
                    inset: false,
                }])
                .child(
                    h_flex()
                        .gap_1p5()
                        .items_center()
                        .text_size(px(12.))
                        .text_color(cx.theme().muted_foreground)
                        .child(Icon::new(icon).size(px(14.)))
                        .child(SharedString::from(label)),
                )
                .child(
                    div()
                        .text_size(px(if big { 28. } else { 20. }))
                        .font_weight(gpui::FontWeight::BOLD)
                        .child(SharedString::from(value)),
                )
                .with_animations(
                    format!("stat-in-{epoch}-{ix}"),
                    vec![
                        Animation::new(std::time::Duration::from_millis(40 * ix as u64)),
                        Animation::new(std::time::Duration::from_millis(260))
                            .with_easing(ease_out_quint()),
                    ],
                    |el, phase, t| {
                        if phase == 0 {
                            el.opacity(0.)
                        } else {
                            el.opacity(t).mt(px((1.0 - t) * 12.))
                        }
                    },
                )
        };

        let action = |id: &'static str,
                      label: String,
                      sub: String,
                      icon: IconName,
                      shell: &Entity<MainView>,
                      page: Page| {
            let shell = shell.clone();
            let sf = crate::theme::surfaces(dark);
            h_flex()
                .id(id)
                .px_4()
                .py_3()
                .gap_3()
                .items_center()
                .cursor_pointer()
                .hover(|s| s.bg(gpui::rgb(sf.hover)))
                .on_click(move |_, _, cx| {
                    let shell = shell.clone();
                    shell.update(cx, |m, cx| {
                        m.set_page(page, cx);
                    });
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
                                .child(SharedString::from(label)),
                        )
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(cx.theme().muted_foreground)
                                .child(SharedString::from(sub)),
                        ),
                )
                .child(
                    Icon::new(IconName::ChevronRight)
                        .size(px(16.))
                        .text_color(gpui::rgb(sf.chevron)),
                )
        };

        let shell = cx.entity();
        v_flex()
            .id("dash-page")
            .flex_1()
            .min_h_0()
            .gap_4()
            .px_6()
            .pt_1()
            .pb_6()
            .overflow_y_scroll()
            .child(
                div().flex().flex_wrap().gap_3().children([
                    stat(0, app::t(cx, 564), format!("{total}"), true, IconName::Database),
                    stat(1, app::t(cx, 565), format!("{n_series}"), true, IconName::Layers),
                    stat(2, app::t(cx, 563), format!("{n_types}"), true, IconName::Table),
                    stat(
                        3,
                        app::fmt1(cx, 505, last.clone()),
                        last,
                        false,
                        IconName::Clock,
                    ),
                ]),
            )
            .child(
                div()
                    .mt_1()
                    .text_size(px(12.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(cx.theme().muted_foreground)
                    .child(SharedString::from(app::t(cx, 562))),
            )
            .child(
                v_flex()
                    .rounded(px(16.))
                    .bg(cx.theme().popover)
                    .border_1()
                    .border_color(gpui::rgb(sf.card_border))
                    .shadow(vec![BoxShadow {
                        color: hsla(0., 0., 0., if dark { 0.35 } else { 0.05 }),
                        offset: point(px(0.), px(6.)),
                        blur_radius: px(20.),
                        spread_radius: px(-6.),
                        inset: false,
                    }])
                    .overflow_hidden()
                    .with_animations(
                        ("qa-in", epoch),
                        vec![
                            Animation::new(std::time::Duration::from_millis(120)),
                            Animation::new(std::time::Duration::from_millis(260))
                                .with_easing(ease_out_quint()),
                        ],
                        |el, phase, t| {
                            if phase == 0 {
                                el.opacity(0.)
                            } else {
                                el.opacity(t).mt(px((1.0 - t) * 12.))
                            }
                        },
                    )
                    .child(action(
                        "qa-download",
                        app::t(cx, 352),
                        crate::dl::DB_URL_SHORT.to_string(),
                        IconName::Download,
                        &shell,
                        Page::Update,
                    ))
                    .child(div().h(px(1.)).bg(cx.theme().border.opacity(0.6)))
                    .child(action(
                        "qa-scan",
                        app::t(cx, 350),
                        app::t(cx, 519),
                        IconName::RefreshCw,
                        &shell,
                        Page::Update,
                    ))
                    .child(div().h(px(1.)).bg(cx.theme().border.opacity(0.6)))
                    .child(action(
                        "qa-search",
                        app::t(cx, 403),
                        app::t(cx, 422),
                        IconName::SlidersHorizontal,
                        &shell,
                        Page::Search,
                    ))
                    .child(div().h(px(1.)).bg(cx.theme().border.opacity(0.6)))
                    .child(action(
                        "qa-db",
                        app::t(cx, 561),
                        app::t(cx, 506),
                        IconName::Database,
                        &shell,
                        Page::Database,
                    ))
                    .child(div().h(px(1.)).bg(cx.theme().border.opacity(0.6)))
                    .child(
                        h_flex()
                            .id("qa-site")
                            .px_4()
                            .py_3()
                            .gap_3()
                            .items_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(gpui::rgb(sf.hover)))
                            .on_click(|_, _, _| {
                                let _ = open::that(crate::windows::SITE_URL);
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
                                    .child(Icon::new(IconName::Globe).size(px(17.))),
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
                                            .child(SharedString::from(app::t(cx, 567))),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .text_color(cx.theme().muted_foreground)
                                            .child(SharedString::from(
                                                "foxofice.github.io/sub_share",
                                            )),
                                    ),
                            )
                            .child(
                                Icon::new(IconName::ChevronRight)
                                    .size(px(16.))
                                    .text_color(gpui::rgb(sf.chevron)),
                            ),
                    ),
            )
    }

    /// Database page: chips + search bar + table card.
    fn render_database_page(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let table = self.table.clone();
        let n_rows = self.filtered.len();
        let n_series = self
            .filtered
            .iter()
            .map(|s| s.path.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let cfg = Globals::global(cx).config.lock().unwrap().clone();
        let last = cfg
            .last_update_time
            .filter(|d| d.year() >= 1970)
            .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "-".into());

        v_flex()
            .size_full()
            .gap_3()
            .px_6()
            .pb_6()
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(self.chip(cx, IconName::Database, app::fmt1(cx, 451, n_rows.to_string())))
                    .child(self.chip(cx, IconName::Layers, app::fmt1(cx, 452, n_series.to_string())))
                    .child(self.chip(cx, IconName::Clock, app::fmt1(cx, 505, last))),
            )
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div().w(px(420.)).child(
                            Input::new(&self.input)
                                .cleanable(true)
                                .w_full()
                                .with_size(gpui_component::Size::Small),
                        ),
                    )
                    .child(
                        div().w(px(150.)).child(
                            Select::new(&self.type_sel)
                                .menu_width(px(150.))
                                .with_size(gpui_component::Size::Small),
                        ),
                    )
                    .child(
                        div().w(px(140.)).child(
                            Select::new(&self.source_sel)
                                .menu_width(px(150.))
                                .with_size(gpui_component::Size::Small),
                        ),
                    )
                    .child(
                        Button::new("btn-clear")
                            .label(app::t(cx, 502))
                            .icon(Icon::new(IconName::X))
                            .ghost()
                            .with_size(gpui_component::Size::Small)
                            .on_click(|_, _, cx| {
                                let filters = Globals::global(cx).filters.clone();
                                filters.update(cx, |store, cx| {
                                    store.filters = Filters::default();
                                    store.version += 1;
                                    cx.notify();
                                });
                                let mv = Globals::global(cx).main_view.clone();
                                if let Some(m) = mv.upgrade() {
                                    m.update(cx, |m, cx| {
                                        m.pending_clear = true;
                                        cx.notify();
                                    });
                                }
                            }),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .rounded(px(14.))
                    .overflow_hidden()
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().secondary)
                    .child(DataTable::new(&table)),
            )
    }

    fn render_page(&self, cx: &mut Context<Self>) -> impl IntoElement {
        match self.page {
            Page::Dashboard => self.render_dashboard(cx).into_any_element(),
            Page::Database => self.render_database_page(cx).into_any_element(),
            Page::Update => self
                .card(cx)
                .id("page-update")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .mx_6()
                .mb_5()
                .p_4()
                .max_w(px(760.))
                .child(self.update_view.clone())
                .into_any_element(),
            Page::Search => self
                .card(cx)
                .id("page-search")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .mx_6()
                .mb_5()
                .p_4()
                .max_w(px(760.))
                .child(self.search_view.clone())
                .into_any_element(),
            Page::Log => self
                .card(cx)
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .mx_6()
                .mb_5()
                .child(crate::windows::log_page(cx))
                .into_any_element(),
            Page::About => v_flex()
                .id("page-about")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .px_6()
                .pb_6()
                .child(
                    div()
                        .flex_shrink_0()
                        .child(crate::windows::about_page(cx)),
                )
                .into_any_element(),
        }
    }
}

fn normalize_all(v: &str) -> String {
    if v.starts_with('(') && v.ends_with(')') {
        String::new()
    } else {
        v.to_string()
    }
}

/// Same character class as C# Uri.EscapeDataString.
pub fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        let c = *b as char;
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
            out.push(c);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

impl Render for MainView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.pending_reload {
            self.pending_reload = false;
            self.on_db_reloaded(window, cx);
        }
        if self.pending_clear {
            self.pending_clear = false;
            self.type_sel.update(cx, |s, cx| {
                s.set_selected_index(Some(IndexPath::default().row(0)), window, cx);
            });
            self.source_sel.update(cx, |s, cx| {
                s.set_selected_index(Some(IndexPath::default().row(0)), window, cx);
            });
            self.input.update(cx, |st, cx| st.set_value("", window, cx));
            self.apply_filters(cx);
        }
        h_flex()
            .id("main-root")
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .on_key_down(|event: &gpui::KeyDownEvent, _, cx| {
                // Hotkeys: F5 -> update page, F3 -> advanced search, F4 -> log.
                let target = match event.keystroke.key.as_str() {
                    "f5" => Some(Page::Update),
                    "f3" => Some(Page::Search),
                    "f4" => Some(Page::Log),
                    _ => None,
                };
                if let Some(page) = target {
                    let shell = Globals::global(cx).main_view.clone();
                    let _ = shell.update(cx, |shell, cx| {
                        shell.set_page(page, cx);
                    });
                }
            })
            .child(self.render_sidebar(cx))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .size_full()
                    .child(
                        TitleBar::new().child(
                            div()
                                .px_2()
                                .text_size(px(12.))
                                .text_color(cx.theme().muted_foreground)
                                .child(SharedString::from(
                                    "SubAtlas — foxofice/sub_share",
                                )),
                        ),
                    )
                    .child(self.render_page_header(cx))
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .child(
                                v_flex()
                                    .size_full()
                                    .min_h_0()
                                    .child(self.render_page(cx))
                                    .with_animation(
                                        ("theme-fade", self.theme_epoch),
                                        Animation::new(std::time::Duration::from_millis(220))
                                            .with_easing(ease_out_quint()),
                                        |el, t| el.opacity(0.35 + 0.65 * t),
                                    ),
                            )
                            .with_animation(
                                ("page-anim", self.page_epoch),
                                Animation::new(std::time::Duration::from_millis(220))
                                    .with_easing(ease_out_quint()),
                                |el, t| el.opacity(t).mt(px((1.0 - t) * 10.)),
                            ),
                    ),
            )
    }
}

/// Open the main window (called from main.rs).
pub fn open_main_window(cx: &mut gpui::App) -> gpui::WindowHandle<Root> {
    let cfg = Globals::global(cx).config.lock().unwrap().clone();
    // Clamp the persisted geometry so the window always fits a 1080p screen.
    let (mut w, mut h) = if cfg.window_width > 200 && cfg.window_height > 200 {
        (cfg.window_width as f32, cfg.window_height as f32)
    } else {
        (1200., 800.)
    };
    w = w.clamp(1000., 1360.);
    h = h.clamp(640., 880.);
    let (x, y) = (
        cfg.window_x.clamp(0, 400) as f32,
        cfg.window_y.clamp(0, 200) as f32,
    );
    let bounds = gpui::Bounds::new(gpui::point(px(x), px(y)), size(px(w), px(h)));

    let opts = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(px(1000.), px(640.))),
        ..TitleBar::window_options()
    };

    cx.open_window(opts, |window, cx| {
        let view = cx.new(|cx| MainView::new(window, cx));
        Globals::global_mut(cx).main_view = view.downgrade();
        let root_view: gpui::AnyView = view.into();
        cx.new(|cx| Root::new(root_view, window, cx))
    })
    .expect("failed to open main window")
}

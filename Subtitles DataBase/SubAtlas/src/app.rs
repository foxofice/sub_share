// Application-wide state: config, database, language, log and filters.
// Shared as a gpui Global so every window can access it.
use crate::config::Config;
use crate::db::SharedDb;
use crate::i18n::LangStore;
use chrono::FixedOffset;
use gpui::{App, AppContext as _, Entity, WeakEntity};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Base directory = directory of the exe (Files/ lives next to it).
pub fn base_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn files_dir() -> PathBuf {
    base_dir().join("Files")
}

/// Default database directory: Files/ next to the exe.
pub fn default_db_dir() -> PathBuf {
    files_dir()
}

static DB_DIR: std::sync::RwLock<Option<PathBuf>> = std::sync::RwLock::new(None);

/// Active database directory (config `db.path`; defaults to exe_dir/Files).
pub fn db_dir() -> PathBuf {
    DB_DIR
        .read()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(default_db_dir)
}

pub fn set_db_dir(dir: PathBuf) {
    if let Ok(mut g) = DB_DIR.write() {
        *g = Some(dir);
    }
}

pub fn db_xml_path() -> PathBuf {
    db_dir().join("db.xml")
}

pub fn config_path() -> PathBuf {
    files_dir().join("config.txt")
}

pub fn db_json_path() -> PathBuf {
    db_dir().join("db.json.zst")
}

pub fn local_offset() -> FixedOffset {
    *chrono::Local::now().offset()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone)]
pub struct LogEntry {
    pub time: chrono::NaiveDateTime,
    pub text: String,
    pub level: LogLevel,
}

pub struct LogStore {
    pub entries: Vec<LogEntry>,
}

impl LogStore {
    pub fn push(&mut self, text: &str, level: LogLevel, cx: &mut gpui::Context<Self>) {
        self.entries.push(LogEntry {
            time: chrono::Local::now().naive_local(),
            text: text.to_string(),
            level,
        });
        if self.entries.len() > 4000 {
            self.entries.drain(0..1000);
        }
        cx.notify();
    }
}

pub struct FilterStore {
    pub filters: crate::search::Filters,
    pub version: u64,
}

impl FilterStore {
    pub fn set(&mut self, filters: crate::search::Filters, cx: &mut gpui::Context<Self>) {
        self.filters = filters;
        self.version += 1;
        cx.notify();
    }
}

pub struct Globals {
    /// After the database reloads, jump to the database page (used only for
    /// the automatic first-run download; manual updates stay on the page).
    pub jump_after_reload: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub lang: Entity<LangStore>,
    pub log: Entity<LogStore>,
    pub filters: Entity<FilterStore>,
    pub config: Arc<Mutex<Config>>,
    pub db: SharedDb,
    pub main_view: WeakEntity<crate::ui::MainView>,
}

impl gpui::Global for Globals {}

impl Globals {
    pub fn global(cx: &App) -> &Globals {
        cx.global::<Globals>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Globals {
        cx.global_mut::<Globals>()
    }
}

/// Convenience: translate with the current language.
pub fn t(cx: &App, id: i32) -> String {
    Globals::global(cx).lang.read(cx).txt(id)
}

pub fn fmt(cx: &App, id: i32, args: &[String]) -> String {
    crate::i18n::fmt(&t(cx, id), args)
}

pub fn fmt1(cx: &App, id: i32, a: impl Into<String>) -> String {
    crate::i18n::fmt1(&t(cx, id), a)
}

pub fn fmt2(cx: &App, id: i32, a: impl Into<String>, b: impl Into<String>) -> String {
    crate::i18n::fmt2(&t(cx, id), a, b)
}

/// Append a log entry (goes to the log window).
pub fn log(cx: &mut App, text: impl Into<String>, level: LogLevel) {
    let log = Globals::global(cx).log.clone();
    log.update(cx, |store, cx| {
        store.push(&text.into(), level, cx);
    });
}

/// Initialize globals, load config/db/languages, set default language.
pub fn init_globals(cx: &mut App) {
    let mut lang = LangStore::new();
    lang.load_dir(&files_dir().join("Languages"));
    // Chinese is the primary UI language; English remains selectable.
    let _ = sys_locale::get_locale();
    lang.set_language("zh-CN");

    let cfg = crate::config::read_config(&config_path());

    // Database location: user override from config, else next to the exe.
    let db_dir = if cfg.db_path.trim().is_empty() {
        default_db_dir()
    } else {
        PathBuf::from(cfg.db_path.trim())
    };
    // Make sure a configured directory still exists before downloads land there.
    let _ = std::fs::create_dir_all(&db_dir);
    set_db_dir(db_dir);

    let mut db_store = crate::db::DbStore::default();
    let startup_msg = match db_store.load_from_file(&db_xml_path()) {
        Ok((n, src)) => {
            let fmt = match src {
                crate::db::DbSource::JsonZstd => "db.json.zst",
                crate::db::DbSource::Xml => "db.xml",
                crate::db::DbSource::XmlMigrated => "db.xml (converted to db.json.zst)",
                crate::db::DbSource::Missing => "no database file",
            };
            Some((format!("loaded {n} rows from {fmt}"), LogLevel::Info))
        }
        Err(e) => Some((format!("failed to load database: {e}"), LogLevel::Error)),
    };

    let lang_entity = cx.new(|_| lang);
    let log_entity = cx.new(|_| LogStore { entries: Vec::new() });
    let filter_entity = cx.new(|_| FilterStore {
        filters: Default::default(),
        version: 0,
    });
    let db = SharedDb(Arc::new(std::sync::RwLock::new(db_store)));

    cx.set_global(Globals {
        jump_after_reload: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        lang: lang_entity,
        log: log_entity,
        filters: filter_entity,
        config: Arc::new(Mutex::new(cfg)),
        db,
        main_view: WeakEntity::<crate::ui::MainView>::new_invalid(),
    });

    if let Some((msg, level)) = startup_msg {
        log(cx, msg, level);
    }
}

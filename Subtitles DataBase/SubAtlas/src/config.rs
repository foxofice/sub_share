// Files/config.txt — same keys and format as the original app.
use crate::db;
use crate::i18n;
use std::path::Path;

pub const CONFIG_FILE: &str = "Files\\config.txt";

#[derive(Debug, Clone)]
pub struct Config {
    pub subs_path: String,
    pub last_update_time: Option<chrono::NaiveDateTime>,
    pub window_width: i32,
    pub window_height: i32,
    pub window_x: i32,
    pub window_y: i32,
    /// Column widths in the original 13-column order; -1 = default.
    pub column_widths: [i32; 13],
}

impl Default for Config {
    fn default() -> Self {
        Self {
            subs_path: "..\\subs_list".to_string(),
            last_update_time: None,
            window_width: 0,
            window_height: 0,
            window_x: -1,
            window_y: -1,
            column_widths: [-1; 13],
        }
    }
}

const WIDTH_KEYS: [&str; 13] = [
    "column.width.time",
    "column.width.name_chs",
    "column.width.name_cht",
    "column.width.name_jp",
    "column.width.name_en",
    "column.width.name_rome",
    "column.width.type",
    "column.width.source",
    "column.width.sub_name",
    "column.width.extension",
    "column.width.providers",
    "column.width.desc",
    "column.width.path",
];

pub fn read_config(path: &Path) -> Config {
    let mut cfg = Config::default();
    let Ok(text) = std::fs::read_to_string(path) else {
        return cfg;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.len() < 2 || line.starts_with("//") {
            continue;
        }
        let Some(idx) = line.find(':') else { continue };
        let key = line[..idx].trim();
        let value = line[idx + 1..].trim();
        match key.to_ascii_lowercase().as_str() {
            "subs.path" => cfg.subs_path = value.to_string(),
            "subs.last_update_time" => {
                if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(value, "%Y/%m/%d %H:%M:%S")
                {
                    cfg.last_update_time = Some(dt);
                }
            }
            "window.width" => cfg.window_width = value.parse().unwrap_or(0),
            "window.height" => cfg.window_height = value.parse().unwrap_or(0),
            "window.x" => cfg.window_x = value.parse().unwrap_or(-1),
            "window.y" => cfg.window_y = value.parse().unwrap_or(-1),
            _ => {
                if let Some(rest) = key.to_ascii_lowercase().strip_prefix("column.width.") {
                    if let Some(pos) = db::Column::index_by_key(rest) {
                        cfg.column_widths[pos] = value.parse().unwrap_or(-1);
                    }
                }
            }
        }
    }
    cfg
}

/// Save with the same key layout (and localized comments) as the original.
pub fn save_config(path: &Path, cfg: &Config, lang: &i18n::LangStore) -> std::io::Result<()> {
    let mut s = String::with_capacity(2048);
    s.push_str(&format!("// {}\r\nsubs.path: {}\r\n\r\n", lang.txt(20), cfg.subs_path));
    let lut = cfg
        .last_update_time
        .map(|d| d.format("%Y/%m/%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "0001/1/1 0:00:00".to_string());
    s.push_str(&format!("// {}\r\nsubs.last_update_time: {}\r\n\r\n", lang.txt(21), lut));
    s.push_str(&format!(
        "// {}\r\nwindow.width: {}\r\nwindow.height: {}\r\n\r\n",
        lang.txt(22),
        cfg.window_width,
        cfg.window_height
    ));
    s.push_str(&format!(
        "// {}\r\nwindow.x: {}\r\nwindow.y: {}\r\n\r\n",
        lang.txt(23),
        cfg.window_x,
        cfg.window_y
    ));
    s.push_str(&format!("// {}\r\n", lang.txt(24)));
    for (i, k) in WIDTH_KEYS.iter().enumerate() {
        s.push_str(&format!("{k}: {}\r\n", cfg.column_widths[i]));
    }

    let tmp = path.with_extension("txt.tmp");
    std::fs::write(&tmp, s)?;
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
    std::fs::rename(&tmp, path)
}

// Data model and db.xml (DataTable-compatible) read/write.
use chrono::FixedOffset;
use std::path::Path;
use std::sync::RwLock;

pub const DB_FILENAME: &str = "Files\\db.xml";
/// Preferred compressed format (JSON + zstd).
pub const DB_JSON_FILENAME: &str = "Files\\db.json.zst";

/// Column identifiers, same order as the original DataTable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    Time,
    NameChs,
    NameCht,
    NameJp,
    NameEn,
    NameRome,
    Type,
    Source,
    SubName,
    Extension,
    Providers,
    Desc,
    Path,
}

pub const COLUMNS: [Column; 13] = [
    Column::Time,
    Column::NameChs,
    Column::NameCht,
    Column::NameJp,
    Column::NameEn,
    Column::NameRome,
    Column::Type,
    Column::Source,
    Column::SubName,
    Column::Extension,
    Column::Providers,
    Column::Desc,
    Column::Path,
];

impl Column {
    pub fn key(&self) -> &'static str {
        match self {
            Column::Time => "time",
            Column::NameChs => "name_chs",
            Column::NameCht => "name_cht",
            Column::NameJp => "name_jp",
            Column::NameEn => "name_en",
            Column::NameRome => "name_rome",
            Column::Type => "type",
            Column::Source => "source",
            Column::SubName => "sub_name",
            Column::Extension => "extension",
            Column::Providers => "providers",
            Column::Desc => "desc",
            Column::Path => "path",
        }
    }

    /// Index of the column in the canonical order, by key name.
    pub fn index_by_key(key: &str) -> Option<usize> {
        COLUMNS.iter().position(|c| c.key().eq_ignore_ascii_case(key))
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubInfo {
    pub time: chrono::NaiveDate,
    pub name_chs: String,
    pub name_cht: String,
    pub name_jp: String,
    pub name_en: String,
    pub name_rome: String,
    pub r#type: String,
    pub source: String,
    pub sub_name: String,
    pub extension: String,
    pub providers: String,
    pub desc: String,
    pub path: String,
}

impl SubInfo {
    /// Time rendered the way DataTable.WriteXml does: local offset appended.
    pub fn time_xml(&self, offset: FixedOffset) -> String {
        let dt = self
            .time
            .and_hms_opt(0, 0, 0)
            .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap());
        dt.and_local_timezone(offset)
            .single()
            .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap().with_timezone(&offset))
            .format("%Y-%m-%dT%H:%M:%S%:z")
            .to_string()
    }
}

pub fn parse_time(s: &str) -> Option<chrono::NaiveDate> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.date_naive());
    }
    for fmt in ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S", "%Y-%m-%d"] {
        if let Ok(d) = chrono::NaiveDateTime::parse_from_str(s, fmt) {
            return Some(d.date());
        }
        if let Ok(d) = chrono::NaiveDate::parse_from_str(s, fmt) {
            return Some(d);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Reading
// ---------------------------------------------------------------------------

pub fn read_db(path: &Path) -> anyhow::Result<Vec<SubInfo>> {
    let mut text = std::fs::read_to_string(path)?;
    // The upstream file (and .NET-written XML) may carry a UTF-8 BOM.
    if text.starts_with('\u{feff}') {
        text.remove(0);
    }
    let mut reader = quick_xml::Reader::from_str(&text);
    reader.config_mut().trim_text(true);

    let mut subs = Vec::new();
    let mut cur: Option<SubInfo> = None;
    let mut cur_key: Option<String> = None;
    let mut buf = Vec::new();

    use quick_xml::events::Event;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "subs" {
                    cur = Some(SubInfo::default());
                } else if cur.is_some() {
                    cur_key = Some(name);
                }
            }
            Ok(Event::Empty(e)) => {
                // Self-closing element (e.g. `<providers />`): just a missing key.
                let _ = e;
                cur_key = None;
            }
            Ok(Event::Text(t)) => {
                if let (Some(sub), Some(key)) = (cur.as_mut(), cur_key.as_deref()) {
                    let decoded = t.unescape()?.into_owned();
                    assign_field(sub, key, &decoded);
                }
            }
            Ok(Event::CData(t)) => {
                if let (Some(sub), Some(key)) = (cur.as_mut(), cur_key.as_deref()) {
                    let raw = String::from_utf8_lossy(t.as_ref()).to_string();
                    assign_field(sub, key, &raw);
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "subs" {
                    if let Some(sub) = cur.take() {
                        subs.push(sub);
                    }
                }
                cur_key = None;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("db.xml parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }
    Ok(subs)
}

fn assign_field(sub: &mut SubInfo, key: &str, value: &str) {
    match key {
        "time" => {
            if let Some(d) = parse_time(value) {
                sub.time = d;
            }
        }
        "name_chs" => sub.name_chs = value.to_string(),
        "name_cht" => sub.name_cht = value.to_string(),
        "name_jp" => sub.name_jp = value.to_string(),
        "name_en" => sub.name_en = value.to_string(),
        "name_rome" => sub.name_rome = value.to_string(),
        "type" => sub.r#type = value.to_string(),
        "source" => sub.source = value.to_string(),
        "sub_name" => sub.sub_name = value.to_string(),
        "extension" => sub.extension = value.to_string(),
        "providers" => sub.providers = value.to_string(),
        "desc" => sub.desc = value.to_string(),
        "path" => sub.path = value.to_string(),
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Writing (byte-shape compatible with DataTable.WriteXml)
// ---------------------------------------------------------------------------

fn escape_xml(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '\'' => out.push_str("&apos;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
}

pub fn write_db(path: &Path, subs: &[SubInfo], offset: FixedOffset) -> std::io::Result<()> {
    let mut out = String::with_capacity(1024 + subs.len() * 512);
    out.push_str("<?xml version=\"1.0\" standalone=\"yes\"?>\r\n<DocumentElement>");
    for sub in subs {
        out.push_str("\r\n  <subs>\r\n");
        let pairs: [(&str, String); 13] = [
            ("time", sub.time_xml(offset)),
            ("name_chs", sub.name_chs.clone()),
            ("name_cht", sub.name_cht.clone()),
            ("name_jp", sub.name_jp.clone()),
            ("name_en", sub.name_en.clone()),
            ("name_rome", sub.name_rome.clone()),
            ("type", sub.r#type.clone()),
            ("source", sub.source.clone()),
            ("sub_name", sub.sub_name.clone()),
            ("extension", sub.extension.clone()),
            ("providers", sub.providers.clone()),
            ("desc", sub.desc.clone()),
            ("path", sub.path.clone()),
        ];
        for (k, v) in pairs {
            if v.is_empty() {
                out.push_str(&format!("    <{k} />\r\n"));
            } else {
                out.push_str(&format!("    <{k}>"));
                escape_xml(&v, &mut out);
                out.push_str(&format!("</{k}>\r\n"));
            }
        }
        out.push_str("  </subs>");
    }
    out.push_str("\r\n</DocumentElement>");

    // Atomic-ish write like the original: temp file then rename.
    let tmp = path.with_extension("xml.tmp");
    std::fs::write(&tmp, out)?;
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
    std::fs::rename(&tmp, path)
}

// ---------------------------------------------------------------------------
// JSON + zstd (preferred storage format)
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize)]
struct SubInfoJson {
    time: String,
    name_chs: String,
    name_cht: String,
    name_jp: String,
    name_en: String,
    name_rome: String,
    #[serde(rename = "type")]
    type_name: String,
    source: String,
    sub_name: String,
    extension: String,
    providers: String,
    desc: String,
    path: String,
}

impl SubInfoJson {
    fn from_sub(sub: &SubInfo) -> Self {
        Self {
            time: sub.time.format("%Y-%m-%d").to_string(),
            name_chs: sub.name_chs.clone(),
            name_cht: sub.name_cht.clone(),
            name_jp: sub.name_jp.clone(),
            name_en: sub.name_en.clone(),
            name_rome: sub.name_rome.clone(),
            type_name: sub.r#type.clone(),
            source: sub.source.clone(),
            sub_name: sub.sub_name.clone(),
            extension: sub.extension.clone(),
            providers: sub.providers.clone(),
            desc: sub.desc.clone(),
            path: sub.path.clone(),
        }
    }

    fn into_sub(self) -> SubInfo {
        SubInfo {
            time: parse_time(&self.time).unwrap_or_else(|| {
                chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
            }),
            name_chs: self.name_chs,
            name_cht: self.name_cht,
            name_jp: self.name_jp,
            name_en: self.name_en,
            name_rome: self.name_rome,
            r#type: self.type_name,
            source: self.source,
            sub_name: self.sub_name,
            extension: self.extension,
            providers: self.providers,
            desc: self.desc,
            path: self.path,
        }
    }
}

fn io_err(e: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
}

/// Parse a JSON + zstd database file.
pub fn read_db_json_zst(path: &Path) -> anyhow::Result<Vec<SubInfo>> {
    let raw = std::fs::read(path)?;
    let json = zstd::stream::decode_all(&raw[..])
        .map_err(|e| anyhow::anyhow!("zstd decode: {e}"))?;
    let rows: Vec<SubInfoJson> =
        serde_json::from_slice(&json).map_err(|e| anyhow::anyhow!("json parse: {e}"))?;
    Ok(rows.into_iter().map(SubInfoJson::into_sub).collect())
}

/// Write a JSON + zstd database file atomically (temp + rename).
pub fn write_db_json_zst(path: &Path, subs: &[SubInfo]) -> std::io::Result<()> {
    let rows: Vec<SubInfoJson> = subs.iter().map(SubInfoJson::from_sub).collect();
    let json = serde_json::to_vec(&rows).map_err(io_err)?;
    let compressed = zstd::stream::encode_all(&json[..], 9)?;
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let tmp = path.with_extension("zst.tmp");
    std::fs::write(&tmp, &compressed)?;
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
    std::fs::rename(&tmp, path)
}

/// Sidecar JSON+zstd path for a given db.xml path.
pub fn json_sidecar_path(xml_path: &Path) -> std::path::PathBuf {
    xml_path.with_file_name("db.json.zst")
}

/// Write both formats (JSON+zstd primary, XML for the original tool).
pub fn write_db_both(xml_path: &Path, subs: &[SubInfo], offset: FixedOffset) -> std::io::Result<()> {
    write_db(xml_path, subs, offset)?;
    write_db_json_zst(&json_sidecar_path(xml_path), subs)
}

/// Which format the store was loaded from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbSource {
    JsonZstd,
    Xml,
    XmlMigrated,
    Missing,
}

// ---------------------------------------------------------------------------
// Global database store
// ---------------------------------------------------------------------------

/// All rows currently loaded in memory.
#[derive(Default)]
pub struct DbStore {
    pub subs: Vec<SubInfo>,
}

impl DbStore {
    /// Load the database, preferring JSON+zstd. If only legacy XML exists it
    /// is parsed and migrated to JSON+zstd automatically.
    pub fn load_from_file(&mut self, xml_path: &Path) -> anyhow::Result<(usize, DbSource)> {
        let json_path = json_sidecar_path(xml_path);
        let json_newer = match (
            std::fs::metadata(&json_path).and_then(|m| m.modified()),
            std::fs::metadata(xml_path).and_then(|m| m.modified()),
        ) {
            (Ok(j), Ok(x)) => j >= x,
            (Ok(_), Err(_)) => true,
            _ => false,
        };

        if json_path.exists() && json_newer {
            match read_db_json_zst(&json_path) {
                Ok(subs) if !subs.is_empty() => {
                    self.subs = subs;
                    return Ok((self.subs.len(), DbSource::JsonZstd));
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("[db] json.zst unreadable, falling back to xml: {e}");
                }
            }
        }

        if xml_path.exists() {
            self.subs = read_db(xml_path)?;
            let n = self.subs.len();
            // Migrate to the compressed format while we are here.
            if write_db_json_zst(&json_path, &self.subs).is_ok() {
                return Ok((n, DbSource::XmlMigrated));
            }
            return Ok((n, DbSource::Xml));
        }

        self.subs.clear();
        Ok((0, DbSource::Missing))
    }

    pub fn save_to_file(&self, path: &Path, offset: FixedOffset) -> std::io::Result<()> {
        write_db_both(path, &self.subs, offset)
    }
}

/// Shared handle so background threads (scan/download) can swap the data.
#[derive(Clone, Default)]
pub struct SharedDb(pub std::sync::Arc<RwLock<DbStore>>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_real_db() {
        // Optional integration data: set SUBATLAS_TEST_DB to a real db.xml
        // (or place one at ./Files/db.xml). Skipped when absent.
        let src = std::env::var("SUBATLAS_TEST_DB")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("Files/db.xml"));
        let src = src.as_path();
        if !src.exists() {
            eprintln!("real db.xml not present; skipped");
            return;
        }
        let subs = read_db(src).expect("parse real db.xml");
        assert!(subs.len() > 4000, "expected thousands of rows, got {}", subs.len());
        let first = &subs[0];
        assert_eq!(first.name_chs, "明日之丈（第一部）");
        assert!(first.path.starts_with("animation\\"));
        // write it back out and re-read; row count must be identical
        let tmp = std::env::temp_dir().join("subdb_roundtrip.xml");
        let offset = chrono::FixedOffset::east_opt(8 * 3600).unwrap();
        write_db(&tmp, &subs, offset).expect("write db");
        let subs2 = read_db(&tmp).expect("re-parse written db");
        assert_eq!(subs.len(), subs2.len());
        // spot-check a few fields survived the round trip
        assert_eq!(subs2[0].name_jp, first.name_jp);
        assert_eq!(subs2[0].time, first.time);
        assert_eq!(subs[100].name_chs, subs2[100].name_chs);
        std::fs::remove_file(&tmp).ok();

        // JSON + zstd round trip must preserve every row.
        let zst = std::env::temp_dir().join("subdb_roundtrip.json.zst");
        write_db_json_zst(&zst, &subs).expect("write json.zst");
        let subs3 = read_db_json_zst(&zst).expect("read json.zst");
        assert_eq!(subs3.len(), subs.len());
        for i in [0, 1, 100, subs.len() - 1] {
            assert_eq!(subs[i].time, subs3[i].time);
            assert_eq!(subs[i].name_chs, subs3[i].name_chs);
            assert_eq!(subs[i].name_jp, subs3[i].name_jp);
            assert_eq!(subs[i].path, subs3[i].path);
            assert_eq!(subs[i].r#type, subs3[i].r#type);
        }
        let zst_size = std::fs::metadata(&zst).unwrap().len();
        println!("json.zst size: {zst_size} bytes (xml {src_size} bytes)", src_size = std::fs::metadata(src).unwrap().len());
        assert!(zst_size < std::fs::metadata(src).unwrap().len());
        std::fs::remove_file(&zst).ok();
    }
}

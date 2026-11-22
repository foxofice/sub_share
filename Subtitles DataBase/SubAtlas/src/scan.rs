// Local directory scan that rebuilds db.xml, mirroring the original
// frm_UpdateDatabase.scan_thread logic (type\year\(date)name folder layout).
use crate::db::{SubInfo, COLUMNS};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub struct ScanParams {
    pub subs_path: String,
    /// None = scan all types; Some = only these type folder names.
    pub types: Option<Vec<String>>,
    /// None = scan all years; Some = only these years.
    pub years: Option<Vec<i32>>,
}

#[derive(Debug, Clone, Default)]
pub struct ScanProgress {
    pub current_path: String,
    pub done: usize,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub scanned: usize,
    pub total: usize,
    pub elapsed: std::time::Duration,
    pub warnings: Vec<String>,
    pub subs: Vec<SubInfo>,
}

fn read_kv_file(path: &Path) -> Vec<(String, String)> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut pairs = Vec::new();
    for line in text.lines() {
        if line.len() < 2 || line.starts_with("//") {
            continue;
        }
        let Some(idx) = line.find(':') else { continue };
        pairs.push((line[..idx].trim().to_string(), line[idx + 1..].trim().to_string()));
    }
    pairs
}

/// Walk the subs tree and collect SubInfo rows (thread-safe stop flag).
pub fn run_scan(
    params: &ScanParams,
    stop: &AtomicBool,
    progress: &Arc<std::sync::Mutex<ScanProgress>>,
    mut warn: impl FnMut(String),
) -> anyhow::Result<ScanResult> {
    let started = Instant::now();
    let root = PathBuf::from(&params.subs_path);
    if !root.is_dir() {
        anyhow::bail!("subs path not found: {}", params.subs_path);
    }

    // type\year\(date)movie\source\sub
    struct MovieDir {
        path: PathBuf,
        type_name: String,
        year: i32,
    }
    let mut movie_dirs: Vec<MovieDir> = Vec::new();
    for type_dir in std::fs::read_dir(&root)?.flatten() {
        let type_path = type_dir.path();
        if !type_path.is_dir() {
            continue;
        }
        let Some(type_name) = type_path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if let Some(types) = &params.types {
            if !types.iter().any(|t| t.eq_ignore_ascii_case(type_name)) {
                continue;
            }
        }
        for year_dir in std::fs::read_dir(&type_path)?.flatten() {
            let year_path = year_dir.path();
            if !year_path.is_dir() {
                continue;
            }
            let Some(year_name) = year_path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let Ok(year) = year_name.parse::<i32>() else {
                warn(format!("skipping non-year directory: {}\\{}", type_name, year_name));
                continue;
            };
            if let Some(years) = &params.years {
                if !years.contains(&year) {
                    continue;
                }
            }
            for movie in std::fs::read_dir(&year_path)?.flatten() {
                let movie_path = movie.path();
                if movie_path.is_dir() {
                    movie_dirs.push(MovieDir {
                        path: movie_path,
                        type_name: type_name.to_string(),
                        year,
                    });
                }
            }
        }
    }

    let total = movie_dirs.len();
    let mut subs: Vec<SubInfo> = Vec::new();
    let mut sources: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for (i, movie) in movie_dirs.iter().enumerate() {
        if stop.load(Ordering::Relaxed) {
            return Ok(ScanResult {
                scanned: subs.len(),
                total,
                elapsed: started.elapsed(),
                warnings: Vec::new(),
                subs: sort_subs(subs),
            });
        }
        let Some(movie_name) = movie.path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        // "(1970.4.1)明日之丈" -> 1970-04-01
        let date = parse_movie_date(movie_name);
        {
            let mut p = progress.lock().unwrap();
            p.current_path = movie_name.to_string();
            p.done = i + 1;
            p.total = total;
        }

        let mut info = SubInfo {
            time: date,
            r#type: movie.type_name.clone(),
            ..Default::default()
        };
        let info_txt = movie.path.join("info.txt");
        if !info_txt.exists() {
            warn(format!("[Warning] missing {}", info_txt.display()));
        } else {
            for (k, v) in read_kv_file(&info_txt) {
                match k.to_ascii_lowercase().as_str() {
                    "chs" => info.name_chs = v,
                    "cht" => info.name_cht = v,
                    "jp" => info.name_jp = v,
                    "en" => info.name_en = v,
                    "rome" => info.name_rome = v,
                    _ => {}
                }
            }
        }

        // source\sub directories
        for source_dir in std::fs::read_dir(&movie.path)?.flatten() {
            let source_path = source_dir.path();
            if !source_path.is_dir() {
                continue;
            }
            let Some(source_name) = source_path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            sources.insert(source_name.to_string());
            for sub_dir in std::fs::read_dir(&source_path)?.flatten() {
                let sub_path = sub_dir.path();
                if !sub_path.is_dir() {
                    continue;
                }
                let Some(sub_name) = sub_path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };

                let mut extensions: std::collections::BTreeSet<String> =
                    std::collections::BTreeSet::new();
                collect_extensions(&sub_path, &mut extensions, 0);

                let mut providers = String::new();
                let mut desc = String::new();
                let sidecar = PathBuf::from(format!("{}.txt", sub_path.display()));
                if sidecar.exists() {
                    let text = std::fs::read_to_string(&sidecar).unwrap_or_default();
                    if let Some(pos) = text.to_lowercase().find("desc:") {
                        desc = text[pos + 5..].trim().to_string();
                    }
                    for (k, v) in read_kv_file(&sidecar) {
                        if k.eq_ignore_ascii_case("providers") {
                            providers = v;
                            break;
                        }
                    }
                }

                let rel = movie
                    .path
                    .strip_prefix(&root)
                    .map(|p| p.to_string_lossy().replace('/', "\\"))
                    .unwrap_or_default();
                let rel = rel.trim_start_matches(['\\', '/']).to_string();

                subs.push(SubInfo {
                    time: date,
                    name_chs: info.name_chs.clone(),
                    name_cht: info.name_cht.clone(),
                    name_jp: info.name_jp.clone(),
                    name_en: info.name_en.clone(),
                    name_rome: info.name_rome.clone(),
                    r#type: movie.type_name.clone(),
                    source: source_name.to_string(),
                    sub_name: sub_name.to_string(),
                    extension: extensions.into_iter().collect::<Vec<_>>().join(";"),
                    providers,
                    desc,
                    path: rel,
                });
            }
        }
    }

    let scanned = subs.len();
    let subs = sort_subs(subs);
    Ok(ScanResult {
        scanned,
        total,
        elapsed: started.elapsed(),
        warnings: Vec::new(),
        subs,
    })
}

fn sort_subs(mut subs: Vec<SubInfo>) -> Vec<SubInfo> {
    // Natural sort by path, then source, then sub_name (like StrCmpLogicalW).
    subs.sort_by(|a, b| {
        natural_cmp(&a.path, &b.path)
            .then_with(|| natural_cmp(&a.source, &b.source))
            .then_with(|| natural_cmp(&a.sub_name, &b.sub_name))
    });
    subs
}

/// Windows Explorer-style natural comparison (digits compare by value).
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let (mut ai, mut bi) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (ai.peek().copied(), bi.peek().copied()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(x), Some(y)) => {
                if x.is_ascii_digit() && y.is_ascii_digit() {
                    let mut ax = String::new();
                    while let Some(c) = ai.peek() {
                        if c.is_ascii_digit() {
                            ax.push(*c);
                            ai.next();
                        } else {
                            break;
                        }
                    }
                    let mut bx = String::new();
                    while let Some(c) = bi.peek() {
                        if c.is_ascii_digit() {
                            bx.push(*c);
                            bi.next();
                        } else {
                            break;
                        }
                    }
                    let nx = ax.trim_start_matches('0').parse::<u64>().unwrap_or(u64::MAX);
                    let ny = bx.trim_start_matches('0').parse::<u64>().unwrap_or(u64::MAX);
                    if nx != ny {
                        return nx.cmp(&ny);
                    }
                } else {
                    let cx = x.to_lowercase().next().unwrap_or(x);
                    let cy = y.to_lowercase().next().unwrap_or(y);
                    // Punctuation/other sort order differences are acceptable here.
                    if cx != cy {
                        return cx.cmp(&cy);
                    }
                    ai.next();
                    bi.next();
                }
            }
        }
    }
}

fn parse_movie_date(movie_name: &str) -> chrono::NaiveDate {
    // "(1970.4.1)..." or fallback 1970-01-01
    let name = movie_name.trim_start_matches(['(', '（']);
    let end = name.find([')', '）']).unwrap_or(0);
    let head = &name[..end];
    let parts: Vec<&str> = head.split('.').collect();
    let year: i32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(1970);
    let month: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let day: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    chrono::NaiveDate::from_ymd_opt(year, month, day)
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
}

fn collect_extensions(dir: &Path, out: &mut std::collections::BTreeSet<String>, depth: u8) {
    if depth > 4 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_extensions(&p, out, depth + 1);
        } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            out.insert(format!(".{}", ext.to_lowercase()));
        }
    }
}

/// Rebuild the db.xml from the scanned rows, then load it back into memory.
pub fn write_scan_result(
    subs: Vec<SubInfo>,
    db_path: &Path,
    offset: chrono::FixedOffset,
) -> anyhow::Result<usize> {
    // JSON + zstd is the primary format; XML is kept for ecosystem compat.
    crate::db::write_db_both(db_path, &subs, offset)?;
    Ok(subs.len())
}

// Re-exported so UI code can build the type list the same way.
pub fn local_types(subs_path: &str) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(Path::new(subs_path)) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();
    names
}

/// Column order helper reused by UI (kept close to the original enum order).
pub fn column_order() -> [crate::db::Column; 13] {
    COLUMNS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_movie_date() {
        assert_eq!(
            parse_movie_date("(1970.4.1)明日之丈"),
            chrono::NaiveDate::from_ymd_opt(1970, 4, 1).unwrap()
        );
        assert_eq!(
            parse_movie_date("(1971.10.24)鲁邦三世"),
            chrono::NaiveDate::from_ymd_opt(1971, 10, 24).unwrap()
        );
        assert_eq!(
            parse_movie_date("(2013.4)进击的巨人"),
            chrono::NaiveDate::from_ymd_opt(2013, 4, 1).unwrap()
        );
    }

    #[test]
    fn natural_order() {
        use std::cmp::Ordering::*;
        assert_eq!(natural_cmp("2", "10"), Less);
        assert_eq!(natural_cmp("BD", "BD-BOX"), Less);
        assert_eq!(natural_cmp("abc", "ABD"), Less);
    }
}

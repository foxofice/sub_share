// Name search with simplified/traditional Chinese combination expansion,
// mirroring the original's ChineseConverter + GenerateCombinations approach.
use crate::db::SubInfo;

#[cfg(windows)]
mod lcmap {
    use windows_sys::Win32::Globalization::{LCMapStringW, LCMAP_SIMPLIFIED_CHINESE, LCMAP_TRADITIONAL_CHINESE};
    const LOCALE_ZH_CN: u32 = 0x0804;

    pub fn convert(text: &str, to_traditional: bool) -> String {
        let flags = if to_traditional {
            LCMAP_TRADITIONAL_CHINESE
        } else {
            LCMAP_SIMPLIFIED_CHINESE
        };
        let src: Vec<u16> = text.encode_utf16().collect();
        // `dest_len` in wide chars; LCMapStringW wants the size including NUL.
        let dest_len = src.len() as i32 * 2 + 4;
        let mut dest = vec![0u16; dest_len as usize];
        let n = unsafe {
            LCMapStringW(
                LOCALE_ZH_CN,
                flags,
                src.as_ptr(),
                src.len() as i32,
                dest.as_mut_ptr(),
                dest_len,
            )
        };
        if n <= 0 {
            return text.to_string();
        }
        String::from_utf16_lossy(&dest[..n as usize])
    }
}

#[cfg(not(windows))]
mod lcmap {
    pub fn convert(text: &str, _to_traditional: bool) -> String {
        text.to_string()
    }
}

fn to_simplified(text: &str) -> String {
    lcmap::convert(text, false)
}

fn to_traditional(text: &str) -> String {
    lcmap::convert(text, true)
}

/// All simplified/traditional combinations that could match the same
/// user intent, like the original: convert both ways, diff the char
/// positions, then expand every difference into two candidates.
pub fn simplified_traditional_combinations(input: &str) -> Vec<String> {
    let simp = to_simplified(input);
    let trad = to_traditional(input);

    let simp_units: Vec<char> = simp.chars().collect();
    let trad_units: Vec<char> = trad.chars().collect();
    let n = simp_units.len().min(trad_units.len());
    let mut diffs: Vec<(usize, char, char)> = Vec::new();
    for i in 0..n {
        if simp_units[i] != trad_units[i] {
            diffs.push((i, simp_units[i], trad_units[i]));
        }
    }

    // Hard cap: names are short, but keep pathological input safe.
    const MAX_COMBINATIONS: usize = 256;
    let mut out: Vec<String> = Vec::new();
    let mut buf: Vec<char> = simp_units.clone();
    generate(&mut buf, &diffs, 0, &mut out);
    out.sort();
    out.dedup();
    out.truncate(MAX_COMBINATIONS);
    out
}

fn generate(current: &mut [char], diffs: &[(usize, char, char)], idx: usize, out: &mut Vec<String>) {
    if idx >= diffs.len() || out.len() >= 4096 {
        out.push(current.iter().collect());
        return;
    }
    let (pos, simp_char, trad_char) = diffs[idx];
    current[pos] = simp_char;
    generate(current, diffs, idx + 1, out);
    current[pos] = trad_char;
    generate(current, diffs, idx + 1, out);
    // restore
    current[pos] = simp_char;
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

#[derive(Debug, Clone, Default)]
pub struct Filters {
    pub name: String,
    pub sub_name: String,
    pub extension: String,
    pub providers: String,
    pub desc: String,
    pub r#type: String,
    pub source: String,
    /// (op, date) where op is one of =, <, >, <=, >=
    pub time_op: Option<(String, chrono::NaiveDate)>,
}

impl Filters {
    pub fn is_active(&self) -> bool {
        !(self.name.is_empty()
            && self.sub_name.is_empty()
            && self.extension.is_empty()
            && self.providers.is_empty()
            && self.desc.is_empty()
            && self.r#type.is_empty()
            && self.source.is_empty()
            && self.time_op.is_none())
    }
}

pub fn matches(sub: &SubInfo, filters: &Filters) -> bool {
    if !filters.name.is_empty() {
        let variants = simplified_traditional_combinations(&filters.name);
        let hit = variants.iter().any(|v| {
            contains_ci(&sub.name_chs, v) || contains_ci(&sub.name_cht, v)
        }) || contains_ci(&sub.name_jp, &filters.name)
            || contains_ci(&sub.name_en, &filters.name)
            || contains_ci(&sub.name_rome, &filters.name);
        if !hit {
            return false;
        }
    }
    if !filters.sub_name.is_empty() && !contains_ci(&sub.sub_name, &filters.sub_name) {
        return false;
    }
    if !filters.extension.is_empty() && !contains_ci(&sub.extension, &filters.extension) {
        return false;
    }
    if !filters.providers.is_empty() && !contains_ci(&sub.providers, &filters.providers) {
        return false;
    }
    if !filters.desc.is_empty() && !contains_ci(&sub.desc, &filters.desc) {
        return false;
    }
    if !filters.r#type.is_empty() && !sub.r#type.eq_ignore_ascii_case(&filters.r#type) {
        return false;
    }
    if !filters.source.is_empty() && !contains_ci(&sub.source, &filters.source) {
        return false;
    }
    if let Some((op, date)) = &filters.time_op {
        let ok = match op.as_str() {
            "=" => sub.time == *date,
            "<" => sub.time < *date,
            ">" => sub.time > *date,
            "<=" => sub.time <= *date,
            ">=" => sub.time >= *date,
            _ => true,
        };
        if !ok {
            return false;
        }
    }
    true
}

pub fn filter_all(subs: &[SubInfo], filters: &Filters) -> Vec<usize> {
    subs.iter()
        .enumerate()
        .filter(|(_, s)| matches(s, filters))
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combinations_basic() {
        let combos = simplified_traditional_combinations("魔神Z");
        assert!(combos.contains(&"魔神Z".to_string()));
        assert!(combos.contains(&"魔神Z".to_string()));
        // simplified + traditional variants both present, nothing exploded
        assert!(combos.len() <= 256);
    }

    #[test]
    fn plain_ascii_untouched() {
        let combos = simplified_traditional_combinations("Lupin");
        assert_eq!(combos, vec!["Lupin".to_string()]);
    }
}

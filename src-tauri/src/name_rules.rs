//! Kural tabanli adlandirma: tek bir adin (uzantisiz govdenin) yeni halini uretir.
//! Saf islevler; diske dokunmaz. Plan `name_rules_plan.rs`, uygulama `name_rules_apply.rs`.

use serde::Deserialize;

/// Windows'ta adda bulunamayan karakterler; her zaman cikarilir.
const BAD_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
/// "Parantez ve noktalama isaretlerini kaldir" acikken bosluga cevrilenler.
const PUNCTUATION: &[char] = &['(', ')', '[', ']', '{', '}', '!', ',', ';', '\'', '’', '`', '~', '@', '#', '$', '%', '^', '&', '+', '='];
const SEPARATORS: &[char] = &['-', '_', ' ', '.'];
const RESERVED: &[&str] = &["CON", "PRN", "AUX", "NUL"];

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NameRuleOptions {
    /// "both" | "dirs" | "files"
    pub scope: String,
    pub include_subdirs: bool,
    /// "keep" | "title" | "sentence" | "lower" | "upper"
    pub case: String,
    /// Bosluk yerine: "keep" | "dash" | "underscore" | "none"
    pub space: String,
    /// Turkce harfleri sadelestir (ç→c, ğ→g, ı→i, ö→o, ş→s, ü→u).
    pub ascii: bool,
    pub clean: bool,
    pub seq_dirs: bool,
    pub seq_files: bool,
    pub start: u64,
    pub pad: usize,
    /// Sona tarih: "none" | "my" (11-2026) | "ym" (2026-11); ogenin son degisiklik tarihi.
    pub date: String,
    /// Ad govdesinin en fazla karakter sayisi (uzanti haric); 0 = sinir yok.
    pub max_len: usize,
    #[serde(default)]
    pub exclude_folders: Vec<String>,
}

/// Govdeyi kurallara gore donusturur. `seq`: bu ogeye verilecek sira numarasi (yoksa None).
/// Kurallar adi tumuyle bosaltirsa None doner.
pub fn transform(stem: &str, o: &NameRuleOptions, seq: Option<u64>, mtime: u64) -> Option<String> {
    let mut s = collapse(stem);
    // Yinelenmezlik: eski sira numarasi ve tarih eki once cikarilir; kural ikinci kez uygulaninca `01-01-…` olmaz.
    if seq.is_some() {
        s = strip_number_prefix(&s);
    }
    if o.date != "none" {
        s = strip_date_suffix(&s);
    }
    s = s.chars().filter(|c| !BAD_CHARS.contains(c) && !c.is_control()).collect();
    if o.clean {
        s = collapse(&s.chars().map(|c| if PUNCTUATION.contains(&c) { ' ' } else { c }).collect::<String>());
    }
    if o.ascii {
        s = s.chars().map(ascii_tr).collect();
    }
    s = apply_case(&s, &o.case, !o.ascii);
    let sep = match o.space.as_str() { "underscore" => "_", "keep" => " ", _ => "-" };
    s = match o.space.as_str() {
        "dash" => s.replace(' ', "-"),
        "underscore" => s.replace(' ', "_"),
        "none" => s.replace(' ', ""),
        _ => s,
    };
    let prefix = seq.map(|n| format!("{n:0width$}{sep}", width = o.pad.clamp(1, 6))).unwrap_or_default();
    let date = crate::file_ops::local_date_suffix(mtime); // gg-aa-yyyy
    let suffix = match o.date.as_str() {
        "my" => format!("{sep}{}-{}", &date[3..5], &date[6..]),
        "ym" => format!("{sep}{}-{}", &date[6..], &date[3..5]),
        _ => String::new(),
    };
    if o.max_len > 0 {
        let room = o.max_len.saturating_sub(prefix.chars().count() + suffix.chars().count()).max(1);
        s = s.chars().take(room).collect();
    }
    let s = s.trim_matches(SEPARATORS);
    if s.is_empty() {
        return None;
    }
    Some(format!("{prefix}{s}{suffix}"))
}

/// Windows'un ayirdigi aygit adi mi (CON, NUL, COM1 …)? Uzanti olsa da gecersizdir.
pub fn is_reserved(name: &str) -> bool {
    let base = name.split('.').next().unwrap_or_default().trim().to_uppercase();
    RESERVED.contains(&base.as_str())
        || (base.len() == 4 && (base.starts_with("COM") || base.starts_with("LPT")) && base.ends_with(|c: char| c.is_ascii_digit() && c != '0'))
}

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `01-ad`, `12_ad`, `003 ad` → `ad`. Dort ve ustu basamak (yil gibi) numara sayilmaz.
fn strip_number_prefix(s: &str) -> String {
    let digits = s.chars().take_while(char::is_ascii_digit).count();
    let rest = &s[digits..];
    let body = rest.trim_start_matches(SEPARATORS);
    if (1..=3).contains(&digits) && body.len() < rest.len() && !body.is_empty() { body.to_string() } else { s.to_string() }
}

/// `ad-11-2026`, `ad_2026-11` → `ad`.
fn strip_date_suffix(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < 9 {
        return s.to_string();
    }
    let (head, tail) = chars.split_at(chars.len() - 8);
    let digit = |i: usize| tail[i].is_ascii_digit();
    let month_year = tail[3] == '-' && [1, 2, 4, 5, 6, 7].into_iter().all(digit);
    let year_month = tail[5] == '-' && [1, 2, 3, 4, 6, 7].into_iter().all(digit);
    if ['-', '_', ' '].contains(&tail[0]) && (month_year || year_month) { head.iter().collect() } else { s.to_string() }
}

fn ascii_tr(c: char) -> char {
    match c {
        'ç' => 'c', 'Ç' => 'C', 'ğ' => 'g', 'Ğ' => 'G', 'ı' => 'i', 'İ' => 'I',
        'ö' => 'o', 'Ö' => 'O', 'ş' => 's', 'Ş' => 'S', 'ü' => 'u', 'Ü' => 'U',
        other => other,
    }
}

/// Turkce kurali: i→İ, ı→I, I→ı, İ→i. `turkish` kapaliyken (harfler sadelestirilmisken) duz Latin kurali.
fn upper(c: char, turkish: bool) -> String {
    match c {
        'i' if turkish => "İ".into(),
        _ => c.to_uppercase().collect(),
    }
}

fn lower(c: char, turkish: bool) -> String {
    match c {
        'I' if turkish => "ı".into(),
        'İ' => "i".into(),
        _ => c.to_lowercase().collect(),
    }
}

fn apply_case(s: &str, mode: &str, turkish: bool) -> String {
    let mut out = String::with_capacity(s.len());
    let mut word_start = true;
    for (i, c) in s.chars().enumerate() {
        let up = match mode {
            "upper" => true,
            "lower" => false,
            "sentence" => i == 0,
            "title" => word_start,
            _ => {
                out.push(c);
                continue;
            }
        };
        out.push_str(&if up { upper(c, turkish) } else { lower(c, turkish) });
        word_start = SEPARATORS.contains(&c);
    }
    out
}

#[cfg(test)]
#[path = "name_rules_tests.rs"]
mod tests;

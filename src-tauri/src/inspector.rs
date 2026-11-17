//! Gelen kutusu dosyalarinin ayrintisi: boyut, tarihler, metin dosyalarinda
//! ilk 300 karakter ve (thumbnail.rs ile) kucuk resim yolu.

use serde::Serialize;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Icerik ozeti icin okunacak en fazla bayt ve gosterilecek en fazla karakter.
const PREVIEW_BYTES: usize = 1536;
pub const PREVIEW_CHARS: usize = 300;

/// Ozet cikarilan duz metin turleri. (PDF/docx icin ayri cikarici gerekir.)
const TEXT_EXTS: &[&str] = &["txt", "csv", "md", "log", "json", "xml", "ini", "yaml", "yml", "html", "htm", "tsv"];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDetail {
    pub path: String,
    pub name: String,
    pub ext: String,
    pub size: u64,
    pub group: String,
    /// Gelen kutusuna tasindigi an (gecmisten); bilinmiyorsa None.
    pub arrived_ms: Option<u64>,
    pub created_ms: Option<u64>,
    pub modified_ms: Option<u64>,
    /// Windows'ta cogu zaman guncellenmez; arayuzde "yaklasik" gosterilir.
    pub accessed_ms: Option<u64>,
    pub content_preview: Option<String>,
    /// Gosterilecek resim (kucukse orijinal, buyukse onbellekteki kucuk resim).
    pub thumbnail_path: Option<String>,
    /// En benzer adli dosya ve benzerlik yuzdesi (Jaro-Winkler >= esik).
    pub similar_to: Option<String>,
    pub similarity: Option<u8>,
    /// Icerigi birebir ayni baska dosyanin yolu (`inbox_dupes`).
    pub duplicate_of: Option<String>,
    /// O dosya yedek klasorundeyse goreli klasor adi; gelen kutusundaysa None.
    pub duplicate_in: Option<String>,
    /// Baslangic karari onerisi (`inbox_suggest`).
    pub suggestion: Option<crate::inbox_suggest::Suggestion>,
    /// Izlenen klasorde bekliyor (otomatik tasima kapali); gelen kutusunda degil.
    pub pending: bool,
}

fn ms(t: std::io::Result<SystemTime>) -> Option<u64> {
    t.ok()?.duration_since(UNIX_EPOCH).ok().map(|d| d.as_millis() as u64)
}

pub fn ext_of(path: &Path) -> String {
    path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default()
}

pub fn is_text(ext: &str) -> bool {
    TEXT_EXTS.contains(&ext)
}

/// Bayt dizisinden ilk `max_chars` karakteri alir. UTF-8 karakterinin ortasindan
/// kesilmis son bayt(lar) atilir; gecersiz baytlar yerine � konur. Ikili gorunen
/// icerik (NUL bayti) icin None.
pub fn text_preview(bytes: &[u8], max_chars: usize) -> Option<String> {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    if bytes.contains(&0) {
        return None;
    }
    let valid_end = match std::str::from_utf8(bytes) {
        Ok(_) => bytes.len(),
        // Sondaki yarim karakter (error_len None) atilir; ortadaki bozuk bayt kalir.
        Err(e) if e.error_len().is_none() => e.valid_up_to(),
        Err(_) => bytes.len(),
    };
    let text = String::from_utf8_lossy(&bytes[..valid_end]);
    let collapsed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out: String = collapsed.chars().take(max_chars).collect();
    if collapsed.chars().count() > max_chars {
        out.push('…');
    }
    (!out.is_empty()).then_some(out)
}

fn read_preview(path: &Path) -> Option<String> {
    let mut buf = Vec::with_capacity(PREVIEW_BYTES);
    File::open(path).ok()?.take(PREVIEW_BYTES as u64).read_to_end(&mut buf).ok()?;
    text_preview(&buf, PREVIEW_CHARS)
}

/// Tek dosyanin ayrintisi (kucuk resim ve benzerlik sonradan doldurulur).
pub fn detail_of(path: &Path, group: &str, arrived_ms: Option<u64>) -> Option<FileDetail> {
    let meta = fs::metadata(path).ok()?;
    if !meta.is_file() {
        return None;
    }
    let ext = ext_of(path);
    Some(FileDetail {
        path: path.to_string_lossy().into_owned(),
        name: path.file_name()?.to_string_lossy().into_owned(),
        size: meta.len(),
        group: group.to_string(),
        arrived_ms,
        created_ms: ms(meta.created()),
        modified_ms: ms(meta.modified()),
        accessed_ms: ms(meta.accessed()),
        content_preview: if is_text(&ext) { read_preview(path) } else { None },
        thumbnail_path: None,
        similar_to: None,
        similarity: None,
        duplicate_of: None,
        duplicate_in: None,
        suggestion: None,
        pending: false,
        ext,
    })
}

/// Her dosya icin, adi (uzantisiz, kucuk harf) en cok benzeyen diger dosyayi bulur.
/// Ayni adli dosyalar (`Kira` / `Kira_1`) da benzer sayilir: icerikleri farkli olabilir.
pub fn mark_similar(files: &mut [FileDetail], threshold: f64) {
    let stems: Vec<String> = files
        .iter()
        .map(|f| Path::new(&f.name).file_stem().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default())
        .collect();
    let best: Vec<Option<(usize, f64)>> = (0..files.len())
        .map(|i| {
            (0..files.len())
                .filter(|&j| j != i && !stems[i].is_empty() && !stems[j].is_empty())
                .map(|j| (j, strsim::jaro_winkler(&stems[i], &stems[j])))
                .filter(|&(_, s)| s >= threshold)
                .max_by(|a, b| a.1.total_cmp(&b.1))
        })
        .collect();
    for (i, b) in best.into_iter().enumerate() {
        if let Some((j, score)) = b {
            files[i].similar_to = Some(files[j].name.clone());
            files[i].similarity = Some((score * 100.0).round() as u8);
        }
    }
}

#[cfg(test)]
#[path = "inspector_tests.rs"]
mod tests;

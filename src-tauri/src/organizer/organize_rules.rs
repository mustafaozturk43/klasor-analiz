//! "Türüne Göre Düzenle" kurallari: uzanti -> (kategori klasoru, tur klasoru),
//! gecici/sistem dosyasi tespiti ve korunacak proje klasoru isaretleri.
//! Kurallar kullanici tarafindan duzenlenebilir; bos liste varsayilanlari kullanir.

use crate::models::{normalize_exts, QUARANTINE_DIR};
use crate::organize_models::RuleDef;
use std::collections::HashMap;
use std::path::Path;

pub const DOCS: &str = "Dokümanlar";
pub const IMAGES: &str = "Resimler";
pub const ARCHIVES: &str = "Arşivler";
pub const OTHERS: &str = "Diğer";

/// Varsayilan (kategori klasoru, tur klasoru, uzantilar).
const DEFAULT_RULES: &[(&str, &str, &[&str])] = &[
    (DOCS, "PDF", &["pdf"]),
    (DOCS, "Word", &["doc", "docx", "odt"]),
    (DOCS, "Excel", &["xls", "xlsx"]),
    (DOCS, "PowerPoint", &["ppt", "pptx"]),
    (DOCS, "Metin", &["txt"]),
    (DOCS, "CSV", &["csv"]),
    (IMAGES, "JPG", &["jpg", "jpeg"]),
    (IMAGES, "PNG", &["png"]),
    (IMAGES, "WEBP", &["webp"]),
    (IMAGES, "BMP", &["bmp"]),
    (IMAGES, "TIFF", &["tif", "tiff"]),
    (IMAGES, "GIF", &["gif"]),
    (IMAGES, "HEIC", &["heic"]),
    (ARCHIVES, "ZIP", &["zip"]),
    (ARCHIVES, "RAR", &["rar"]),
    (ARCHIVES, "7Z", &["7z"]),
    (ARCHIVES, "TAR", &["tar", "gz"]),
    (ARCHIVES, "ISO", &["iso"]),
];

/// Bir klasoru yazilim projesi yapan dosya/klasor adlari (kucuk harf).
const PROJECT_MARKERS: &[&str] = &[
    ".git", ".svn", ".hg", "package.json", "cargo.toml", "pyproject.toml", "go.mod",
    "pom.xml", "build.gradle", "composer.json", "node_modules", "gemfile",
];
/// Proje isareti sayilan uzantilar (Visual Studio cozum/proje dosyalari).
const PROJECT_EXTS: &[&str] = &["sln", "csproj", "vcxproj"];

/// Asla tasinmayan sistem dosyalari (kucuk harf).
const SYSTEM_FILES: &[&str] = &["desktop.ini", "thumbs.db", ".ds_store", "ehthumbs.db"];

/// Windows'ta klasor adinda kullanilamayan karakterler.
const BAD_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

pub fn default_rules() -> Vec<RuleDef> {
    DEFAULT_RULES
        .iter()
        .map(|(c, k, exts)| RuleDef {
            category: c.to_string(),
            kind: k.to_string(),
            extensions: exts.iter().map(|e| e.to_string()).collect(),
        })
        .collect()
}

/// Varsayilan kural listesi (kural duzenleyicinin "Varsayılana dön" dugmesi icin).
#[tauri::command]
pub fn default_organize_rules() -> Vec<RuleDef> {
    default_rules()
}

/// Dogrulanmis kural seti: uzanti -> (kategori, tur).
pub struct RuleSet {
    map: HashMap<String, (String, String)>,
}

pub(crate) fn check_folder_name(label: &str, value: &str) -> Result<String, String> {
    let v = value.trim().trim_end_matches('.').to_string();
    if v.is_empty() {
        return Err(format!("{label} adı boş olamaz."));
    }
    if v.contains(BAD_CHARS) {
        return Err(format!("“{v}” klasör adında kullanılamayan karakter var (< > : \" / \\ | ? *)."));
    }
    if v.eq_ignore_ascii_case(QUARANTINE_DIR) {
        return Err(format!("“{QUARANTINE_DIR}” klasör adı olarak kullanılamaz."));
    }
    Ok(v)
}

impl RuleSet {
    /// Kurallari dogrular. Bos liste varsayilanlari kullanir. Ayni uzanti
    /// iki kurala baglanmissa hangi klasore gidecegi belirsiz oldugu icin hata verir.
    pub fn new(defs: &[RuleDef]) -> Result<Self, String> {
        let owned;
        let defs = if defs.is_empty() { owned = default_rules(); &owned } else { defs };
        let mut map: HashMap<String, (String, String)> = HashMap::new();
        for d in defs {
            let category = check_folder_name("Kategori klasörü", &d.category)?;
            let kind = check_folder_name("Tür klasörü", &d.kind)?;
            let exts = normalize_exts(&d.extensions);
            if exts.is_empty() {
                return Err(format!("“{kind}” kuralında en az bir uzantı olmalı."));
            }
            for ext in exts {
                if let Some((_, other)) = map.get(&ext) {
                    return Err(format!("“.{ext}” uzantısı iki kurala bağlı: {other} ve {kind}."));
                }
                map.insert(ext, (category.clone(), kind.clone()));
            }
        }
        Ok(Self { map })
    }

    /// Uzantinin hedef klasorleri. `include_other` ise tanimsiz uzantilar
    /// `Diğer/<UZANTI>` (uzantisizlar `Diğer/Uzantısız`) altina gider.
    pub fn target_of(&self, ext: &str, include_other: bool) -> Option<(String, String)> {
        if let Some(hit) = self.map.get(ext) {
            return Some(hit.clone());
        }
        if !include_other {
            return None;
        }
        let kind = if ext.is_empty() { "Uzantısız".to_string() } else { ext.to_uppercase() };
        Some((OTHERS.to_string(), kind))
    }
}

/// Yil alt klasoru adi (yerel saat, degisiklik tarihinden); tarih yoksa "Tarihsiz".
pub fn year_folder(mtime: u64) -> String {
    if mtime == 0 {
        return "Tarihsiz".into();
    }
    let date = crate::file_ops::local_date_suffix(mtime); // gg-aa-yyyy
    date[date.len() - 4..].to_string()
}

/// Office kilit dosyalari (`~$…`), `.tmp`/`.crdownload` gibi gecici dosyalar ve sistem dosyalari.
pub fn is_temp_or_system(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.starts_with("~$")
        || SYSTEM_FILES.contains(&lower.as_str())
        || [".tmp", ".crdownload", ".part", ".partial"].iter().any(|s| lower.ends_with(s))
}

/// Ad listesinde proje isareti var mi (kucuk harf karsilastirma).
pub fn has_project_marker<'a>(names: impl IntoIterator<Item = &'a str>) -> bool {
    names.into_iter().any(|n| {
        let lower = n.to_lowercase();
        PROJECT_MARKERS.contains(&lower.as_str())
            || Path::new(&lower)
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| PROJECT_EXTS.contains(&e))
    })
}

/// Diskteki klasor proje gibi gorunuyor mu?
pub fn is_project_dir(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .map(|rd| {
            let names: Vec<String> = rd
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect();
            has_project_marker(names.iter().map(String::as_str))
        })
        .unwrap_or(false)
}

#[cfg(test)]
#[path = "organize_rules_tests.rs"]
mod tests;

//! Uygulamanin tek ayar kaynagi: gelen kutusu otomasyonu (kaynak klasorler, yedek klasoru,
//! uzanti gruplari) ve arayuz tercihleri (`ui_prefs`: tema, son klasorler, profiller). Watcher pencere kapaliyken de calisacagi icin ayarlar on
//! yuzde degil, uygulama veri dizinindeki `settings.json` dosyasinda tutulur.

use crate::models::{normalize_exts, INBOX_DIR};
use crate::organize_rules::check_folder_name;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

const FILE_NAME: &str = "settings.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtGroup {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default)]
    pub source_folders: Vec<String>,
    #[serde(default)]
    pub backup_root: String,
    #[serde(default = "default_groups")]
    pub ext_groups: Vec<ExtGroup>,
    /// Kaynak klasorler izlensin mi? Her zaman kapali baslar; kullanici acar.
    #[serde(default)]
    pub automation_enabled: bool,
    /// Hicbir gruba uymayan dosyalar `Diğer` grubuna gitsin mi? (kapaliysa yerinde kalir)
    #[serde(default)]
    pub include_other: bool,
    /// Uygulama acilinca kaynakta bekleyen dosyalar da yerlestirilsin mi?
    #[serde(default)]
    pub scan_pending_on_start: bool,
    /// Uygulama Windows oturumu acilinca baslasin mi? (`startup.rs` baslangic kaydina yansitir)
    #[serde(default)]
    pub start_with_windows: bool,
    /// Otomatik tasimalarda Windows bildirimi gosterilsin mi?
    #[serde(default)]
    pub notify_moves: bool,
    /// Arayuz tercihleri (anahtar -> JSON deger). Yalnizca `set_ui_pref` degistirir;
    /// `save_settings` on yuzden gelen kopyayi yok sayar.
    #[serde(default)]
    pub ui_prefs: BTreeMap<String, serde_json::Value>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            source_folders: Vec::new(),
            backup_root: String::new(),
            ext_groups: default_groups(),
            automation_enabled: false,
            include_other: false,
            scan_pending_on_start: false,
            start_with_windows: false,
            notify_moves: false,
            ui_prefs: BTreeMap::new(),
        }
    }
}

pub const OTHER_GROUP: &str = "Diğer";

pub fn default_groups() -> Vec<ExtGroup> {
    let g = |name: &str, exts: &[&str]| ExtGroup { name: name.into(), extensions: exts.iter().map(|e| e.to_string()).collect() };
    vec![
        g("Dokümanlar", &["pdf", "docx", "doc", "xlsx", "xls", "pptx", "ppt", "txt", "csv", "odt"]),
        g("Görseller", &["jpg", "jpeg", "png", "webp", "bmp", "tif", "tiff", "gif", "heic"]),
        g("Videolar", &["mp4", "mov", "mkv", "avi", "webm"]),
        g("Arşivler", &["zip", "rar", "7z", "tar", "gz", "iso"]),
        g("Kurulum", &["exe", "msi"]),
    ]
}

/// Karsilastirma icin yol anahtari: varsa gercek yol, yoksa yazildigi gibi;
/// Windows'ta buyuk/kucuk harf duyarsiz ve sondaki ayirac olmadan.
fn path_key(p: &str) -> PathBuf {
    let raw = Path::new(p.trim());
    let real = fs::canonicalize(raw).unwrap_or_else(|_| raw.to_path_buf());
    let s = real.to_string_lossy().trim_start_matches(r"\\?\").trim_end_matches(['\\', '/']).to_lowercase();
    PathBuf::from(s)
}

/// `a` ile `b` ayni klasor mu ya da biri digerinin icinde mi?
fn overlaps(a: &Path, b: &Path) -> bool {
    a.starts_with(b) || b.starts_with(a)
}

impl AppSettings {
    /// Ayarlari temizler ve dogrular; kaydedilecek hali dondurur.
    pub fn validated(mut self) -> Result<Self, String> {
        self.backup_root = self.backup_root.trim().to_string();
        let mut seen = Vec::<PathBuf>::new();
        let mut sources = Vec::new();
        for s in self.source_folders.iter().map(|s| s.trim()).filter(|s| !s.is_empty()) {
            let key = path_key(s);
            if !seen.contains(&key) {
                seen.push(key);
                sources.push(s.to_string());
            }
        }
        self.source_folders = sources;

        if !self.backup_root.is_empty() {
            let backup = path_key(&self.backup_root);
            for (src, key) in self.source_folders.iter().zip(&seen) {
                if overlaps(key, &backup) {
                    return Err(format!(
                        "Yedek klasörü ile kaynak klasör iç içe olamaz: {src}. Aksi halde taşınan dosyalar yeniden yakalanır."
                    ));
                }
            }
        }
        if self.automation_enabled && (self.source_folders.is_empty() || self.backup_root.is_empty()) {
            return Err("Otomasyonu açmak için en az bir kaynak klasör ve bir yedek klasörü seçin.".into());
        }

        let mut owner: HashMap<String, String> = HashMap::new();
        let mut names: Vec<String> = Vec::new();
        for g in &mut self.ext_groups {
            g.name = check_folder_name("Grup", &g.name)?;
            if g.name.eq_ignore_ascii_case(INBOX_DIR) || g.name.eq_ignore_ascii_case(OTHER_GROUP) {
                return Err(format!("“{}” grup adı olarak kullanılamaz.", g.name));
            }
            if names.iter().any(|n| n.to_lowercase() == g.name.to_lowercase()) {
                return Err(format!("“{}” adında iki grup var.", g.name));
            }
            names.push(g.name.clone());
            let mut exts = normalize_exts(&g.extensions);
            exts.dedup();
            let mut unique = Vec::new();
            for e in exts {
                if e.contains(['.', ' ', '/', '\\']) {
                    return Err(format!("“{e}” geçerli bir uzantı değil."));
                }
                if let Some(other) = owner.get(&e) {
                    if other != &g.name {
                        return Err(format!("“.{e}” zaten {other} grubunda. Bir uzantı yalnızca bir gruba bağlanabilir."));
                    }
                    continue;
                }
                owner.insert(e.clone(), g.name.clone());
                unique.push(e);
            }
            if unique.is_empty() {
                return Err(format!("{} grubunda en az bir uzantı olmalı.", g.name));
            }
            g.extensions = unique;
        }
        if self.ext_groups.is_empty() {
            return Err("En az bir uzantı grubu olmalı.".into());
        }
        Ok(self)
    }

    /// Uzantinin bagli oldugu grup; yoksa (acik ise) `Diğer`, degilse None.
    pub fn group_of(&self, ext: &str) -> Option<&str> {
        let ext = ext.trim_start_matches('.').to_lowercase();
        self.ext_groups
            .iter()
            .find(|g| g.extensions.contains(&ext))
            .map(|g| g.name.as_str())
            .or(if self.include_other { Some(OTHER_GROUP) } else { None })
    }
}

/// Kayitli ayarlari yukler; dosya yok veya bozuksa varsayilanlar. Bozuk dosyanin bir kopyasi
/// `settings.json.bozuk` olarak saklanir: ilk kayit onu ezmeden once elle kurtarilabilir.
pub fn load(dir: &Path) -> AppSettings {
    let file = dir.join(FILE_NAME);
    let Ok(text) = fs::read_to_string(&file) else { return AppSettings::default() };
    serde_json::from_str(&text).unwrap_or_else(|_| {
        let _ = fs::copy(&file, dir.join(format!("{FILE_NAME}.bozuk")));
        AppSettings::default()
    })
}

/// Ayarlari dogrular ve diske yazar (once gecici dosya, sonra yeniden adlandirma).
pub fn save(dir: &Path, settings: AppSettings) -> Result<AppSettings, String> {
    let clean = settings.validated()?;
    write(dir, &clean)?;
    Ok(clean)
}

/// Ayarlari dogrulamadan, oldugu gibi diske yazar (atomik).
pub fn write(dir: &Path, settings: &AppSettings) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    let tmp = dir.join(format!("{FILE_NAME}.tmp"));
    fs::write(&tmp, json).map_err(|e| e.to_string())?;
    fs::rename(&tmp, dir.join(FILE_NAME)).map_err(|e| e.to_string())
}

/// Uygulama durumunda tutulan ayarlar.
pub struct SettingsState {
    pub settings: std::sync::Mutex<AppSettings>,
    pub dir: PathBuf,
}

impl SettingsState {
    pub fn load(dir: &Path) -> Self {
        Self { settings: std::sync::Mutex::new(load(dir)), dir: dir.to_path_buf() }
    }
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;

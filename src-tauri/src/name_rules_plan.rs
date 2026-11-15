//! Kural tabanli adlandirma plani: secili klasorun icindeki klasor ve dosyalari gezer,
//! her biri icin yeni adi hesaplar. Diske yazmaz. Kok klasorun kendi adi degismez.
//! Karantina, gelen kutusu, gizli/sistem ogeleri ve hariç klasorler hic listelenmez;
//! proje klasorleri ve ayarlarda yolu kayitli klasorler listelenir ama korunur.

use crate::models::{INBOX_DIR, QUARANTINE_DIR};
use crate::name_rules::{is_reserved, transform, NameRuleOptions};
use crate::organize_rules::{is_project_dir, is_temp_or_system};
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_PATH_CHARS: usize = 259;
/// Onizleme tek seferde bu kadar ogeyi asarsa durur (arayuz ve plan boyutu icin).
const MAX_ENTRIES: usize = 20_000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NameEntry {
    pub from: String,
    /// Yeni tam yol; ad degismiyorsa veya oge atlaniyorsa None.
    pub to: Option<String>,
    pub warning: Option<String>,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NamePlan {
    pub id: String,
    pub entries: Vec<NameEntry>,
}

struct Item {
    name: String,
    path: PathBuf,
    is_dir: bool,
    mtime: u64,
}

struct Walk<'a> {
    o: &'a NameRuleOptions,
    excluded: HashSet<String>,
    protected: Vec<String>,
    entries: Vec<NameEntry>,
}

/// `protected`: ayarlarda yolu kayitli klasorler (izlenen kaynaklar, yedek kok). Bunlar ve
/// ust klasorleri yeniden adlandirilmaz; kok bunlardan birinin icindeyse plan reddedilir.
pub fn plan_name_rules(root: &Path, o: &NameRuleOptions, protected: &[String]) -> Result<NamePlan, String> {
    if !root.is_dir() {
        return Err(format!("Klasör bulunamadı: {}", root.display()));
    }
    let protected: Vec<String> = protected.iter().map(|p| norm(Path::new(p))).filter(|p| !p.is_empty()).collect();
    let root_norm = norm(root);
    if protected.iter().any(|p| within(p, &root_norm)) {
        return Err("Bu klasör Gelen Kutusu otomasyonunun izlediği veya yedeklediği bir klasörün içinde; burada adlandırma yapılmaz.".into());
    }
    let mut excluded: HashSet<String> = o.exclude_folders.iter().map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
    excluded.extend([QUARANTINE_DIR.to_lowercase(), INBOX_DIR.to_lowercase()]);
    let mut walk = Walk { o, excluded, protected, entries: Vec::new() };
    walk.dir(root, root)?;
    let id = format!("NR-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos());
    Ok(NamePlan { id, entries: walk.entries })
}

impl Walk<'_> {
    /// `dir`: klasorun bugunku yolu; `new_dir`: ust klasorler adlandirildiktan sonraki yolu (uzunluk denetimi icin).
    fn dir(&mut self, dir: &Path, new_dir: &Path) -> Result<(), String> {
        let items = self.list(dir)?;
        // Bu klasordeki butun adlar dolu sayilir: uygulama sirasi ne olursa olsun kimse baskasinin adina tasinmaz.
        let mut used: HashSet<String> = items.iter().map(|i| i.name.to_lowercase()).collect();
        let (mut next_dir, mut next_file) = (self.o.start, self.o.start);
        for it in items {
            if self.entries.len() >= MAX_ENTRIES {
                return Err(format!("{MAX_ENTRIES} öğeden fazlası var; daha küçük bir klasör seçin veya alt klasörleri kapatın."));
            }
            let mut entry = NameEntry { from: it.path.to_string_lossy().into_owned(), to: None, warning: None, is_dir: it.is_dir };
            let mut final_name = it.name.clone();
            let guarded = it.is_dir.then(|| self.guard(&it.path)).flatten();
            let in_scope = self.o.scope == "both" || (self.o.scope == "dirs") == it.is_dir;
            if let Some(reason) = guarded {
                entry.warning = Some(reason.into());
            } else if in_scope {
                let counter = if it.is_dir { &mut next_dir } else { &mut next_file };
                let seq = (if it.is_dir { self.o.seq_dirs } else { self.o.seq_files }).then(|| {
                    *counter += 1;
                    *counter - 1
                });
                match self.rename(&it, seq, new_dir, &mut used) {
                    Ok(Some((name, numbered))) => {
                        entry.to = Some(dir.join(&name).to_string_lossy().into_owned());
                        entry.warning = numbered.then(|| "Bu ad dolu; numara eklendi.".to_string());
                        final_name = name;
                    }
                    Ok(None) => {}
                    Err(warning) => entry.warning = Some(warning),
                }
            }
            self.entries.push(entry);
            if it.is_dir && self.o.include_subdirs && guarded.is_none() {
                self.dir(&it.path, &new_dir.join(&final_name))?;
            }
        }
        Ok(())
    }

    /// Yeni adi ve cakisma numarasi eklenip eklenmedigini dondurur; ad degismiyorsa Ok(None), oge atlanacaksa Err(neden).
    fn rename(&self, it: &Item, seq: Option<u64>, new_dir: &Path, used: &mut HashSet<String>) -> Result<Option<(String, bool)>, String> {
        let (stem, ext) = split(&it.name, it.is_dir);
        let new_stem = transform(stem, self.o, seq, it.mtime).ok_or("Kurallar bu ad için boş sonuç veriyor; atlanacak.")?;
        let wanted = format!("{new_stem}{ext}");
        if wanted == it.name {
            return Ok(None);
        }
        if is_reserved(&wanted) {
            return Err("Windows'un ayırdığı bir ad (CON, NUL, COM1 …); atlanacak.".into());
        }
        let own = it.name.to_lowercase();
        let mut candidate = wanted;
        let mut n = 1;
        // Yalnizca harf degisimi kendi adiyla cakismaz; baskasinin adiyla cakisirsa numaralanir.
        while candidate.to_lowercase() != own && used.contains(&candidate.to_lowercase()) {
            candidate = format!("{new_stem} ({n}){ext}");
            n += 1;
        }
        if new_dir.join(&candidate).to_string_lossy().chars().count() > MAX_PATH_CHARS {
            return Err("Yeni yol 260 karakteri aşıyor; atlanacak.".into());
        }
        used.insert(candidate.to_lowercase());
        Ok(Some((candidate, n > 1)))
    }

    /// Klasor korunuyorsa nedenini dondurur; korunan klasorun icine de girilmez.
    fn guard(&self, path: &Path) -> Option<&'static str> {
        let here = norm(path);
        if self.protected.iter().any(|p| within(&here, p)) {
            return Some("Ayarlarda yolu kayıtlı (izlenen veya yedek) klasör; korunur.");
        }
        is_project_dir(path).then_some("Proje klasörü; içindekilerle birlikte korunur.")
    }

    /// Klasorun dogrudan icerigi: once klasorler, sonra dosyalar; ada gore.
    fn list(&self, dir: &Path) -> Result<Vec<Item>, String> {
        let mut items = Vec::new();
        for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let Ok(entry) = entry else { continue };
            let (Ok(kind), Ok(meta)) = (entry.file_type(), entry.metadata()) else { continue };
            let Some(name) = entry.file_name().to_str().map(str::to_string) else { continue };
            let is_dir = kind.is_dir();
            if kind.is_symlink() || name.starts_with('.') || hidden(&meta) || (is_dir && self.excluded.contains(&name.to_lowercase())) || (!is_dir && is_temp_or_system(&name)) {
                continue;
            }
            let mtime = meta.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
            items.push(Item { name, path: entry.path(), is_dir, mtime });
        }
        items.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
        Ok(items)
    }
}

/// Dosyada govde ve uzanti (`.pdf`); klasorde adin tamami govdedir. `.gizli` gibi adlarda uzanti yoktur.
fn split(name: &str, is_dir: bool) -> (&str, &str) {
    match name.rfind('.') {
        Some(dot) if !is_dir && dot > 0 => name.split_at(dot),
        _ => (name, ""),
    }
}

#[cfg(windows)]
fn hidden(meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    meta.file_attributes() & 0x6 != 0 // FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM
}

#[cfg(not(windows))]
fn hidden(_meta: &fs::Metadata) -> bool {
    false
}

/// Karsilastirma icin yol: kucuk harf, ters egik cizgi, sonda ayirici yok.
fn norm(path: &Path) -> String {
    path.to_string_lossy().to_lowercase().replace('/', "\\").trim_end_matches('\\').to_string()
}

/// `outer`, `inner` ile ayni mi veya onun ust klasoru mu?
fn within(outer: &str, inner: &str) -> bool {
    inner == outer || inner.strip_prefix(outer).is_some_and(|rest| rest.starts_with('\\'))
}

#[cfg(test)]
#[path = "name_rules_plan_tests.rs"]
mod tests;

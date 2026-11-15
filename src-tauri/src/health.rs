//! Klasor sagligi raporu: en buyuk dosyalar, en uzun suredir degismeyenler,
//! 0 baytlik dosyalar ve bos klasorler. Yalnizca metadata okunur (hash yok).

use crate::models::QUARANTINE_DIR;
use crate::organize_cleanup::{is_quarantine_path, prune_empty_dirs};
use crate::organize_rules::is_temp_or_system;
use crate::path_guard::{ensure_inside, in_quarantine};
use crate::scanner::walker;
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Her listede gosterilecek en fazla kayit.
const TOP_N: usize = 200;
/// Bos dosya/klasor listelerinin ust siniri (dev klasorlerde arayuzu korur).
const MAX_LIST: usize = 5000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthFile {
    pub path: String,
    pub size: u64,
    pub mtime: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    pub root_path: String,
    pub total_files: u64,
    pub total_bytes: u64,
    pub largest: Vec<HealthFile>,
    pub oldest: Vec<HealthFile>,
    pub empty_files: Vec<HealthFile>,
    pub empty_dirs: Vec<String>,
    pub warnings: Vec<String>,
}

#[tauri::command(async)]
pub fn folder_health(root_path: String, include_subdirs: bool, exclude_folders: Vec<String>) -> Result<HealthReport, String> {
    build(Path::new(&root_path), include_subdirs, &exclude_folders)
}

/// Secili bos klasorleri ve bu yuzden bosalan ust klasorlerini kaldirir.
/// Dolu klasorler (arada dosya eklenmis olsa bile) silinmez.
#[tauri::command(async)]
pub fn remove_empty_folders(root_path: String, paths: Vec<String>) -> Result<u64, String> {
    let root = PathBuf::from(&root_path);
    let mut dirs = Vec::new();
    for p in &paths {
        let dir = PathBuf::from(p);
        ensure_inside(&root, &dir)?;
        if in_quarantine(&root, &dir) {
            return Err("Karantina klasörleri bu ekrandan kaldırılamaz.".into());
        }
        dirs.push(dir);
    }
    Ok(prune_empty_dirs(&root, dirs, &|d| is_quarantine_path(&root, d)))
}

pub fn build(root: &Path, include_subdirs: bool, exclude_folders: &[String]) -> Result<HealthReport, String> {
    if !root.is_dir() {
        return Err(format!("Klasör bulunamadı: {}", root.display()));
    }
    let mut excluded: HashSet<String> = exclude_folders.iter().map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
    excluded.insert(QUARANTINE_DIR.to_lowercase());

    let empties = Arc::new(Mutex::new(Vec::<String>::new()));
    let shared = Arc::clone(&empties);
    let root_buf = root.to_path_buf();
    let walk = walker(root)
        .skip_hidden(false)
        .max_depth(if include_subdirs { usize::MAX } else { 1 })
        .process_read_dir(move |_depth, dir, _state, children| {
            if dir != root_buf.as_path()
                && children.is_empty()
                && let Ok(mut list) = shared.lock()
            {
                list.push(dir.to_string_lossy().into_owned());
            }
            children.retain(|c| match c {
                Ok(e) => !(e.file_type.is_dir() && excluded.contains(&e.file_name.to_string_lossy().to_lowercase())),
                Err(_) => true,
            });
        });

    let mut files: Vec<HealthFile> = Vec::new();
    let mut empty_files = Vec::new();
    let mut warnings = Vec::new();
    let mut shallow_empty = Vec::new();
    for entry in walk {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => { warnings.push(e.to_string()); continue; }
        };
        let path = entry.path();
        if entry.file_type().is_dir() {
            // Alt klasorler taranmiyorsa 1. duzey klasorlerin bosluguna dogrudan bakilir.
            if !include_subdirs && entry.depth() == 1 && std::fs::read_dir(&path).is_ok_and(|mut rd| rd.next().is_none()) {
                shallow_empty.push(path.to_string_lossy().into_owned());
            }
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            warnings.push(format!("Okunamadı: {}", path.display()));
            continue;
        };
        let mtime = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
        let item = HealthFile { path: path.to_string_lossy().into_owned(), size: meta.len(), mtime };
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        if item.size == 0 && !is_temp_or_system(&name) && empty_files.len() < MAX_LIST {
            empty_files.push(item.clone());
        }
        files.push(item);
    }

    let total_files = files.len() as u64;
    let total_bytes = files.iter().map(|f| f.size).sum();
    let mut largest = files.clone();
    largest.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.path.cmp(&b.path)));
    largest.truncate(TOP_N);
    let mut oldest: Vec<HealthFile> = files.into_iter().filter(|f| f.mtime > 0).collect();
    oldest.sort_by(|a, b| a.mtime.cmp(&b.mtime).then_with(|| a.path.cmp(&b.path)));
    oldest.truncate(TOP_N);
    empty_files.sort_by(|a, b| a.path.cmp(&b.path));

    let mut empty_dirs = empties.lock().map(|l| l.clone()).unwrap_or_default();
    empty_dirs.extend(shallow_empty);
    empty_dirs.sort();
    empty_dirs.dedup();
    empty_dirs.truncate(MAX_LIST);

    Ok(HealthReport { root_path: root.to_string_lossy().into_owned(), total_files, total_bytes, largest, oldest, empty_files, empty_dirs, warnings })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn reports_largest_oldest_empty_files_and_dirs() {
        let root = std::env::temp_dir().join(format!("sfo_health_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("bos/ic")).unwrap();
        fs::create_dir_all(root.join(QUARANTINE_DIR).join("x")).unwrap();
        fs::write(root.join("buyuk.bin"), vec![0u8; 5000]).unwrap();
        fs::write(root.join("kucuk.txt"), b"abc").unwrap();
        fs::write(root.join("sifir.txt"), b"").unwrap();
        fs::write(root.join("desktop.ini"), b"").unwrap();
        let old = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000_000);
        fs::File::options().write(true).open(root.join("kucuk.txt")).unwrap().set_modified(old).unwrap();

        let r = build(&root, true, &[]).unwrap();
        assert_eq!(r.total_files, 4);
        assert!(r.largest[0].path.ends_with("buyuk.bin"));
        assert!(r.oldest[0].path.ends_with("kucuk.txt"));
        assert_eq!(r.empty_files.len(), 1, "desktop.ini sayilmaz");
        assert_eq!(r.empty_dirs, vec![root.join("bos").join("ic").to_string_lossy().into_owned()], "karantina listelenmez");

        let removed = remove_empty_folders(root.to_string_lossy().into_owned(), r.empty_dirs.clone()).unwrap();
        assert_eq!(removed, 2, "bos/ic ve bosalan bos");
        assert!(remove_empty_folders(root.to_string_lossy().into_owned(), vec![root.join(QUARANTINE_DIR).join("x").to_string_lossy().into_owned()]).is_err());
        fs::remove_dir_all(&root).unwrap();
    }
}

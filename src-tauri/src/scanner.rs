//! jwalk ile paralel dizin tarama + kategori filtreleme.

use crate::models::{FileCategory, FileItem, FolderFile, INBOX_DIR, QUARANTINE_DIR};
use std::collections::HashSet;
use std::path::Path;

const DOCUMENT_EXTS: &[&str] = &["pdf", "docx", "doc", "xlsx", "xls", "pptx", "ppt", "txt", "csv", "odt"];
const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "tif", "tiff", "gif", "heic"];
const ARCHIVE_EXTS: &[&str] = &["zip", "rar", "7z", "tar", "gz", "iso"];

/// Her yuruyus icin sinirli ve bagimsiz havuz: cagiran rayon iscisini bekleyerek
/// ayni havuzu tuketmez; eszamanli analiz/hash islerinden etkilenmez.
pub(crate) fn walker(root: &Path) -> jwalk::WalkDir {
    let threads = std::thread::available_parallelism().map(|n| n.get().min(4)).unwrap_or(2);
    jwalk::WalkDir::new(root).parallelism(jwalk::Parallelism::RayonNewPool(threads))
}

/// Uzanti yasakli listede mi (normalize edilmis `banned` beklenir)?
fn is_banned(ext: &str, banned: &[String]) -> bool {
    !banned.is_empty() && banned.iter().any(|b| b == ext)
}

/// Uzantiyi hedef kategoriye esler; eslesmezse None dondurur.
pub fn category_of(ext: &str) -> Option<FileCategory> {
    if DOCUMENT_EXTS.contains(&ext) {
        Some(FileCategory::Document)
    } else if IMAGE_EXTS.contains(&ext) {
        Some(FileCategory::Image)
    } else if ARCHIVE_EXTS.contains(&ext) {
        Some(FileCategory::Archive)
    } else {
        None
    }
}

/// Yol (root'a gore) atlanacak bir klasorun icinde mi? `reserved`: ayrilmis klasor adlari.
fn is_excluded(path: &Path, root: &Path, excluded: &HashSet<String>, reserved: &[&str]) -> bool {
    path.strip_prefix(root)
        .map(|rel| {
            rel.components().any(|c| {
                let name = c.as_os_str().to_string_lossy().to_lowercase();
                reserved.iter().any(|r| name == r.to_lowercase()) || excluded.contains(&name)
            })
        })
        .unwrap_or(false)
}

/// Tarama ciktisi: dosyalar ve okunamadigi icin atlanan girdilerin uyarilari.
pub struct ScanOutput {
    pub files: Vec<FileItem>,
    pub warnings: Vec<String>,
}

/// `scan_reporting` ile ayni; uyarilari atar.
pub fn scan(
    root: &Path, include_subdirs: bool, exclude_folders: &[String], banned: &[String], include_other: bool,
    on_progress: &(dyn Fn(u64) + Sync), is_cancelled: &(dyn Fn() -> bool + Sync),
) -> Result<Vec<FileItem>, String> {
    scan_reporting(root, include_subdirs, exclude_folders, banned, include_other, on_progress, is_cancelled).map(|o| o.files)
}

/// Root klasorunu tarar, hedef kategorideki dosyalari FileItem olarak dondurur.
///
/// - `include_subdirs = false` ise sadece root'un direkt dosyalari taranir.
/// - `exclude_folders` (buyuk/kucuk harf duyarsiz), `_Kopyalar_Karantina` ve
///   `_GelenKutusu` atlanir; `banned` uzantilar kategoriye girse bile atlanir.
/// - `include_other = true` ise kategori disi dosyalar `FileCategory::Other` olur.
/// - Okunamayan klasor/girdi taramayi durdurmaz; `warnings` listesine yazilir.
/// - `is_cancelled` true dondurdugunde tarama `CANCEL_MSG` ile kesilir.
pub fn scan_reporting(
    root: &Path,
    include_subdirs: bool,
    exclude_folders: &[String],
    banned: &[String],
    include_other: bool,
    on_progress: &(dyn Fn(u64) + Sync),
    is_cancelled: &(dyn Fn() -> bool + Sync),
) -> Result<ScanOutput, String> {
    if !root.exists() {
        return Err(format!("Klasor bulunamadi: {}", root.display()));
    }

    let excluded: HashSet<String> = exclude_folders
        .iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let mut files: Vec<FileItem> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut found: u64 = 0;

    let walker = walker(root)
        .skip_hidden(false)
        .max_depth(if include_subdirs { usize::MAX } else { 1 });

    for entry in walker {
        if is_cancelled() {
            return Err(crate::models::CANCEL_MSG.to_string());
        }

        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warnings.push(format!("Okunamadı, atlandı: {e}"));
                continue;
            }
        };
        if entry.file_type().is_dir() {
            continue;
        }

        let path = entry.path();
        if is_excluded(&path, root, &excluded, &[QUARANTINE_DIR, INBOX_DIR]) {
            continue;
        }

        found += 1;
        if found.is_multiple_of(25) {
            on_progress(found);
        }

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        let category = match category_of(&extension) {
            Some(c) => c,
            None if include_other => FileCategory::Other,
            None => continue,
        };

        // Yasakli uzantilar kategoriye girse bile taranmaz.
        if is_banned(&extension, banned) {
            continue;
        }

        // jwalk metadata'si Windows'ta onbelleklidir (ekstra syscall yok).
        let meta = entry.metadata().ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let mtime = meta
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| name.clone());

        files.push(FileItem {
            path: path.to_string_lossy().into_owned(),
            name,
            stem,
            extension,
            size,
            mtime,
            category,
            hash_id: String::new(),
        });
    }

    on_progress(found);
    Ok(ScanOutput { files, warnings })
}

/// Klasordeki dosyalari düz (hashsiz) listeler; sadece metadata okunur.
/// `include_uncategorized = false`: yalniz hedef kategoriler (Dokuman/Resim/Arsiv);
/// `true`: kategori disi dosyalar da `category: None` olarak doner.
/// Yasakli uzantilar ve `_Kopyalar_Karantina` her zaman atlanir.
pub fn list_files(
    root: &Path,
    include_subdirs: bool,
    exclude_folders: &[String],
    banned: &[String],
    include_uncategorized: bool,
) -> Result<Vec<FolderFile>, String> {
    if !root.exists() {
        return Err(format!("Klasor bulunamadi: {}", root.display()));
    }

    let excluded: HashSet<String> = exclude_folders
        .iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let mut files: Vec<FolderFile> = Vec::new();

    let walker = walker(root)
        .skip_hidden(false)
        .max_depth(if include_subdirs { usize::MAX } else { 1 });

    for entry in walker {
        let Ok(entry) = entry else { continue }; // okunamayan girdi listeyi durdurmaz
        if entry.file_type().is_dir() {
            continue;
        }

        let path = entry.path();
        if is_excluded(&path, root, &excluded, &[QUARANTINE_DIR]) {
            continue;
        }

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        if is_banned(&extension, banned) {
            continue;
        }

        let category = category_of(&extension);
        if category.is_none() && !include_uncategorized {
            continue;
        }

        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

        files.push(FolderFile {
            name,
            path: path.to_string_lossy().into_owned(),
            size,
            category,
            created_at: entry.metadata().ok().and_then(|m| m.created().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()),
            modified_at: entry.metadata().ok().and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()),
        });
    }

    files.sort_by_key(|f| std::cmp::Reverse(f.size));
    Ok(files)
}

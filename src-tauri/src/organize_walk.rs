//! "Türüne Göre Düzenle" icin dosya toplama: hariç klasorler ve karantina
//! budanir, proje klasorleri (package.json, .git ...) butunuyle korunur.

use crate::models::{CancelFn, CANCEL_MSG, QUARANTINE_DIR};
use crate::organize_rules::has_project_marker;
use crate::scanner::walker;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Taranan tek bir dosya.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub path: PathBuf,
    pub name: String,
    /// Kucuk harf, noktasiz uzanti (yoksa bos).
    pub ext: String,
    pub size: u64,
    pub mtime: u64,
}

pub struct WalkResult {
    pub files: Vec<Candidate>,
    pub protected_dirs: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn collect(
    root: &Path,
    include_subdirs: bool,
    exclude_folders: &[String],
    protect_projects: bool,
    on_found: &dyn Fn(u64),
    is_cancelled: CancelFn<'_>,
) -> Result<WalkResult, String> {
    if !root.is_dir() {
        return Err(format!("Klasör bulunamadı: {}", root.display()));
    }
    let mut excluded: HashSet<String> = exclude_folders
        .iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    excluded.insert(QUARANTINE_DIR.to_lowercase());

    let protected = Arc::new(Mutex::new(Vec::<String>::new()));
    let shared = Arc::clone(&protected);
    let root_buf = root.to_path_buf();

    let walk = walker(root)
        .skip_hidden(false)
        .max_depth(if include_subdirs { usize::MAX } else { 1 })
        .process_read_dir(move |_depth, dir, _state, children| {
            if protect_projects && dir != root_buf.as_path() {
                let names: Vec<String> = children
                    .iter()
                    .filter_map(|c| c.as_ref().ok())
                    .map(|e| e.file_name.to_string_lossy().into_owned())
                    .collect();
                if has_project_marker(names.iter().map(String::as_str)) {
                    if let Ok(mut list) = shared.lock() {
                        list.push(dir.to_string_lossy().into_owned());
                    }
                    children.clear();
                    return;
                }
            }
            children.retain(|c| match c {
                Ok(e) => {
                    !(e.file_type.is_dir()
                        && excluded.contains(&e.file_name.to_string_lossy().to_lowercase()))
                }
                Err(_) => true,
            });
        });

    let mut files = Vec::new();
    let mut warnings = Vec::new();
    for entry in walk {
        if is_cancelled() {
            return Err(CANCEL_MSG.to_string());
        }
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warnings.push(e.to_string());
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_string) else {
            warnings.push(format!("Adı okunamayan dosya atlandı: {}", path.display()));
            continue;
        };
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_lowercase)
            .unwrap_or_default();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                warnings.push(format!("{}: {e}", path.display()));
                continue;
            }
        };
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        files.push(Candidate { path, name, ext, size: meta.len(), mtime });
        if files.len().is_multiple_of(50) {
            on_found(files.len() as u64);
        }
    }
    on_found(files.len() as u64);
    // jwalk paralel calisir; deterministik plan icin yola gore sirala.
    files.sort_by(|a, b| a.path.cmp(&b.path));

    let mut protected_dirs = protected.lock().map(|l| l.clone()).unwrap_or_default();
    protected_dirs.sort();
    Ok(WalkResult { files, protected_dirs, warnings })
}

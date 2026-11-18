//! Iki klasoru tek tarama hattinda birlestirip gruplar.
//! Amac: A ve B klasorleri arasindaki birebir kopyalari (ve isim benzerlerini)
//! tek seferde bulmak. Tasima hedefi her zaman A kokudur (frontend karar verir).

use crate::grouper;
use crate::hash_cache::HashCache;
use crate::models::{
    CancelFn, ComparedFile, ComparisonDetails, FolderFile, ProgressFn, ScanProgress, ScanResult, DEFAULT_SIMILARITY_THRESHOLD,
};
use crate::scanner;
use crate::AppState;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use tauri::ipc::Channel;
use tauri::{AppHandle, State};

/// A ve B klasorlerini tarar, birlesik dosya listesini gruplar.
pub fn run_compare(
    root_a: &Path,
    root_b: &Path,
    include_subdirs: bool,
    threshold: f64,
    exclude_folders: &[String],
    banned: &[String],
    include_other: bool,
    cache: &HashCache,
    report_scan: &(dyn Fn(u64) + Sync),
    progress: ProgressFn<'_>,
    is_cancelled: CancelFn<'_>,
) -> Result<ScanResult, String> {
    let a = root_a.canonicalize().map_err(|e| e.to_string())?;
    let b = root_b.canonicalize().map_err(|e| e.to_string())?;
    if a.starts_with(&b) || b.starts_with(&a) {
        return Err("A ve B farklı ve iç içe olmayan klasörler olmalıdır.".into());
    }
    let scan_a = scanner::scan_reporting(root_a, include_subdirs, exclude_folders, banned, include_other, report_scan, is_cancelled)?;
    let (files_a, mut warnings) = (scan_a.files, scan_a.warnings);
    let found_a = files_a.len() as u64;

    let report_b = |found_b: u64| report_scan(found_a + found_b);
    let scan_b = scanner::scan_reporting(root_b, include_subdirs, exclude_folders, banned, include_other, &report_b, is_cancelled)?;
    let mut files_b = scan_b.files;
    warnings.extend(scan_b.warnings);

    let total_files = files_a.len() + files_b.len();
    let paths_a: std::collections::HashSet<_> = files_a.iter().map(|f| f.path.clone()).collect();
    let mut files = files_a;
    files.append(&mut files_b);
    let mut inventory: Vec<ComparedFile> = files.iter().map(|f| ComparedFile {
        source: if paths_a.contains(&f.path) { "A" } else { "B" }.into(),
        group_id: None,
        file: FolderFile {
            name: f.name.clone(), path: f.path.clone(), size: f.size, category: Some(f.category),
            created_at: std::fs::metadata(&f.path).ok().and_then(|m| m.created().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()),
            modified_at: Some(f.mtime),
        },
    }).collect();
    let (mut groups, unreadable) = grouper::group_reporting(files, threshold, cache, progress, is_cancelled)?;
    warnings.extend(unreadable);
    groups.retain(|g| g.files.iter().any(|f| paths_a.contains(&f.path))
        && g.files.iter().any(|f| !paths_a.contains(&f.path)));
    groups.sort_by_key(|g| g.files.iter().map(|f| &f.path).min().cloned());
    let mut membership = std::collections::HashMap::new();
    for (i, g) in groups.iter_mut().enumerate() {
        g.id = format!("GRP-{:04}", i + 1);
        for f in &g.files { membership.insert(f.path.clone(), g.id.clone()); }
    }
    for entry in &mut inventory { entry.group_id = membership.get(&entry.file.path).cloned(); }
    inventory.sort_by(|a, b| a.group_id.cmp(&b.group_id).then(a.file.name.cmp(&b.file.name)));

    Ok(ScanResult {
        root_path: root_a.to_string_lossy().into_owned(),
        total_files,
        groups,
        comparison: Some(ComparisonDetails { root_b: root_b.to_string_lossy().into_owned(), files: inventory }),
        warnings,
    })
}

/// A ve B klasorlerini karsilastirip gruplari dondurur (Tauri komutu).
#[tauri::command(async)]
pub(crate) fn compare_folders(
    app: AppHandle, state: State<'_, AppState>,
    root_a: String,
    root_b: String,
    include_subdirs: Option<bool>,
    similarity_threshold: Option<f64>,
    exclude_folders: Option<Vec<String>>,
    banned_extensions: Option<Vec<String>>,
    include_other: Option<bool>,
    on_progress: Channel<ScanProgress>,
) -> Result<ScanResult, String> {
    state.cancel_scan.store(false, Ordering::SeqCst);
    let cancel_flag = &state.cancel_scan;
    let is_cancelled = move || cancel_flag.load(Ordering::Relaxed);

    let include = include_subdirs.unwrap_or(true);
    let threshold = similarity_threshold
        .unwrap_or(DEFAULT_SIMILARITY_THRESHOLD)
        .clamp(0.5, 0.99);
    let excludes = exclude_folders.unwrap_or_default();
    let banned = crate::models::normalize_exts(&banned_extensions.unwrap_or_default());
    let include_other = include_other.unwrap_or(false);

    let emit = |phase: &str, done: u64, total: u64| {
        let _ = on_progress.send(ScanProgress {
            phase: phase.to_string(),
            files_done: done,
            total_files: total,
        });
    };

    for r in [&root_a, &root_b] { crate::asset_scope::allow_dir(&app, Path::new(r)); }
    emit("scanning", 0, 0);
    let result = run_compare(
        &PathBuf::from(&root_a),
        &PathBuf::from(&root_b),
        include,
        threshold,
        &excludes,
        &banned,
        include_other,
        &state.hash_cache,
        &|found| emit("scanning", found, 0),
        &|done, total| emit("hashing", done, total),
        &is_cancelled,
    )?;
    let _ = state.hash_cache.save(&state.dir);
    emit("done", 1, 1);
    Ok(result)
}

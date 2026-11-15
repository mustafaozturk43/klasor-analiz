//! Karantina yonetimi: icerigi listeleme, dosyalari asil yerlerine geri yukleme,
//! Geri Donusum Kutusu'na gonderme ve secili dosyalari karantinaya alma. Tasimalar
//! gecmise (dosyaya dokunmadan once) yazilir ve geri alinabilir. Kalici silme
//! yapilmaz; kutuya gonderilenler uygulamadan degil Geri Donusum Kutusu'ndan geri alinir.

use crate::models::{MoveRecord, UndoRecord, QUARANTINE_DIR};
use crate::organize_cleanup::prune_empty_dirs;
use crate::organize_models::FailedMove;
use crate::path_guard::{ensure_inside, in_quarantine};
use crate::{file_ops, history, naming, AppState};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuarantineItem {
    pub path: String,
    /// Karantinadaki ilk klasor (etiket veya kategori adi).
    pub group: String,
    pub size: u64,
    pub mtime: u64,
    /// Gecmiste kayitli asil konum (yoksa None).
    pub original: Option<String>,
    /// Karantinaya alinma zamani (ms); gecmiste kaydi yoksa None.
    pub quarantined_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuarantineListing {
    pub quarantine_path: String,
    pub items: Vec<QuarantineItem>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchOutcome {
    pub done: u64,
    pub failed: Vec<FailedMove>,
}

/// Gecmisteki tasimalardan `hedef -> kaynak` haritasi (kucuk harf anahtar, en yeni kazanir).
fn originals(records: &[UndoRecord]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for rec in records.iter().rev() {
        for m in &rec.moves {
            map.insert(m.to.to_lowercase(), m.from.clone());
        }
    }
    map
}

/// Gecmisteki tasimalardan `hedef -> islem zamani (ms)` haritasi (en yeni kazanir).
fn moved_at(records: &[UndoRecord]) -> HashMap<String, u64> {
    records.iter().rev().flat_map(|r| r.moves.iter().map(move |m| (m.to.to_lowercase(), r.timestamp))).collect()
}

pub(crate) fn list_core(app: &AppState, root: &Path) -> Result<QuarantineListing, String> {
    let q = root.join(QUARANTINE_DIR);
    let mut items = Vec::new();
    if q.is_dir() {
        let (map, when) = {
            let history = app.history.lock().map_err(|e| e.to_string())?;
            (originals(&history), moved_at(&history))
        };
        for entry in jwalk::WalkDir::new(&q).skip_hidden(false) {
            let entry = entry.map_err(|e| e.to_string())?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let meta = entry.metadata().map_err(|e| e.to_string())?;
            let group = path.strip_prefix(&q).ok().and_then(|r| r.components().next())
                .filter(|_| path.parent() != Some(q.as_path()))
                .map(|c| c.as_os_str().to_string_lossy().into_owned()).unwrap_or_default();
            let key = path.to_string_lossy().to_lowercase();
            items.push(QuarantineItem {
                original: map.get(&key).cloned(),
                quarantined_ms: when.get(&key).copied(),
                path: path.to_string_lossy().into_owned(),
                group,
                size: meta.len(),
                mtime: meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0),
            });
        }
    }
    items.sort_by(|a, b| a.group.cmp(&b.group).then_with(|| a.path.cmp(&b.path)));
    let total_bytes = items.iter().map(|i| i.size).sum();
    Ok(QuarantineListing { quarantine_path: q.to_string_lossy().into_owned(), items, total_bytes })
}

/// Dosyalari tek bir gecmis kaydi altinda tasir. Her tasima dosyaya dokunmadan
/// once kayda yazilir; basarisiz olan kayittan cikarilir ve islem devam eder.
fn logged_moves(app: &AppState, tag: &str, sources: &[PathBuf], dest_for: &dyn Fn(&Path) -> PathBuf) -> Result<BatchOutcome, String> {
    let mut guard = app.history.lock().map_err(|e| e.to_string())?;
    let now = crate::now_ms();
    history::push(&mut guard, UndoRecord { id: format!("MV-{now}"), timestamp: now, tag: tag.into(), moves: Vec::new() });
    let mut out = BatchOutcome { done: 0, failed: Vec::new() };
    for src in sources {
        let dest = dest_for(src);
        let mut result = dest.parent().map_or(Ok(()), |p| fs::create_dir_all(p).map_err(|e| e.to_string()));
        if result.is_ok() {
            guard[0].moves.push(MoveRecord { from: src.to_string_lossy().into_owned(), to: dest.to_string_lossy().into_owned() });
            result = history::save(&app.dir, &guard).and_then(|_| file_ops::transfer(src, &dest));
            if result.is_err() {
                guard[0].moves.pop();
            }
        }
        match result {
            Ok(()) => out.done += 1,
            Err(error) => out.failed.push(FailedMove { path: src.to_string_lossy().into_owned(), error }),
        }
    }
    if guard[0].moves.is_empty() {
        guard.remove(0);
    }
    history::save(&app.dir, &guard)?;
    Ok(out)
}

fn quarantine_sources(root: &Path, paths: &[String]) -> Result<Vec<PathBuf>, String> {
    let q = root.join(QUARANTINE_DIR);
    paths.iter().map(|p| { let p = PathBuf::from(p); ensure_inside(&q, &p).map(|_| p) }).collect()
}

pub(crate) fn restore_core(app: &AppState, root: &Path, paths: &[String]) -> Result<BatchOutcome, String> {
    let sources = quarantine_sources(root, paths)?;
    let map = originals(&app.history.lock().map_err(|e| e.to_string())?);
    let q = root.join(QUARANTINE_DIR);
    let dest_for = |src: &Path| -> PathBuf {
        if let Some(orig) = map.get(&src.to_string_lossy().to_lowercase()) {
            let o = PathBuf::from(orig);
            if !o.exists() && !in_quarantine(root, &o) {
                return o;
            }
        }
        // Asil yer bilinmiyor veya dolu: karantinadaki klasor adiyla ana klasore.
        let group = src.strip_prefix(&q).ok().and_then(|r| r.parent()).map(Path::to_path_buf).unwrap_or_default();
        let name = src.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        naming::unique_dest(&root.join(group), &name)
    };
    let out = logged_moves(app, "Karantinadan geri yükleme", &sources, &dest_for)?;
    prune_empty_dirs(&q, sources.iter().filter_map(|s| s.parent().map(Path::to_path_buf)), &|_| false);
    Ok(out)
}

/// Secili karantina dosyalarini `send` ile (uretimde Geri Donusum Kutusu) gonderir.
pub(crate) fn delete_core(root: &Path, paths: &[String], send: &dyn Fn(&Path) -> Result<(), String>) -> Result<u64, String> {
    let sources = quarantine_sources(root, paths)?;
    let mut deleted = 0;
    for src in &sources {
        if !src.is_file() {
            return Err(format!("Yalnızca dosyalar gönderilebilir: {}", src.display()));
        }
        send(src).map_err(|e| format!("{}: {e}", src.display()))?;
        deleted += 1;
    }
    prune_empty_dirs(&root.join(QUARANTINE_DIR), sources.iter().filter_map(|s| s.parent().map(Path::to_path_buf)), &|_| false);
    Ok(deleted)
}

pub(crate) fn quarantine_core(app: &AppState, root: &Path, paths: &[String], label: &str) -> Result<BatchOutcome, String> {
    let mut sources = Vec::new();
    for p in paths {
        let p = PathBuf::from(p);
        ensure_inside(root, &p)?;
        if in_quarantine(root, &p) || !p.is_file() {
            return Err(format!("Karantinaya alınamaz: {}", p.display()));
        }
        sources.push(p);
    }
    let dir = root.join(QUARANTINE_DIR).join(naming::sanitize_tag(label));
    let dest_for = |src: &Path| naming::unique_dest(&dir, &src.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default());
    logged_moves(app, &format!("Karantinaya alma · {label}"), &sources, &dest_for)
}

#[tauri::command(async)]
pub(crate) fn list_quarantine(handle: tauri::AppHandle, app: State<'_, AppState>, root_path: String) -> Result<QuarantineListing, String> {
    crate::asset_scope::allow_dir(&handle, Path::new(&root_path));
    list_core(&app, Path::new(&root_path))
}

#[tauri::command(async)]
pub(crate) fn restore_quarantined(app: State<'_, AppState>, root_path: String, paths: Vec<String>) -> Result<BatchOutcome, String> {
    restore_core(&app, Path::new(&root_path), &paths)
}

#[tauri::command(async)]
pub(crate) fn delete_quarantined(root_path: String, paths: Vec<String>) -> Result<u64, String> {
    delete_core(Path::new(&root_path), &paths, &|p| trash::delete(p).map_err(|e| e.to_string()))
}

#[tauri::command(async)]
pub(crate) fn quarantine_files(app: State<'_, AppState>, root_path: String, paths: Vec<String>, label: String) -> Result<BatchOutcome, String> {
    quarantine_core(&app, Path::new(&root_path), &paths, &label)
}

#[cfg(test)]
#[path = "quarantine_tests.rs"]
mod tests;

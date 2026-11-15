//! Harita > Klasordeki dosyalar: secilen dosyalari "hedef klasor + grup adi"
//! yoluna tasir. Once on izleme plani uretilir ve bekletilir; uygulama sirasinda
//! dosya damgalari yeniden dogrulanir, her tasima once Gecmis'e yazilir.

use crate::models::{MoveRecord, UndoRecord, QUARANTINE_DIR};
use crate::operations::MoveOutcome;
use crate::organize_cleanup::prune_empty_dirs;
use crate::organize_models::FailedMove;
use crate::path_guard::{canonical, in_quarantine};
use crate::rename::FileStamps;
use crate::selection_paths::{free_dest, split_group};
use crate::{file_ops, history, AppState};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

const MAX_PATH_CHARS: usize = 259;
const CHANGED: &str = "Dosya veya hedef değişti; yeniden ön izleyin.";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionMoveRequest {
    pub roots: Vec<String>,
    pub paths: Vec<String>,
    pub target_base: String,
    pub group: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionMovePlan {
    pub id: String,
    pub target_dir: String,
    pub created_dirs: Vec<String>,
    pub entries: Vec<MoveRecord>,
    pub renamed: usize,
}

#[derive(Default)]
pub struct SelectionMoveState {
    pending: Mutex<Option<(SelectionMovePlan, FileStamps)>>,
}

fn has_quarantine(p: &Path) -> bool {
    p.components().any(|c| c.as_os_str().to_string_lossy().eq_ignore_ascii_case(QUARANTINE_DIR))
}

fn stamp(meta: &fs::Metadata) -> (u64, Option<SystemTime>) {
    (meta.len(), meta.modified().ok())
}

/// Hedef yolu ve her dosyanin yeni yolunu hesaplar; diske dokunmaz.
pub fn plan_selection_move(req: &SelectionMoveRequest) -> Result<(SelectionMovePlan, FileStamps), String> {
    if req.paths.is_empty() {
        return Err("Önce dosya seçin.".into());
    }
    let roots: Vec<PathBuf> = req.roots.iter().filter(|r| !r.trim().is_empty()).map(|r| canonical(Path::new(r))).collect::<Result<_, _>>()?;
    let segs = split_group(&req.group)?;
    let base = PathBuf::from(&req.target_base);
    let base_c = canonical(&base).map_err(|_| "Hedef klasör bulunamadı.".to_string())?;
    if !base_c.is_dir() {
        return Err("Hedef bir klasör olmalı.".into());
    }
    if has_quarantine(&base_c) {
        return Err("Karantina hedef olarak seçilemez.".into());
    }
    let (mut target, mut created) = (base.clone(), Vec::new());
    for seg in &segs {
        target = target.join(seg);
        if target.exists() && !target.is_dir() {
            return Err(format!("“{seg}” adında bir dosya var; klasör oluşturulamaz."));
        }
        if !target.exists() {
            created.push(target.to_string_lossy().into_owned());
        }
    }
    let target_c = if created.is_empty() { canonical(&target)? } else { PathBuf::new() };
    let (mut seen, mut reserved, mut stamps, mut entries, mut renamed) = (HashSet::new(), HashSet::new(), Vec::new(), Vec::new(), 0);
    for raw in &req.paths {
        let p = canonical(Path::new(raw))?;
        if !roots.iter().any(|r| p.starts_with(r) && p != *r) || roots.iter().any(|r| in_quarantine(r, &p)) {
            return Err(format!("Bu dosya taşınamaz (açık klasör dışında veya karantinada): {raw}"));
        }
        if !seen.insert(p.to_string_lossy().to_lowercase()) {
            return Err(format!("Aynı dosya iki kez seçildi: {raw}"));
        }
        if p.parent().is_some_and(|d| d == target_c) {
            return Err("Dosyalar zaten bu klasörde.".into());
        }
        let meta = fs::metadata(&p).map_err(|e| format!("{raw}: {e}"))?;
        if !meta.is_file() {
            return Err(format!("Yalnızca dosyalar taşınabilir: {raw}"));
        }
        stamps.push(stamp(&meta));
        let name = Path::new(raw).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let dest = free_dest(&target, &name, &mut reserved);
        if dest.to_string_lossy().chars().count() > MAX_PATH_CHARS {
            return Err(format!("{name}: yeni yol 260 karakteri aşıyor."));
        }
        renamed += usize::from(dest.file_name().is_some_and(|n| n.to_string_lossy() != name));
        entries.push(MoveRecord { from: raw.clone(), to: dest.to_string_lossy().into_owned() });
    }
    let id = format!("SM-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos());
    let plan = SelectionMovePlan { id, target_dir: target.to_string_lossy().into_owned(), created_dirs: created, entries, renamed };
    Ok((plan, stamps))
}

/// Plani uygular: damgalar tutmazsa hicbir sey yapmaz; her tasimayi once Gecmis'e yazar.
pub(crate) fn apply_selection_core(app: &AppState, plan: &SelectionMovePlan, stamps: &FileStamps) -> Result<MoveOutcome, String> {
    for (entry, st) in plan.entries.iter().zip(stamps) {
        let meta = fs::metadata(&entry.from).map_err(|_| CHANGED.to_string())?;
        if stamp(&meta) != *st || Path::new(&entry.to).exists() {
            return Err(CHANGED.into());
        }
    }
    let target = PathBuf::from(&plan.target_dir);
    fs::create_dir_all(&target).map_err(|e| e.to_string())?;
    let label = target.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| plan.target_dir.clone());
    let mut guard = app.history.lock().map_err(|e| e.to_string())?;
    history::push(&mut guard, UndoRecord { id: plan.id.clone(), timestamp: crate::now_ms(), tag: format!("Seçilenleri taşı · {label}"), moves: Vec::new() });
    history::save(&app.dir, &guard)?;
    let mut out = MoveOutcome { moved: Vec::new(), failed: Vec::new(), removed_dirs: 0 };
    for entry in &plan.entries {
        guard[0].moves.push(entry.clone());
        history::save(&app.dir, &guard)?;
        match file_ops::transfer(Path::new(&entry.from), Path::new(&entry.to)) {
            Ok(()) => out.moved.push(entry.clone()),
            Err(error) => {
                guard[0].moves.pop();
                out.failed.push(FailedMove { path: entry.from.clone(), error });
            }
        }
    }
    if guard[0].moves.is_empty() {
        guard.remove(0);
        if let Some(boundary) = plan.created_dirs.first().and_then(|d| Path::new(d).parent()) {
            out.removed_dirs = prune_empty_dirs(boundary, plan.created_dirs.iter().map(PathBuf::from), &|_| false);
        }
    }
    history::save(&app.dir, &guard)?;
    Ok(out)
}

#[tauri::command(async)]
pub(crate) fn preview_selection_move(state: State<'_, SelectionMoveState>, request: SelectionMoveRequest) -> Result<SelectionMovePlan, String> {
    let (plan, stamps) = plan_selection_move(&request)?;
    *state.pending.lock().map_err(|e| e.to_string())? = Some((plan.clone(), stamps));
    Ok(plan)
}

#[tauri::command(async)]
pub(crate) fn apply_selection_move(state: State<'_, SelectionMoveState>, app: State<'_, AppState>, plan_id: String) -> Result<MoveOutcome, String> {
    let (plan, stamps) = {
        let mut pending = state.pending.lock().map_err(|e| e.to_string())?;
        match pending.take() {
            Some((plan, stamps)) if plan.id == plan_id => (plan, stamps),
            other => { *pending = other; return Err("Plan güncel değil; yeniden ön izleyin.".into()); }
        }
    };
    apply_selection_core(&app, &plan, &stamps)
}

#[cfg(test)]
#[path = "selection_move_tests.rs"]
mod tests;

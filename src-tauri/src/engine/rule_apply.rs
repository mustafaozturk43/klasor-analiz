//! Kural motoru komutlari: onizleme plani sunucuda bekletilir; uygulamada dosya damgalari
//! yeniden dogrulanir. Tasimalar ortak `batch_move` ile (once Gecmis'e yazilir, geri alinabilir);
//! Geri Donusum Kutusu eylemi yalnizca `allow_trash` ile ve Gecmis'e yazilmadan yapilir.

use super::rule_model::Rule;
use super::rule_plan::{plan_rules, RuleEntry, RulePlan};
use crate::batch_move::{ensure_unchanged, run_batches, CHANGED};
use crate::models::{MoveRecord, ScanProgress};
use crate::organize_models::FailedMove;
use crate::path_guard::ensure_inside;
use crate::rename::FileStamps;
use crate::AppState;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::ipc::Channel;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleOutcome {
    pub moved: Vec<MoveRecord>,
    pub trashed: usize,
    pub failed: Vec<FailedMove>,
    pub removed_dirs: u64,
}

#[derive(Default)]
pub struct RuleEngineState {
    pending: Mutex<Option<(RulePlan, FileStamps)>>,
}

/// `send_to_trash`: testlerde sahte; uygulamada Windows Geri Donusum Kutusu.
pub(crate) fn apply_core(
    app: &AppState,
    plan: &RulePlan,
    stamps: &FileStamps,
    allow_trash: bool,
    send_to_trash: &dyn Fn(&Path) -> Result<(), String>,
    progress: &dyn Fn(u64, u64),
) -> Result<RuleOutcome, String> {
    if plan.entries.is_empty() {
        return Err("Uygulanacak işlem yok.".into());
    }
    if plan.trash_count > 0 && !allow_trash {
        return Err("Geri Dönüşüm Kutusu'na gönderme onaylanmadı.".into());
    }
    let root = Path::new(&plan.root);
    let (mut moves, mut move_stamps, mut trash): (Vec<MoveRecord>, FileStamps, Vec<(&RuleEntry, _)>) = (Vec::new(), Vec::new(), Vec::new());
    for (entry, st) in plan.entries.iter().zip(stamps) {
        if entry.to.is_empty() {
            // Kok disindaki dosya asla gonderilmez.
            ensure_inside(root, Path::new(&entry.from))?;
            trash.push((entry, *st));
        } else {
            moves.push(MoveRecord { from: entry.from.clone(), to: entry.to.clone() });
            move_stamps.push(*st);
        }
    }
    ensure_unchanged(&moves, &move_stamps)?;
    for (entry, st) in &trash {
        let meta = fs::metadata(&entry.from).map_err(|_| CHANGED.to_string())?;
        if (meta.len(), meta.modified().ok()) != *st {
            return Err(CHANGED.into());
        }
    }
    let total = (moves.len() + trash.len()) as u64;
    let label = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| plan.root.clone());
    let mut out = if moves.is_empty() {
        RuleOutcome { moved: Vec::new(), trashed: 0, failed: Vec::new(), removed_dirs: 0 }
    } else {
        let done = run_batches(app, &plan.id, format!("Kural motoru · {label}"), &moves, root, &plan.created_dirs, &|d, _| progress(d, total))?;
        RuleOutcome { moved: done.moved, trashed: 0, failed: done.failed, removed_dirs: done.removed_dirs }
    };
    let base = moves.len() as u64;
    for (i, (entry, _)) in trash.iter().enumerate() {
        match send_to_trash(Path::new(&entry.from)) {
            Ok(()) => out.trashed += 1,
            Err(error) => out.failed.push(FailedMove { path: entry.from.clone(), error }),
        }
        progress(base + i as u64 + 1, total);
    }
    Ok(out)
}

#[tauri::command(async)]
pub(crate) fn preview_rules(state: State<'_, RuleEngineState>, root: String, rules: Vec<Rule>, recursive: bool) -> Result<RulePlan, String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let (plan, stamps) = plan_rules(Path::new(&root), &rules, recursive, now)?;
    *state.pending.lock().map_err(|e| e.to_string())? = Some((plan.clone(), stamps));
    Ok(plan)
}

#[tauri::command(async)]
pub(crate) fn apply_rules(
    state: State<'_, RuleEngineState>,
    app: State<'_, AppState>,
    plan_id: String,
    allow_trash: bool,
    on_progress: Channel<ScanProgress>,
) -> Result<RuleOutcome, String> {
    let (plan, stamps) = {
        let mut pending = state.pending.lock().map_err(|e| e.to_string())?;
        match pending.take() {
            Some((plan, stamps)) if plan.id == plan_id => (plan, stamps),
            other => {
                *pending = other;
                return Err("Ön izleme güncel değil; yeniden ön izleyin.".into());
            }
        }
    };
    let send = |done: u64, total: u64| {
        let _ = on_progress.send(ScanProgress { phase: "moving".into(), files_done: done, total_files: total });
    };
    apply_core(&app, &plan, &stamps, allow_trash, &|p| trash::delete(p).map_err(|e| e.to_string()), &send)
}

#[cfg(test)]
#[path = "rule_engine_tests.rs"]
mod tests;

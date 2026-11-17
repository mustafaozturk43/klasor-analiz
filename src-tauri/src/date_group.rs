//! Tarihe Gore (Yıl\Ay) gruplama komutlari: on izleme plani sunucuda bekletilir;
//! uygulamada dosya damgalari yeniden dogrulanir, her tasima once Gecmis'e yazilir.
//! Geri alma, genel `history_cmds` yoluyla yapilir (bosalan Yıl\Ay klasorleri de kalkar).

use crate::date_group_plan::{plan_date_group, DateGroupOptions, DateGroupPlan};
use crate::models::{ScanProgress, UndoRecord};
use crate::operations::MoveOutcome;
use crate::organize_cleanup::prune_empty_dirs;
use crate::organize_models::FailedMove;
use crate::rename::FileStamps;
use crate::{file_ops, history, AppState};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::State;

const CHANGED: &str = "Dosyalar ön izlemeden sonra değişti; yeniden ön izleyin.";

#[derive(Default)]
pub struct DateGroupState {
    pending: Mutex<Option<(DateGroupPlan, FileStamps)>>,
}

pub(crate) fn apply_core(app: &AppState, plan: &DateGroupPlan, stamps: &FileStamps) -> Result<MoveOutcome, String> {
    if plan.entries.is_empty() {
        return Err("Taşınacak dosya yok.".into());
    }
    for (entry, st) in plan.entries.iter().zip(stamps) {
        let meta = fs::metadata(&entry.from).map_err(|_| CHANGED.to_string())?;
        if (meta.len(), meta.modified().ok()) != *st || Path::new(&entry.to).exists() {
            return Err(CHANGED.into());
        }
    }
    let label = Path::new(&plan.root).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| plan.root.clone());
    let mut guard = app.history.lock().map_err(|e| e.to_string())?;
    history::push(&mut guard, UndoRecord { id: plan.id.clone(), timestamp: crate::now_ms(), tag: format!("Tarihe göre gruplama · {label}"), moves: Vec::new() });
    history::save(&app.dir, &guard)?;
    let mut out = MoveOutcome { moved: Vec::new(), failed: Vec::new(), removed_dirs: 0 };
    for entry in &plan.entries {
        let to = Path::new(&entry.to);
        if let Some(dir) = to.parent() {
            if let Err(e) = fs::create_dir_all(dir) {
                out.failed.push(FailedMove { path: entry.from.clone(), error: e.to_string() });
                continue;
            }
        }
        // Once gecmise yaz, sonra tasi: islem yarida kesilse de geri alinabilir.
        guard[0].moves.push(entry.clone());
        history::save(&app.dir, &guard)?;
        match file_ops::transfer(Path::new(&entry.from), to) {
            Ok(()) => out.moved.push(entry.clone()),
            Err(error) => {
                guard[0].moves.pop();
                out.failed.push(FailedMove { path: entry.from.clone(), error });
            }
        }
    }
    if guard[0].moves.is_empty() {
        guard.remove(0);
    }
    history::save(&app.dir, &guard)?;
    // Basarisiz tasimalardan bos kalan yeni Yıl\Ay klasorleri.
    out.removed_dirs = prune_empty_dirs(Path::new(&plan.root), plan.created_dirs.iter().map(PathBuf::from), &|_| false);
    Ok(out)
}

#[tauri::command(async)]
pub(crate) fn preview_date_group(state: State<'_, DateGroupState>, root: String, options: DateGroupOptions, on_progress: Channel<ScanProgress>) -> Result<DateGroupPlan, String> {
    let send = |done: u64, total: u64| {
        let _ = on_progress.send(ScanProgress { phase: "dating".into(), files_done: done, total_files: total });
    };
    let (plan, stamps) = plan_date_group(Path::new(&root), &options, &crate::media_dates::date_taken, &send)?;
    *state.pending.lock().map_err(|e| e.to_string())? = Some((plan.clone(), stamps));
    Ok(plan)
}

#[tauri::command(async)]
pub(crate) fn apply_date_group(state: State<'_, DateGroupState>, app: State<'_, AppState>, plan_id: String) -> Result<MoveOutcome, String> {
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
    apply_core(&app, &plan, &stamps)
}

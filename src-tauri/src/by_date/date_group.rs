//! Tarihe Gore (Yıl\Ay) gruplama komutlari: on izleme plani sunucuda bekletilir;
//! uygulamada dosya damgalari yeniden dogrulanir, her tasima once Gecmis'e yazilir.
//! Geri alma, genel `history_cmds` yoluyla yapilir (bosalan Yıl\Ay klasorleri de kalkar).

use crate::date_group_index::{rebuild_index, write_index};
use crate::date_group_plan::{collect, plan_date_group, DateGroupOptions, DateGroupPlan};
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

/// Gecmis dosyasi parti basina bir kez yazilir; buyuk islerde yazim sayisi ~40 ile sinirlanir (25..250 dosya).
fn batch_size(total: usize) -> usize {
    (total / 40).clamp(25, 250)
}

pub(crate) fn apply_core(app: &AppState, plan: &DateGroupPlan, stamps: &FileStamps, progress: &dyn Fn(u64, u64)) -> Result<MoveOutcome, String> {
    if plan.entries.is_empty() {
        return Err("Taşınacak dosya yok.".into());
    }
    for (entry, st) in plan.entries.iter().zip(stamps) {
        let meta = fs::metadata(&entry.from).map_err(|_| CHANGED.to_string())?;
        if (meta.len(), meta.modified().ok()) != *st || Path::new(&entry.to).exists() {
            return Err(CHANGED.into());
        }
    }
    // Dizin dosyasi tasimadan once kok klasore yazilir.
    write_index(Path::new(&plan.root), &plan.entries)?;
    let label =Path::new(&plan.root).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| plan.root.clone());
    let mut guard = app.history.lock().map_err(|e| e.to_string())?;
    history::push(&mut guard, UndoRecord { id: plan.id.clone(), timestamp: crate::now_ms(), tag: format!("Tarihe göre gruplama · {label}"), moves: Vec::new() });
    history::save(&app.dir, &guard)?;
    let mut out = MoveOutcome { moved: Vec::new(), failed: Vec::new(), removed_dirs: 0 };
    let total = plan.entries.len() as u64;
    progress(0, total);
    for chunk in plan.entries.chunks(batch_size(plan.entries.len())) {
        let mut ready = Vec::with_capacity(chunk.len());
        for entry in chunk {
            match Path::new(&entry.to).parent().map(fs::create_dir_all).transpose() {
                Ok(_) => ready.push(entry),
                Err(e) => out.failed.push(FailedMove { path: entry.from.clone(), error: e.to_string() }),
            }
        }
        // Once gecmise yaz (parti basina tek dosya yazimi), sonra tasi: islem yarida kesilse de geri alinabilir.
        guard[0].moves.extend(ready.iter().map(|e| (*e).clone()));
        history::save(&app.dir, &guard)?;
        let mut failed_from = Vec::new();
        for entry in ready {
            match file_ops::transfer(Path::new(&entry.from), Path::new(&entry.to)) {
                Ok(()) => out.moved.push(entry.clone()),
                Err(error) => {
                    failed_from.push(entry.from.clone());
                    out.failed.push(FailedMove { path: entry.from.clone(), error });
                }
            }
        }
        if !failed_from.is_empty() {
            guard[0].moves.retain(|m| !failed_from.contains(&m.from));
            history::save(&app.dir, &guard)?;
        }
        progress((out.moved.len() + out.failed.len()) as u64, total);
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
pub(crate) fn apply_date_group(state: State<'_, DateGroupState>, app: State<'_, AppState>, plan_id: String, on_progress: Channel<ScanProgress>) -> Result<MoveOutcome, String> {
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
    apply_core(&app, &plan, &stamps, &send)
}

/// `Tarih_Dizini.html` dosyasini kok klasorun alt klasorlerindeki gercek dosyalara gore bastan yazar.
/// Dosya tasimaz; geri alma sonrasi bayat kalan bolumleri temizler.
#[tauri::command(async)]
pub(crate) fn rebuild_date_index(root: String) -> Result<usize, String> {
    let root = Path::new(&root);
    if !root.is_dir() {
        return Err("Önce var olan bir klasör seçin.".into());
    }
    let mut files = Vec::new();
    collect(root, true, &[], &mut files);
    files.retain(|p| p.parent() != Some(root));
    if files.is_empty() {
        return Err("Alt klasörlerde dizine eklenecek dosya yok.".into());
    }
    rebuild_index(root, &files)
}

#[cfg(test)]
#[path = "date_group_scale_tests.rs"]
mod scale_tests;

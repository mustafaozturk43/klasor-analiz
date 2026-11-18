//! Tarihe Gore (Yıl\Ay) gruplama komutlari: on izleme plani sunucuda bekletilir;
//! uygulamada dosya damgalari yeniden dogrulanir, her tasima once Gecmis'e yazilir.
//! Geri alma, genel `history_cmds` yoluyla yapilir (bosalan Yıl\Ay klasorleri de kalkar).

use crate::date_group_index::{rebuild_index, write_index};
use crate::date_group_plan::{collect, plan_date_group, DateGroupOptions, DateGroupPlan};
use crate::batch_move::{ensure_unchanged, run_batches};
use crate::models::ScanProgress;
use crate::operations::MoveOutcome;
use crate::rename::FileStamps;
use crate::AppState;
use std::path::Path;
use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::State;

#[derive(Default)]
pub struct DateGroupState {
    pending: Mutex<Option<(DateGroupPlan, FileStamps)>>,
}

pub(crate) fn apply_core(app: &AppState, plan: &DateGroupPlan, stamps: &FileStamps, progress: &dyn Fn(u64, u64)) -> Result<MoveOutcome, String> {
    if plan.entries.is_empty() {
        return Err("Taşınacak dosya yok.".into());
    }
    ensure_unchanged(&plan.entries, stamps)?;
    let root = Path::new(&plan.root);
    // Dizin dosyasi tasimadan once kok klasore yazilir.
    write_index(root, &plan.entries)?;
    let label = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| plan.root.clone());
    run_batches(app, &plan.id, format!("Tarihe göre gruplama · {label}"), &plan.entries, root, &plan.created_dirs, progress)
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

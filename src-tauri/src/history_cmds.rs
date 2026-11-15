//! Islem gecmisi komutlari: listeleme, temizleme, en son islemi veya
//! gecmisten secilen herhangi bir islemi geri alma.

use crate::models::UndoRecord;
use crate::{history, mover, AppState};
use std::collections::HashSet;
use tauri::State;

/// Hedef kayittan SONRA yapilan islemlerden biri ayni dosyalara dokunmus mu?
/// Dokunduysa hedefi geri almak yanlis dosyayi tasiyabilir; once onlar geri alinmali.
pub fn touched_later(newer: &[UndoRecord], target: &UndoRecord) -> Option<String> {
    let paths: HashSet<String> = target.moves.iter().flat_map(|m| [m.from.to_lowercase(), m.to.to_lowercase()]).collect();
    newer
        .iter()
        .find(|r| r.moves.iter().any(|m| paths.contains(&m.from.to_lowercase()) || paths.contains(&m.to.to_lowercase())))
        .map(|r| r.tag.clone())
}

pub(crate) fn undo_entry_core(app: &AppState, id: &str) -> Result<UndoRecord, String> {
    let mut guard = app.history.lock().map_err(|e| e.to_string())?;
    let idx = guard.iter().position(|r| r.id == id).ok_or("Bu işlem geçmişte bulunamadı.")?;
    if let Some(tag) = touched_later(&guard[..idx], &guard[idx]) {
        return Err(format!("Bu işlemden sonra yapılan “{tag}” aynı dosyalara dokundu. Önce daha yeni işlemi geri alın."));
    }
    let record = guard[idx].clone();
    if record.tag == crate::name_rules_apply::HISTORY_LABEL {
        // Klasor adlari ve yalnizca harf degisimleri: dosya tasima yolu bunlari geri alamaz; bos klasor de silinmez.
        crate::name_rules_apply::undo(&record)?;
    } else {
        mover::undo_move(&record)?;
        // Islemin olusturdugu (artik bos) hedef klasorleri de kaldir.
        crate::organize_cleanup::prune_after_undo(&record.moves);
    }
    guard.remove(idx);
    history::save(&app.dir, &guard)?;
    Ok(record)
}

/// En son islemi geri alir; geri alinan kaydi dondurur, gecmis bossa None.
#[tauri::command(async)]
pub(crate) fn undo_last_move(state: State<'_, AppState>) -> Result<Option<UndoRecord>, String> {
    let first = state.history.lock().map_err(|e| e.to_string())?.first().map(|r| r.id.clone());
    first.map(|id| undo_entry_core(&state, &id)).transpose()
}

/// Gecmisten secilen islemi geri alir (daha yeni islemlerle cakismiyorsa).
#[tauri::command(async)]
pub(crate) fn undo_history_entry(state: State<'_, AppState>, id: String) -> Result<UndoRecord, String> {
    undo_entry_core(&state, &id)
}

#[tauri::command]
pub(crate) fn get_move_history(state: State<'_, AppState>) -> Result<Vec<UndoRecord>, String> {
    Ok(state.history.lock().map_err(|e| e.to_string())?.clone())
}

#[tauri::command]
pub(crate) fn clear_move_history(state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.history.lock().map_err(|e| e.to_string())?;
    guard.clear();
    history::save(&state.dir, &guard)
}

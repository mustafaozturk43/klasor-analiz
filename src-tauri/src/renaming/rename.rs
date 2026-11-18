//! Preview plans are retained server-side, one pending plan per application.
use crate::{AppState, file_ops, history};
use crate::models::{MoveRecord, UndoRecord};
use serde::Serialize;
use std::{collections::HashSet, fs, path::PathBuf, sync::Mutex, time::SystemTime};
use tauri::State;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameEntry { pub from: String, pub to: Option<String>, pub warning: Option<String> }
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenamePlan { pub id: String, pub entries: Vec<RenameEntry> }
pub type FileStamps = Vec<(u64, Option<SystemTime>)>;
/// `label`: islem gecmisinde gorunecek ad.
pub(crate) struct Pending { pub(crate) plan: RenamePlan, pub(crate) stamps: FileStamps, pub(crate) label: &'static str }
#[derive(Default)]
pub struct RenameState { pub(crate) pending: Mutex<Option<Pending>> }

pub fn plan_date_rename(roots: Vec<String>, paths: Vec<String>) -> Result<(RenamePlan, FileStamps), String> {
    if paths.is_empty() { return Err("Önce dosya seçin.".into()); }
    let roots: Vec<_> = roots.iter().map(|r| fs::canonicalize(r).map_err(|e| e.to_string())).collect::<Result<_, _>>()?;
    let mut entries = Vec::new();
    let mut stamps = Vec::new();
    let mut reserved = HashSet::new();
    let mut seen = HashSet::new();
    for raw in paths {
        let path = fs::canonicalize(&raw).map_err(|e| e.to_string())?;
        if !roots.iter().any(|r| path.starts_with(r)) || !seen.insert(path.clone()) {
            return Err("Geçersiz veya tekrarlanan kaynak.".into());
        }
        if path.components().any(|c| c.as_os_str().to_string_lossy().eq_ignore_ascii_case(crate::models::QUARANTINE_DIR)) {
            return Err("Karantina dosyaları bu işlemde değiştirilemez.".into());
        }
        let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
        if !meta.is_file() { return Err("Yalnız dosyalar isimlendirilebilir.".into()); }
        stamps.push((meta.len(), meta.modified().ok()));
        let date = meta.created().ok().and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok());
        let mut entry = RenameEntry { from: path.to_string_lossy().into_owned(), to: None, warning: None };
        if let Some(date) = date {
            let suffix = file_ops::local_date_suffix(date.as_secs());
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            if stem.ends_with(&format!("_{suffix}")) {
                entry.warning = Some("Tarih zaten dosya adında; atlanacak.".into());
            } else {
                let ext = path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
                let mut n = 0;
                let target = loop {
                    let extra = if n == 0 { String::new() } else { format!(" ({n})") };
                    let candidate = path.with_file_name(format!("{stem}_{suffix}{extra}{ext}"));
                    if !candidate.exists() && reserved.insert(candidate.clone()) { break candidate; }
                    n += 1;
                };
                entry.to = Some(target.to_string_lossy().into_owned());
            }
        } else { entry.warning = Some("Oluşturulma tarihi okunamadı; atlanacak.".into()); }
        entries.push(entry);
    }
    let id = format!("RN-{}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos());
    let plan = RenamePlan { id, entries };
    Ok((plan, stamps))
}

#[tauri::command(async)]
pub fn preview_date_rename(state: State<'_, RenameState>, roots: Vec<String>, paths: Vec<String>) -> Result<RenamePlan, String> {
    let (plan, stamps) = plan_date_rename(roots, paths)?;
    *state.pending.lock().map_err(|e| e.to_string())? = Some(Pending { plan: plan.clone(), stamps, label: "Tarihle isimlendirme" });
    Ok(plan)
}

/// Onizlenmis herhangi bir adlandirma planini (tarih veya sablon) uygular.
#[tauri::command(async)]
pub(crate) fn apply_rename_plan(state: State<'_, RenameState>, app: State<'_, AppState>, plan_id: String) -> Result<usize, String> {
    apply_pending(&state, &app, plan_id)
}

pub(crate) fn apply_pending(state: &RenameState, app: &AppState, plan_id: String) -> Result<usize, String> {
    let mut pending = state.pending.lock().map_err(|e| e.to_string())?;
    let p = pending.as_ref().ok_or("İsimlendirme planı yok; yeniden ön izleyin.")?;
    if p.plan.id != plan_id { return Err("Plan güncel değil.".into()); }
    for (entry, stamp) in p.plan.entries.iter().zip(&p.stamps) {
        let meta = fs::metadata(&entry.from).map_err(|e| e.to_string())?;
        if (meta.len(), meta.modified().ok()) != *stamp || entry.to.as_ref().is_some_and(|s| PathBuf::from(s).exists()) {
            return Err("Dosya veya hedef değişti; yeniden ön izleyin.".into());
        }
    }
    let p = pending.take().unwrap();
    let mut guard = app.history.lock().map_err(|e| e.to_string())?;
    let record = UndoRecord { id: p.plan.id, timestamp: crate::now_ms(), tag: p.label.into(), moves: Vec::new() };
    history::push(&mut guard, record);
    history::save(&app.dir, &guard)?;
    let mut count = 0;
    for entry in p.plan.entries {
        if let Some(to) = entry.to {
            let from = PathBuf::from(&entry.from);
            let dest = PathBuf::from(&to);
            // Persist the intended move before touching the file; undo skips absent destinations.
            guard[0].moves.push(MoveRecord { from: entry.from, to });
            history::save(&app.dir, &guard)?;
            if let Err(e) = file_ops::transfer(&from, &dest) {
                guard[0].moves.pop();
                history::save(&app.dir, &guard)?;
                return Err(format!("{count} dosya işlendi; işlem durdu: {e}. Geri al kullanılabilir."));
            }
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
#[path = "rename_tests.rs"]
mod tests;

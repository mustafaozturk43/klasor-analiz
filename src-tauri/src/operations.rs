//! Aksam Kontrolu toplu islemleri: hedef klasorler ve secili gelen kutusu
//! dosyalarini tasima. Kaynak her zaman gelen kutusunda, hedef her zaman yedek
//! klasorunde (gelen kutusu ve karantina disinda) olmali. Tasima once gecmise
//! yazilir (geri alinabilir), ardindan bosalan grup ve gun klasorleri silinir.

use crate::models::{MoveRecord, UndoRecord, INBOX_DIR, QUARANTINE_DIR};
use crate::naming::{unique_dest_with, Suffix};
use crate::organize_cleanup::prune_empty_dirs;
use crate::organize_models::FailedMove;
use crate::organize_rules::check_folder_name;
use crate::path_guard::{canonical, ensure_inside};
use crate::settings::SettingsState;
use crate::{file_ops, history, AppState};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetFolder {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveOutcome {
    pub moved: Vec<MoveRecord>,
    pub failed: Vec<FailedMove>,
    pub removed_dirs: u64,
}

fn is_reserved(name: &str) -> bool {
    name.eq_ignore_ascii_case(INBOX_DIR) || name.eq_ignore_ascii_case(QUARANTINE_DIR) || name.starts_with('.')
}

pub fn backup_root(settings: &State<'_, SettingsState>) -> Result<PathBuf, String> {
    let root = settings.settings.lock().map_err(|e| e.to_string())?.backup_root.clone();
    if root.is_empty() || !Path::new(&root).is_dir() {
        return Err("Önce Ayarlar sekmesinden var olan bir yedek klasörü seçin.".into());
    }
    Ok(PathBuf::from(root))
}

/// Yedek klasorunun dogrudan alt klasorleri (gelen kutusu ve karantina haric), ada gore.
pub fn target_folders(backup: &Path) -> Vec<TargetFolder> {
    let mut out: Vec<TargetFolder> = std::fs::read_dir(backup)
        .map(|rd| rd.flatten().filter(|e| e.path().is_dir()).collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            (!is_reserved(&name)).then(|| TargetFolder { name, path: e.path().to_string_lossy().into_owned() })
        })
        .collect();
    out.sort_by_key(|t| t.name.to_lowercase());
    out
}

/// Hedef klasor yedek klasorunun icinde olmali; gelen kutusu veya karantina olamaz.
fn check_target(backup: &Path, target: &Path) -> Result<(), String> {
    ensure_inside(backup, target)?;
    let rel = canonical(target)?.strip_prefix(canonical(backup)?).map(Path::to_path_buf).unwrap_or_default();
    match rel.components().next() {
        Some(c) if is_reserved(&c.as_os_str().to_string_lossy()) => Err("Gelen kutusu veya karantina hedef olarak seçilemez.".into()),
        _ => Ok(()),
    }
}

/// `group`: hedefin icinde olusturulacak alt klasor ("Mayıs" ya da "Faturalar\Mayıs"); bos ise dogrudan hedefe tasinir.
pub(crate) fn move_core(app: &AppState, backup: &Path, paths: &[String], target: &Path, group: &str) -> Result<MoveOutcome, String> {
    if paths.is_empty() {
        return Err("Önce dosya seçin.".into());
    }
    check_target(backup, target)?;
    let segments = crate::selection_paths::split_group(group)?;
    if let Some(bad) = segments.iter().find(|s| is_reserved(s)) {
        return Err(format!("“{bad}” grup adı olarak kullanılamaz."));
    }
    let target = &segments.iter().fold(target.to_path_buf(), |dir, seg| dir.join(seg));
    let inbox = backup.join(INBOX_DIR);
    for p in paths {
        ensure_inside(&inbox, Path::new(p)).map_err(|_| format!("Yalnızca gelen kutusundaki dosyalar taşınabilir: {p}"))?;
    }
    std::fs::create_dir_all(target).map_err(|e| format!("{}: {e}", target.display()))?;
    let label = target.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let mut guard = app.history.lock().map_err(|e| e.to_string())?;
    let now = crate::now_ms();
    history::push(&mut guard, UndoRecord { id: format!("IM-{now}"), timestamp: now, tag: format!("Gelen kutusundan taşıma · {label}"), moves: Vec::new() });
    let mut out = MoveOutcome { moved: Vec::new(), failed: Vec::new(), removed_dirs: 0 };
    for p in paths {
        let from = PathBuf::from(p);
        let name = from.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let dest = unique_dest_with(target, &name, Suffix::Underscore);
        // Once gecmise yaz, sonra tasi: islem yarida kesilse de geri alinabilir.
        guard[0].moves.push(MoveRecord { from: p.clone(), to: dest.to_string_lossy().into_owned() });
        history::save(&app.dir, &guard)?;
        match file_ops::transfer(&from, &dest) {
            Ok(()) => out.moved.push(guard[0].moves.last().cloned().unwrap_or_else(|| MoveRecord { from: p.clone(), to: String::new() })),
            Err(error) => {
                guard[0].moves.pop();
                out.failed.push(FailedMove { path: p.clone(), error });
            }
        }
    }
    if guard[0].moves.is_empty() {
        guard.remove(0);
    }
    history::save(&app.dir, &guard)?;
    drop(guard);
    let parents = out.moved.iter().filter_map(|m| Path::new(&m.from).parent().map(Path::to_path_buf));
    out.removed_dirs = prune_empty_dirs(&inbox, parents, &|_| false);
    Ok(out)
}

#[tauri::command]
pub(crate) fn get_target_subfolders(settings: State<'_, SettingsState>) -> Result<Vec<TargetFolder>, String> {
    Ok(target_folders(&backup_root(&settings)?))
}

/// Yedek klasorunde yeni bir hedef klasor olusturur (varsa olduğu gibi dondurur).
#[tauri::command]
pub(crate) fn create_target_folder(settings: State<'_, SettingsState>, name: String) -> Result<TargetFolder, String> {
    let backup = backup_root(&settings)?;
    let name = check_folder_name("Klasör", &name)?;
    if is_reserved(&name) {
        return Err(format!("“{name}” hedef klasör adı olarak kullanılamaz."));
    }
    let path = backup.join(&name);
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(TargetFolder { name, path: path.to_string_lossy().into_owned() })
}

#[tauri::command(async)]
pub(crate) fn move_selected_files(app: State<'_, AppState>, settings: State<'_, SettingsState>, paths: Vec<String>, target: String, group: Option<String>) -> Result<MoveOutcome, String> {
    move_core(&app, &backup_root(&settings)?, &paths, Path::new(&target), group.as_deref().unwrap_or(""))
}

#[cfg(test)]
#[path = "operations_tests.rs"]
mod tests;

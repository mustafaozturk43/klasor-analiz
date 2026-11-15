//! Gelen kutusundaki secili dosyalari Windows Geri Donusum Kutusu'na gonderir.
//! Kalici silme yapilmaz. Yalnizca gelen kutusundaki dosyalar kabul edilir. Bu
//! islem uygulamanin Islem Gecmisi'nden geri alinamaz; dosyalar Geri Donusum
//! Kutusu'ndan geri yuklenir. Bosalan grup ve gun klasorleri kaldirilir.

use crate::models::INBOX_DIR;
use crate::operations::backup_root;
use crate::organize_cleanup::prune_empty_dirs;
use crate::organize_models::FailedMove;
use crate::path_guard::ensure_inside;
use crate::settings::SettingsState;
use serde::Serialize;
use std::path::Path;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashOutcome {
    pub trashed: Vec<String>,
    pub failed: Vec<FailedMove>,
}

pub fn trash_core(backup: &Path, paths: &[String], send: &dyn Fn(&Path) -> Result<(), String>) -> Result<TrashOutcome, String> {
    if paths.is_empty() {
        return Err("Önce dosya seçin.".into());
    }
    let inbox = backup.join(INBOX_DIR);
    // Tek bir gecersiz yol bile varsa hicbir dosyaya dokunulmaz.
    for p in paths {
        ensure_inside(&inbox, Path::new(p)).map_err(|_| format!("Yalnızca gelen kutusundaki dosyalar gönderilebilir: {p}"))?;
        if !Path::new(p).is_file() {
            return Err(format!("Yalnızca dosyalar gönderilebilir: {p}"));
        }
    }
    let mut out = TrashOutcome { trashed: Vec::new(), failed: Vec::new() };
    for p in paths {
        match send(Path::new(p)) {
            Ok(()) => out.trashed.push(p.clone()),
            Err(error) => out.failed.push(FailedMove { path: p.clone(), error }),
        }
    }
    let parents = out.trashed.iter().filter_map(|p| Path::new(p).parent().map(Path::to_path_buf));
    prune_empty_dirs(&inbox, parents, &|_| false);
    Ok(out)
}

#[tauri::command(async)]
pub(crate) fn delete_files_to_trash(settings: State<'_, SettingsState>, paths: Vec<String>) -> Result<TrashOutcome, String> {
    trash_core(&backup_root(&settings)?, &paths, &|p| trash::delete(p).map_err(|e| e.to_string()))
}

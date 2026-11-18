//! Karantina klasoru ile ilgili Tauri komutlari.

use crate::models::QUARANTINE_DIR;
use std::path::{Path, PathBuf};

/// Karantina klasorunu butunuyle Geri Donusum Kutusu'na gonderir (kalici silme
/// yapilmaz). Yalnizca ana klasorun hemen altindaki gercek `_Kopyalar_Karantina`
/// klasoru kabul edilir; baglanti (symlink/junction) ise dokunulmaz.
pub fn clear_core(root: &Path, send: &dyn Fn(&Path) -> Result<(), String>) -> Result<(), String> {
    if !root.is_dir() {
        return Err(format!("Klasör bulunamadı: {}", root.display()));
    }
    let q = root.join(QUARANTINE_DIR);
    let Ok(meta) = std::fs::symlink_metadata(&q) else { return Ok(()) };
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err("Karantina klasörü beklenen türde değil; dokunulmadı.".into());
    }
    send(&q)
}

#[tauri::command(async)]
pub(crate) fn clear_quarantine(root_path: String) -> Result<(), String> {
    clear_core(&PathBuf::from(&root_path), &|p| trash::delete(p).map_err(|e| e.to_string()))
}

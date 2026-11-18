//! `asset://` protokolu kapsami: `tauri.conf.json` icinde bos birakilir, onizleme
//! yalnizca kullanicinin sectigi klasorler, ayarlardaki klasorler ve kucuk resim
//! onbellegi icin calisma zamaninda acilir. Kapsam bellekte tutulur; her komut
//! kendi kok klasorunu ister, uygulama acilirken de ayarlardakiler yeniden acilir.

use crate::settings::SettingsState;
use std::path::Path;
use tauri::{AppHandle, Manager};

/// Klasoru (alt klasorleriyle) onizleme icin acar. Bos ya da var olmayan yol yok sayilir.
pub fn allow_dir(app: &AppHandle, dir: &Path) {
    if dir.as_os_str().is_empty() || !dir.is_dir() {
        return;
    }
    let _ = app.asset_protocol_scope().allow_directory(dir, true);
}

/// Kucuk resim onbellegi (`app_cache_dir/thumbs`): Gelen Kutusu ve Shell kucuk resimleri.
pub fn allow_thumbs(app: &AppHandle) {
    let Ok(base) = app.path().app_cache_dir() else { return };
    let dir = base.join("thumbs");
    if std::fs::create_dir_all(&dir).is_ok() {
        allow_dir(app, &dir);
    }
}

/// Ayarlardaki kaynak klasorleri ve yedek klasoru acar (acilista ve her kayittan sonra).
pub fn allow_settings(app: &AppHandle) {
    allow_thumbs(app);
    let Some(state) = app.try_state::<SettingsState>() else { return };
    let Ok(s) = state.settings.lock() else { return };
    s.source_folders.iter().chain([&s.backup_root]).for_each(|p| allow_dir(app, Path::new(p)));
}

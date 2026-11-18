//! Ayarlar komutlari: okuma, kaydetme, arayuz tercihi ve varsayilan uzanti gruplari.
//! Kayittan sonra watcher yeni ayarlarla yeniden kurulur.

use crate::settings::{self, AppSettings, ExtGroup, SettingsState};
use tauri::{AppHandle, State};

#[tauri::command]
pub(crate) fn get_settings(state: State<'_, SettingsState>) -> Result<AppSettings, String> {
    Ok(state.settings.lock().map_err(|e| e.to_string())?.clone())
}

/// Ayarlari dogrular ve kaydeder; temizlenmis (kaydedilen) hali dondurur.
#[tauri::command]
pub(crate) fn save_settings(app: AppHandle, state: State<'_, SettingsState>, mut settings: AppSettings) -> Result<AppSettings, String> {
    let saved = {
        let mut guard = state.settings.lock().map_err(|e| e.to_string())?;
        // Arayuz tercihleri yalnizca `set_ui_pref` ile degisir: eski bir taslak onlari ezemez.
        settings.ui_prefs = guard.ui_prefs.clone();
        let saved = settings::save(&state.dir, settings)?;
        *guard = saved.clone();
        saved
    };
    // Kilit birakildiktan sonra: watcher yeni ayarlarla kurulur, tepsi ve arayuz yeni durumu alir.
    crate::asset_scope::allow_settings(&app);
    crate::watcher::restart(&app);
    crate::automation::notify_changed(&app);
    crate::startup::sync(&app, saved.start_with_windows).map_err(|e| format!("Ayarlar kaydedildi. {e}"))?;
    Ok(saved)
}

/// Bugun sayfasindaki "Otomatik taşı" anahtari: yalnizca bu alani kaydeder, watcher'i yeniden kurar.
#[tauri::command]
pub(crate) fn set_auto_move(app: AppHandle, state: State<'_, SettingsState>, enabled: bool) -> Result<AppSettings, String> {
    let saved = {
        let mut guard = state.settings.lock().map_err(|e| e.to_string())?;
        let next = AppSettings { auto_move: enabled, ..guard.clone() };
        settings::write(&state.dir, &next)?;
        *guard = next.clone();
        next
    };
    crate::watcher::restart(&app);
    crate::automation::notify_changed(&app);
    Ok(saved)
}

/// Tek bir arayuz tercihini (tema, son klasorler, profiller ...) `settings.json` icine yazar;
/// `null` deger anahtari siler. Watcher'i yeniden baslatmaz.
#[tauri::command]
pub(crate) fn set_ui_pref(state: State<'_, SettingsState>, key: String, value: serde_json::Value) -> Result<(), String> {
    let mut guard = state.settings.lock().map_err(|e| e.to_string())?;
    set_pref(&mut guard, &state.dir, key, value)
}

pub(crate) fn set_pref(current: &mut AppSettings, dir: &std::path::Path, key: String, value: serde_json::Value) -> Result<(), String> {
    if key.is_empty() || key.len() > 64 {
        return Err("Geçersiz tercih anahtarı.".into());
    }
    let mut next = current.clone();
    if value.is_null() { next.ui_prefs.remove(&key); } else { next.ui_prefs.insert(key, value); }
    settings::write(dir, &next)?;
    *current = next;
    Ok(())
}

#[tauri::command]
pub(crate) fn default_ext_groups() -> Vec<ExtGroup> {
    settings::default_groups()
}

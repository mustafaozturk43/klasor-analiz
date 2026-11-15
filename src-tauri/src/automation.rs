//! Otomasyonun anlik durumu. "Acik/kapali" ayarlarda kalicidir; "duraklatildi"
//! ise yalnizca bu oturum icindir (tepsiden duraklatilip devam ettirilir).
//! Watcher (Asama 3) dosya tasimadan once `is_running` ile durumu kontrol eder.

use crate::settings::{AppSettings, SettingsState};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Default)]
pub struct AutomationState {
    pub paused: AtomicBool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationStatus {
    pub enabled: bool,
    pub paused: bool,
    pub running: bool,
    pub sources: usize,
    /// Tepside ve arayuzde gosterilen tek satirlik durum.
    pub label: String,
}

pub fn status_of(settings: &AppSettings, paused: bool) -> AutomationStatus {
    let enabled = settings.automation_enabled;
    let sources = settings.source_folders.len();
    let label = match (enabled, paused) {
        (false, _) => "Otomasyon kapalı".to_string(),
        (true, true) => "Otomasyon duraklatıldı".to_string(),
        (true, false) => format!("Otomasyon açık · {sources} kaynak izleniyor"),
    };
    AutomationStatus { enabled, paused, running: enabled && !paused, sources, label }
}

/// Ayarlar ve duraklatma durumundan guncel durum.
pub fn current(app: &AppHandle) -> AutomationStatus {
    let paused = app.state::<AutomationState>().paused.load(Ordering::SeqCst);
    let settings = app.state::<SettingsState>().settings.lock().map(|s| s.clone()).unwrap_or_default();
    status_of(&settings, paused)
}

/// Durum degisince tepsiyi gunceller ve arayuze `automation-status` olayi gonderir.
pub fn notify_changed(app: &AppHandle) {
    let status = current(app);
    crate::tray::refresh(app, &status);
    let _ = app.emit("automation-status", &status);
}

pub fn set_paused(app: &AppHandle, paused: bool) {
    app.state::<AutomationState>().paused.store(paused, Ordering::SeqCst);
    notify_changed(app);
}

#[tauri::command]
pub(crate) fn get_automation_status(app: AppHandle) -> AutomationStatus {
    current(&app)
}

#[tauri::command]
pub(crate) fn set_automation_paused(app: AppHandle, paused: bool) -> AutomationStatus {
    set_paused(&app, paused);
    current(&app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_label_reflects_enabled_and_paused() {
        let mut s = AppSettings { source_folders: vec!["A".into(), "B".into()], ..AppSettings::default() };
        assert_eq!(status_of(&s, false).label, "Otomasyon kapalı");
        assert!(!status_of(&s, true).running, "kapaliyken duraklatma onemsiz");
        s.automation_enabled = true;
        let on = status_of(&s, false);
        assert!(on.running);
        assert_eq!(on.label, "Otomasyon açık · 2 kaynak izleniyor");
        let paused = status_of(&s, true);
        assert!(!paused.running && paused.paused);
        assert_eq!(paused.label, "Otomasyon duraklatıldı");
    }
}

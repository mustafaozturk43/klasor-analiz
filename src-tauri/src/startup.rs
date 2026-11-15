//! Windows ile baslatma. Kaynak `settings.json` icindeki `start_with_windows` ayaridir;
//! bu modul onu Windows'un kullanici baslangic kaydina (HKCU ...\Run) yansitir.
//! Otomatik baslatilan kopya `--autostart` argumaniyla acilir ve otomasyon aciksa
//! penceresini gostermeden tepside calisir.

use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

pub const HIDDEN_ARG: &str = "--autostart";

pub fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![HIDDEN_ARG]))
}

/// Baslangic kaydini ayara esitler. Acikken her seferinde yeniden yazilir: tasinabilir
/// paket baska klasore tasindiysa kayit yeni EXE yolunu gosterir.
pub fn sync(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let auto = app.autolaunch();
    if enabled {
        return auto.enable().map_err(|e| format!("Windows başlangıç kaydı yazılamadı: {e}"));
    }
    if auto.is_enabled().unwrap_or(false) {
        return auto.disable().map_err(|e| format!("Windows başlangıç kaydı silinemedi: {e}"));
    }
    Ok(())
}

/// Bu acilis pencere gostermeden mi baslamali? Yalnizca Windows baslattiysa ve
/// otomasyon aciksa; aksi halde tepside yapacak is yoktur, pencere acilir.
pub fn starts_hidden<I: IntoIterator<Item = String>>(args: I, automation_enabled: bool) -> bool {
    automation_enabled && args.into_iter().any(|a| a == HIDDEN_ARG)
}

/// Acilista cagrilir: kaydi ayara esitler ve gerekmiyorsa pencereyi gosterir
/// (pencere `tauri.conf.json` icinde gizli olusturulur).
pub fn on_launch(app: &AppHandle, start_with_windows: bool, automation_enabled: bool) {
    let _ = sync(app, start_with_windows);
    if !starts_hidden(std::env::args(), automation_enabled) {
        crate::tray::show_main(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_only_when_windows_started_it_and_automation_is_on() {
        let args = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(starts_hidden(args(&["app.exe", HIDDEN_ARG]), true));
        assert!(!starts_hidden(args(&["app.exe", HIDDEN_ARG]), false));
        assert!(!starts_hidden(args(&["app.exe"]), true));
    }
}

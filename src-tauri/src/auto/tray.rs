//! Sistem tepsisi: durum satiri, Paneli ac, Duraklat/Devam et, Bugunun klasorunu
//! ac, Cikis. Otomasyon aciksa pencereyi kapatmak uygulamayi kapatmaz, tepsiye
//! gizler (izleme surer); kapaliysa kapatma dugmesi uygulamayi normal kapatir.

use crate::automation::{self, AutomationStatus, AutomationState};
use crate::settings::SettingsState;
use crate::{inbox_path, shell};
use std::path::Path;
use std::sync::atomic::Ordering;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Window, WindowEvent, Wry};

const OPEN: &str = "tray-open";
const TOGGLE: &str = "tray-toggle";
const TODAY: &str = "tray-today";
const QUIT: &str = "tray-quit";

/// Metni durumla degisen menu ogeleri.
pub struct TrayItems {
    status: MenuItem<Wry>,
    toggle: MenuItem<Wry>,
}

pub fn toggle_text(status: &AutomationStatus) -> &'static str {
    if status.paused { "Devam et" } else { "Duraklat" }
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let status = MenuItem::with_id(app, "tray-status", "", false, None::<&str>)?;
    let open = MenuItem::with_id(app, OPEN, "Paneli aç", true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, TOGGLE, "Duraklat", true, None::<&str>)?;
    let today = MenuItem::with_id(app, TODAY, "Bugünün klasörünü aç", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT, "Çıkış", true, None::<&str>)?;
    let (sep1, sep2) = (PredefinedMenuItem::separator(app)?, PredefinedMenuItem::separator(app)?);
    let menu = Menu::with_items(app, &[&status, &sep1, &open, &toggle, &today, &sep2, &quit])?;

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("Smart File Organizer")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| on_menu(app, event.id.as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    app.manage(TrayItems { status, toggle });
    refresh(app, &automation::current(app));
    Ok(())
}

/// Menu metinlerini duruma gore gunceller.
pub fn refresh(app: &AppHandle, status: &AutomationStatus) {
    if let Some(items) = app.try_state::<TrayItems>() {
        let _ = items.status.set_text(&status.label);
        let _ = items.toggle.set_text(toggle_text(status));
        let _ = items.toggle.set_enabled(status.enabled);
    }
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(format!("Smart File Organizer · {}", status.label)));
    }
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.webview_windows().into_values().next() {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn on_menu(app: &AppHandle, id: &str) {
    match id {
        OPEN => show_main(app),
        TOGGLE => {
            let paused = app.state::<AutomationState>().paused.load(Ordering::SeqCst);
            automation::set_paused(app, !paused);
        }
        TODAY => open_today(app),
        QUIT => app.exit(0),
        _ => {}
    }
}

/// Bugunun gelen kutusu klasorunu (yoksa olusturup) acar. Yedek klasoru
/// secilmemisse paneli Ayarlar sekmesinde acar.
fn open_today(app: &AppHandle) {
    let backup = app.state::<SettingsState>().settings.lock().map(|s| s.backup_root.clone()).unwrap_or_default();
    if backup.is_empty() || !Path::new(&backup).is_dir() {
        show_main(app);
        let _ = app.emit("open-module", "settings");
        return;
    }
    let dir = inbox_path::today_dir(Path::new(&backup));
    if std::fs::create_dir_all(&dir).is_ok() {
        let _ = shell::open_file(&dir.to_string_lossy());
    }
}

/// Otomasyon aciksa kapatma istegini tepsiye gizlemeye cevirir.
pub fn on_window_event(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event
        && automation::current(window.app_handle()).enabled
    {
        api.prevent_close();
        let _ = window.hide();
    }
}

//! Tek kopya kurali: uygulama zaten aciksa ikinci acilis yeni bir surec
//! baslatmaz; calisan kopyanin penceresi geri yuklenip one getirilir.
//! Iki kopya ayni gecmis dosyasini (move_history.json) ayni anda yazip
//! birbirinin kaydini ezebilecegi icin bu kural veri guvenligi icin gereklidir.

use tauri::{AppHandle, Manager, Runtime};

/// `Builder`'a ilk eklenti olarak kaydedilmelidir.
pub fn plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri_plugin_single_instance::init(|app, _args, _cwd| focus_existing(app))
}

fn focus_existing<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.webview_windows().into_values().next() {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

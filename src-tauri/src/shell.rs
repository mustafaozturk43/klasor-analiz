//! Dosya/klasoru Windows Gezgini'nde acar veya varsayilan uygulamayla calistirir.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

#[tauri::command]
pub(crate) fn reveal_in_explorer(path: String) -> Result<(), String> {
    reveal(&path)
}

#[tauri::command]
pub(crate) fn open_file_default(path: String) -> Result<(), String> {
    open_file(&path)
}

/// Explorer'i ham argumanla baslatir. Yol tirnak icinde verilir: aksi halde bosluklu
/// yollarda Explorer "Belgelerim" klasorune duser, virgullu adlar arguman sayilir.
fn explorer(arg: String) -> Result<(), String> {
    Command::new("explorer").raw_arg(arg).spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// Dosyanin bulundugu klasoru acip dosyayi vurgular.
pub fn reveal(path: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        return Err(format!("Dosya bulunamadi: {path}"));
    }
    explorer(format!("/select,\"{path}\""))
}

/// Uygulama icinden tek tikla calistirilmayan uzantilar (program, betik, kisayol).
pub fn is_executable(path: &str) -> bool {
    Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .is_some_and(|e| crate::models::DEFAULT_BANNED_EXTS.contains(&e.as_str()))
}

/// Dosyayi varsayilan uygulamayla acar (ornegin PDF'i okuyucuda); klasoru Explorer'da acar.
/// Calistirilabilir dosyalar acilmaz: kullanici onlari klasorde gorup kendisi baslatir.
pub fn open_file(path: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        return Err(format!("Dosya bulunamadi: {path}"));
    }
    if is_executable(path) {
        return Err("Çalıştırılabilir dosyalar uygulama içinden açılmaz. “Klasörde göster” ile bulup kendiniz başlatabilirsiniz.".into());
    }
    // `cmd /c start` yerine Explorer: yol kabuk tarafindan yorumlanmaz (`%VAR%`, `&`, `^`)
    // ve konsol penceresi acilmaz.
    explorer(format!("\"{path}\""))
}

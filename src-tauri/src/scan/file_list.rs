//! Klasor dosya listesi komutu (hash yok, sadece metadata).
//! Frontend'in "Dosyalari Listele" panelini besler.

use crate::models::FolderFile;
use crate::scanner;
use std::path::PathBuf;

/// Tum (veya kategoriye uyan) dosyalari düz liste olarak dondurur.
#[tauri::command(async)]
pub fn list_files(
    app: tauri::AppHandle,
    root_path: String,
    include_subdirs: Option<bool>,
    exclude_folders: Option<Vec<String>>,
    banned_extensions: Option<Vec<String>>,
    include_uncategorized: Option<bool>,
) -> Result<Vec<FolderFile>, String> {
    crate::asset_scope::allow_dir(&app, std::path::Path::new(&root_path));
    list_files_core(root_path, include_subdirs, exclude_folders, banned_extensions, include_uncategorized)
}

pub(crate) fn list_files_core(
    root_path: String,
    include_subdirs: Option<bool>,
    exclude_folders: Option<Vec<String>>,
    banned_extensions: Option<Vec<String>>,
    include_uncategorized: Option<bool>,
) -> Result<Vec<FolderFile>, String> {
    let include = include_subdirs.unwrap_or(true);
    let excludes = exclude_folders.unwrap_or_default();
    let banned = crate::models::normalize_exts(&banned_extensions.unwrap_or_default());
    let include_uncat = include_uncategorized.unwrap_or(false);

    scanner::list_files(
        &PathBuf::from(&root_path),
        include,
        &excludes,
        &banned,
        include_uncat,
    )
}

#[cfg(test)]
#[path = "file_list_tests.rs"]
mod tests;

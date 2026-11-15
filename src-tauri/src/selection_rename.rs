//! Harita > Klasordeki dosyalar: secilenleri sablonla adlandirma. Gelen kutusu
//! adlandirmasiyla ayni motor (`inbox_rename::plan_rename_in`); dosyalar acik
//! kok(ler)in icinde olmali. Uygulama `rename::apply_rename_plan` ile yapilir.

use crate::inbox_rename::{plan_rename_in, ListRenameOptions};
use crate::rename::{FileStamps, Pending, RenamePlan, RenameState};
use std::path::PathBuf;
use tauri::State;

pub const HISTORY_LABEL: &str = "Seçilenleri adlandır";

pub fn plan_selection_rename(roots: &[String], paths: &[String], opts: &ListRenameOptions) -> Result<(RenamePlan, FileStamps), String> {
    let bases: Vec<PathBuf> = roots.iter().filter(|r| !r.trim().is_empty()).map(PathBuf::from).collect();
    plan_rename_in(&bases, "Yalnızca açık klasördeki dosyalar adlandırılabilir", paths, opts)
}

/// Onizleme planini olusturur ve uygulama icin bekletir.
#[tauri::command(async)]
pub fn preview_selection_rename(state: State<'_, RenameState>, roots: Vec<String>, paths: Vec<String>, options: ListRenameOptions) -> Result<RenamePlan, String> {
    let (plan, stamps) = plan_selection_rename(&roots, &paths, &options)?;
    *state.pending.lock().map_err(|e| e.to_string())? = Some(Pending { plan: plan.clone(), stamps, label: HISTORY_LABEL });
    Ok(plan)
}

#[cfg(test)]
#[path = "selection_rename_tests.rs"]
mod tests;

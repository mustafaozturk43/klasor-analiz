//! "Türüne Göre Düzenle" komutlari: once onizleme (plan sunucuda bekletilir),
//! sonra ayni planin uygulanmasi. Tum tasimalar tek bir geri alma kaydidir.

use crate::models::ScanProgress;
use crate::organize_models::*;
use crate::{organize_plan, organize_rules, organize_walk, AppState};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::State;

/// Bekleyen (onizlenmis) tek plan.
#[derive(Default)]
pub struct OrganizeState {
    pending: Mutex<Option<OrganizePlan>>,
    /// Onizlemenin kendi iptal bayragi: taramanin bayragini sifirlamaz, ondan etkilenmez.
    cancel: std::sync::atomic::AtomicBool,
}

/// Suren duzenleme onizlemesini durdurur.
#[tauri::command]
pub(crate) fn cancel_organize(org: State<'_, OrganizeState>) {
    org.cancel.store(true, Ordering::SeqCst);
}

#[tauri::command(async)]
pub(crate) fn preview_organize(
    app: State<'_, AppState>,
    org: State<'_, OrganizeState>,
    root_path: String,
    options: OrganizeOptions,
    on_progress: Channel<ScanProgress>,
) -> Result<OrganizePlan, String> {
    let emit = |phase: &str, done: u64, total: u64| {
        let _ = on_progress.send(ScanProgress { phase: phase.into(), files_done: done, total_files: total });
    };
    preview_core(&app, &org, &root_path, &options, &emit)
}

#[tauri::command(async)]
pub(crate) fn apply_organize(
    app: State<'_, AppState>,
    org: State<'_, OrganizeState>,
    plan_id: String,
) -> Result<OrganizeOutcome, String> {
    apply_core(&app, &org, &plan_id)
}

pub(crate) fn preview_core(
    app: &AppState,
    org: &OrganizeState,
    root_path: &str,
    options: &OrganizeOptions,
    emit: &(dyn Fn(&str, u64, u64) + Sync),
) -> Result<OrganizePlan, String> {
    org.cancel.store(false, Ordering::SeqCst);
    let is_cancelled = || org.cancel.load(Ordering::Relaxed);
    let root = PathBuf::from(root_path);
    // Hatali kurallar uzun bir taramadan once bildirilir.
    organize_rules::RuleSet::new(&options.rules)?;
    if options.protect_projects && organize_rules::is_project_dir(&root) {
        return Err("Seçilen klasör bir yazılım projesine benziyor (package.json, .git …). \
                    Başka bir klasör seçin veya “Proje klasörlerini koru” seçeneğini kapatın."
            .into());
    }
    emit("scanning", 0, 0);
    let walked = organize_walk::collect(
        &root, options.include_subdirs, &options.exclude_folders, options.protect_projects,
        &|n| emit("scanning", n, 0), &is_cancelled,
    )?;
    let total_files = walked.files.len() as u64;
    emit("hashing", 0, 0);
    let entries = organize_plan::build(
        &root, walked.files, options, &app.hash_cache, &|d, t| emit("hashing", d, t), &is_cancelled,
    )?;
    let _ = app.hash_cache.save(&app.dir);
    emit("done", 1, 1);

    let plan = OrganizePlan {
        id: format!("ORG-{}", crate::now_ms()),
        root_path: root_path.to_string(),
        total_files,
        kinds: summarize(&entries),
        entries,
        protected_dirs: walked.protected_dirs,
        warnings: walked.warnings,
        remove_empty_dirs: options.remove_empty_dirs,
    };
    *org.pending.lock().map_err(|e| e.to_string())? = Some(plan.clone());
    Ok(plan)
}

/// Tur klasorunde sonucta duracak dosya sayisi/boyutu (karantina ve atlananlar
/// haric), kategori sirasinda.
fn summarize(entries: &[OrganizeEntry]) -> Vec<KindCount> {
    let mut map: BTreeMap<(String, String), (u64, u64)> = BTreeMap::new();
    for e in entries.iter().filter(|e| matches!(e.action, OrganizeAction::Move | OrganizeAction::InPlace)) {
        if let (Some(c), Some(k)) = (&e.category, &e.kind) {
            let slot = map.entry((c.clone(), k.clone())).or_default();
            slot.0 += 1;
            slot.1 += e.size;
        }
    }
    let mut out: Vec<KindCount> = map
        .into_iter()
        .map(|((category, kind), (count, bytes))| KindCount { category, kind, count, bytes })
        .collect();
    out.sort_by(|a, b| a.category.cmp(&b.category).then(b.count.cmp(&a.count)));
    out
}

pub(crate) fn apply_core(app: &AppState, org: &OrganizeState, plan_id: &str) -> Result<OrganizeOutcome, String> {
    let plan = {
        let mut pending = org.pending.lock().map_err(|e| e.to_string())?;
        match pending.as_ref() {
            Some(p) if p.id == plan_id => pending.take().unwrap(),
            Some(_) => return Err("Önizleme güncel değil; yeniden önizleyin.".into()),
            None => return Err("Uygulanacak önizleme yok; önce tarayın.".into()),
        }
    };
    crate::organize_apply::run_plan(app, &plan, "Türüne göre düzenleme")
}

#[cfg(test)]
#[path = "organize_tests.rs"]
mod tests;

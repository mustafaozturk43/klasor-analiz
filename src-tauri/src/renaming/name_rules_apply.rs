//! Kural tabanli adlandirmanin komutlari: onizleme plani sunucuda bekletilir, onaydan sonra
//! uygulanir. Klasorler de adlandirildigi icin `file_ops::transfer` (yalnizca dosya) yerine
//! `fs::rename` kullanilir; en derin oge once adlandirilir ki ust klasorun yolu bozulmasin.
//! Islem gecmise tek kayit olarak yazilir ve `undo` ile ters sirada geri alinir.

use crate::models::{MoveRecord, UndoRecord};
use crate::name_rules::NameRuleOptions;
use crate::name_rules_plan::{plan_name_rules, NamePlan};
use crate::settings::SettingsState;
use crate::{history, AppState};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::State;

/// Gecmisteki etiket; geri alma bu etikete bakarak klasor destekli yolu secer (`history_cmds`).
pub const HISTORY_LABEL: &str = "Kural ile adlandırma";

#[derive(Default)]
pub struct NameRulesState {
    pub(crate) pending: Mutex<Option<NamePlan>>,
}

#[tauri::command(async)]
pub(crate) fn preview_name_rules(state: State<'_, NameRulesState>, settings: State<'_, SettingsState>, root_path: String, options: NameRuleOptions) -> Result<NamePlan, String> {
    let protected: Vec<String> = {
        let s = settings.settings.lock().map_err(|e| e.to_string())?;
        s.source_folders.iter().cloned().chain([s.backup_root.clone()]).collect()
    };
    let plan = plan_name_rules(Path::new(&root_path), &options, &protected)?;
    *state.pending.lock().map_err(|e| e.to_string())? = Some(plan.clone());
    Ok(plan)
}

#[tauri::command(async)]
pub(crate) fn apply_name_rules(state: State<'_, NameRulesState>, app: State<'_, AppState>, plan_id: String) -> Result<usize, String> {
    let mut pending = state.pending.lock().map_err(|e| e.to_string())?;
    if pending.as_ref().map(|p| p.id.as_str()) != Some(plan_id.as_str()) {
        return Err("Adlandırma planı güncel değil; yeniden ön izleyin.".into());
    }
    apply_plan(pending.take().unwrap(), &app)
}

/// Plani uygular; degisen oge sayisini dondurur. Disk onizlemeden beri degistiyse hicbir seye dokunmaz.
pub(crate) fn apply_plan(plan: NamePlan, app: &AppState) -> Result<usize, String> {
    let mut changes: Vec<(PathBuf, PathBuf)> = plan.entries.iter().filter_map(|e| Some((PathBuf::from(&e.from), PathBuf::from(e.to.as_ref()?)))).collect();
    for (from, to) in &changes {
        if !from.exists() || (!case_only(from, to) && to.exists()) {
            return Err("Bir öğe veya hedef ad değişti; yeniden ön izleyin.".into());
        }
    }
    // En derin once: icerik, bulundugu klasor adlandirilmadan once adlandirilir.
    changes.sort_by_key(|(from, _)| std::cmp::Reverse(from.components().count()));
    let mut guard = app.history.lock().map_err(|e| e.to_string())?;
    history::push(&mut guard, UndoRecord { id: plan.id, timestamp: crate::now_ms(), tag: HISTORY_LABEL.into(), moves: Vec::new() });
    history::save(&app.dir, &guard)?;
    let mut count = 0;
    for (from, to) in changes {
        // Niyet once kaydedilir; geri alma, gerceklesmemis adimlari atlar.
        guard[0].moves.push(MoveRecord { from: from.to_string_lossy().into_owned(), to: to.to_string_lossy().into_owned() });
        history::save(&app.dir, &guard)?;
        if let Err(e) = rename_path(&from, &to) {
            guard[0].moves.pop();
            history::save(&app.dir, &guard)?;
            return Err(format!("{count} ad değişti; işlem durdu: {} → {e}. Geri al kullanılabilir.", from.display()));
        }
        count += 1;
    }
    Ok(count)
}

/// Kaydi ters sirada geri alir (once ust klasor eski adina doner, sonra icindekiler).
pub fn undo(record: &UndoRecord) -> Result<(), String> {
    for rec in record.moves.iter().rev() {
        let (from, to) = (Path::new(&rec.from), Path::new(&rec.to));
        if to.exists() {
            rename_path(to, from).map_err(|e| format!("Geri alınamadı: {} → {e}", to.display()))?;
        } else if !from.exists() {
            return Err(format!("Geri alınacak öğe bulunamadı: {}", rec.to));
        }
    }
    Ok(())
}

/// Ayni klasorde, yalnizca buyuk/kucuk harfi farkli iki ad mi? Windows bunlari ayni oge sayar.
fn case_only(from: &Path, to: &Path) -> bool {
    let name = |p: &Path| p.file_name().map(|n| n.to_string_lossy().to_lowercase());
    from != to && from.parent() == to.parent() && name(from) == name(to)
}

/// Uzerine yazmadan yeniden adlandirir (dosya veya klasor). Yalnizca harf degisiminde gecici ad uzerinden iki adim.
pub fn rename_path(from: &Path, to: &Path) -> Result<(), String> {
    if !case_only(from, to) {
        if to.exists() {
            return Err(format!("Hedef zaten var: {}", to.display()));
        }
        return fs::rename(from, to).map_err(|e| e.to_string());
    }
    let dir = from.parent().unwrap_or(Path::new(""));
    let temp = (0..).map(|n| dir.join(format!(".sfo-ad-{}-{n}", std::process::id()))).find(|p| !p.exists()).unwrap();
    fs::rename(from, &temp).map_err(|e| e.to_string())?;
    fs::rename(&temp, to).map_err(|e| {
        let _ = fs::rename(&temp, from); // Ikinci adim basarisizsa eski ada don.
        e.to_string()
    })
}

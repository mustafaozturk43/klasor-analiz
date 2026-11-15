//! Gelen kutusunun bir gunu: grup klasorlerine gore dosya ayrintilari, var olan
//! gunlerin listesi ve benzer adli dosyalar. Aksam Kontrolu paneli bunu gosterir.

use crate::inbox_dupes;
use crate::inbox_path::{day_dir, DAY_SUFFIX};
use crate::inspector::{self, FileDetail};
use crate::models::{UndoRecord, DEFAULT_SIMILARITY_THRESHOLD, INBOX_DIR};
use crate::settings::SettingsState;
use crate::watcher_core::HISTORY_TAG;
use crate::{thumbnail, AppState};
use chrono::NaiveDate;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tauri::{AppHandle, Manager, State};

/// Bir gunde listelenecek en fazla dosya (arayuz sanal liste kullanir).
const MAX_FILES: usize = 5000;
/// Grup klasorune konmamis dosyalar.
pub const LOOSE_GROUP: &str = "Gruplanmamış";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxGroup {
    pub name: String,
    pub bytes: u64,
    pub files: Vec<FileDetail>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxDay {
    pub date: String,
    pub files: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxListing {
    pub date: String,
    pub dir: String,
    pub exists: bool,
    pub days: Vec<InboxDay>,
    pub groups: Vec<InboxGroup>,
    pub total_files: usize,
    pub total_bytes: u64,
    pub truncated: bool,
}

fn files_in(dir: &Path) -> Vec<std::path::PathBuf> {
    fs::read_dir(dir).map(|rd| rd.flatten().map(|e| e.path()).collect()).unwrap_or_default()
}

/// `_GelenKutusu` altindaki gunler (yeniden eskiye) ve dosya sayilari.
pub fn list_days(backup_root: &Path) -> Vec<InboxDay> {
    let mut days: Vec<InboxDay> = files_in(&backup_root.join(INBOX_DIR))
        .into_iter()
        .filter(|p| p.is_dir())
        .filter_map(|p| {
            let name = p.file_name()?.to_string_lossy().into_owned();
            let date = name.strip_suffix(DAY_SUFFIX)?;
            NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
            let count = files_in(&p).iter().map(|e| if e.is_dir() { files_in(e).iter().filter(|f| f.is_file()).count() } else { usize::from(e.is_file()) }).sum();
            Some(InboxDay { date: date.to_string(), files: count })
        })
        .collect();
    days.sort_by(|a, b| b.date.cmp(&a.date));
    days
}

/// Gecmisteki gelen kutusu tasimalarindan hedef yol -> tasinma zamani.
fn arrivals(history: &[UndoRecord]) -> HashMap<String, u64> {
    history
        .iter()
        .filter(|r| r.tag == HISTORY_TAG)
        .flat_map(|r| r.moves.iter().map(move |m| (m.to.to_lowercase(), r.timestamp)))
        .collect()
}

pub fn list_core(backup_root: &Path, date: NaiveDate, history: &[UndoRecord], cache_dir: &Path) -> InboxListing {
    let dir = day_dir(backup_root, date);
    let arrived = arrivals(history);
    let mut all: Vec<FileDetail> = Vec::new();
    let mut truncated = false;
    let mut entries = files_in(&dir);
    entries.sort();
    for entry in entries {
        let (group, paths) = if entry.is_dir() {
            (entry.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), files_in(&entry))
        } else {
            (LOOSE_GROUP.to_string(), vec![entry])
        };
        for p in paths {
            if all.len() >= MAX_FILES {
                truncated = true;
                break;
            }
            let when = arrived.get(&p.to_string_lossy().to_lowercase()).copied();
            if let Some(d) = inspector::detail_of(&p, &group, when) {
                all.push(d);
            }
        }
    }
    inspector::mark_similar(&mut all, DEFAULT_SIMILARITY_THRESHOLD);
    inbox_dupes::mark_exact(&mut all, backup_root);
    crate::inbox_suggest::suggest(&mut all, backup_root);
    thumbnail::fill(&mut all, cache_dir);

    let mut groups: Vec<InboxGroup> = Vec::new();
    for f in all {
        match groups.iter_mut().find(|g| g.name == f.group) {
            Some(g) => {
                g.bytes += f.size;
                g.files.push(f);
            }
            None => groups.push(InboxGroup { name: f.group.clone(), bytes: f.size, files: vec![f] }),
        }
    }
    for g in &mut groups {
        // En yeni gelen ustte; gelis zamani bilinmiyorsa degistirilme zamani.
        g.files.sort_by_key(|f| std::cmp::Reverse(f.arrived_ms.or(f.modified_ms).unwrap_or(0)));
    }
    let total_files = groups.iter().map(|g| g.files.len()).sum();
    let total_bytes = groups.iter().map(|g| g.bytes).sum();
    InboxListing {
        date: date.format("%Y-%m-%d").to_string(),
        dir: dir.to_string_lossy().into_owned(),
        exists: dir.is_dir(),
        days: list_days(backup_root),
        groups,
        total_files,
        total_bytes,
        truncated,
    }
}

/// Bir gunun gelen kutusu (tarih verilmezse bugun, yerel saat).
#[tauri::command(async)]
pub(crate) fn list_inbox(app: AppHandle, state: State<'_, AppState>, settings: State<'_, SettingsState>, date: Option<String>) -> Result<InboxListing, String> {
    let backup = settings.settings.lock().map_err(|e| e.to_string())?.backup_root.clone();
    if backup.is_empty() {
        return Err("Önce Ayarlar sekmesinden yedek klasörünü seçin.".into());
    }
    crate::asset_scope::allow_dir(&app, Path::new(&backup));
    crate::asset_scope::allow_thumbs(&app);
    let day = match date.as_deref() {
        Some(d) => NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|_| format!("Geçersiz tarih: {d}"))?,
        None => chrono::Local::now().date_naive(),
    };
    let history = state.history.lock().map_err(|e| e.to_string())?.clone();
    let cache = app.path().app_cache_dir().unwrap_or_else(|_| std::env::temp_dir()).join("thumbs");
    // Oturumda bir kez: eski kucuk resimler onbellekte birikmesin.
    static PRUNE: std::sync::Once = std::sync::Once::new();
    PRUNE.call_once(|| { thumbnail::prune_cache(&cache, thumbnail::MAX_CACHE_AGE); });
    Ok(list_core(Path::new(&backup), day, &history, &cache))
}

//! Watcher'in Tauri'den bagimsiz cekirdegi: hangi dosya alinir, yazimi bitti mi,
//! nereye tasinir ve gecmise nasil yazilir. `watcher.rs` bunlari olaylara baglar.

use crate::file_ops::transfer;
use crate::inbox_path::day_dir;
use crate::models::{MoveRecord, UndoRecord};
use crate::naming::{unique_dest_with, Suffix};
use crate::organize_rules::is_temp_or_system;
use crate::settings::AppSettings;
use chrono::NaiveDate;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

/// Dosyanin boyutu ve degistirilme zamani bu sure boyunca sabit kalmali.
pub const STABLE_FOR: Duration = Duration::from_secs(2);
pub const HISTORY_TAG: &str = "Otomatik yerleştirme · Gelen kutusu";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxMove {
    pub from: String,
    pub to: String,
    pub group: String,
}

/// Gelen kutusuna alinabilecek bir dosya mi? (klasor, gecici/sistem dosyasi degil)
pub fn eligible(path: &Path) -> bool {
    let Some(name) = path.file_name().map(|n| n.to_string_lossy().into_owned()) else { return false };
    path.is_file() && !is_temp_or_system(&name) && !name.starts_with('.')
}

/// Dosyanin gidecegi grup ve klasor; hicbir gruba uymuyorsa None (yerinde kalir).
pub fn target_dir(settings: &AppSettings, date: NaiveDate, path: &Path) -> Option<(String, PathBuf)> {
    let ext = path.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
    let group = settings.group_of(&ext)?.to_string();
    let dir = day_dir(Path::new(&settings.backup_root), date).join(&group);
    Some((group, dir))
}

/// Iki klasor yolu ayni mi? Windows'ta buyuk/kucuk harf, `/` ile `\`, sondaki
/// ayirac ve `\\?\` on eki farki onemsizdir (ayarlardaki yol ile `notify`'in
/// bildirdigi yol bu ayrintilarda ayrisabilir).
pub fn same_dir(a: &Path, b: &Path) -> bool {
    let key = |p: &Path| {
        let s = p.to_string_lossy().replace('/', "\\").to_lowercase();
        s.trim_start_matches(r"\\?\").trim_end_matches('\\').to_string()
    };
    key(a) == key(b)
}

/// Tasimanin hedefini belirler (grup klasorunu olusturur, dosyaya dokunmaz);
/// ayni ad varsa `ad_1.uzanti`. Hicbir gruba uymuyorsa None.
pub fn plan_inbox_move(settings: &AppSettings, date: NaiveDate, path: &Path) -> Result<Option<InboxMove>, String> {
    let Some((group, dir)) = target_dir(settings, date, path) else { return Ok(None) };
    let name = path.file_name().ok_or("Dosya adı okunamadı.")?.to_string_lossy().into_owned();
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let dest = unique_dest_with(&dir, &name, Suffix::Underscore);
    Ok(Some(InboxMove { from: path.to_string_lossy().into_owned(), to: dest.to_string_lossy().into_owned(), group }))
}

/// Dosyayi gunun gelen kutusuna tasir (gecmis kaydi olmadan; watcher `plan_inbox_move`
/// ile once gecmise yazar, sonra tasir).
pub fn move_to_inbox(settings: &AppSettings, date: NaiveDate, path: &Path) -> Result<Option<InboxMove>, String> {
    let Some(m) = plan_inbox_move(settings, date, path)? else { return Ok(None) };
    transfer(path, Path::new(&m.to))?;
    Ok(Some(m))
}

/// Tasinamayan dosyanin az once yazilan gecmis kaydini geri alir.
pub fn unrecord(history: &mut Vec<UndoRecord>, m: &InboxMove) {
    let Some(first) = history.first_mut().filter(|r| r.tag == HISTORY_TAG) else { return };
    if first.moves.last().is_some_and(|l| l.from == m.from && l.to == m.to) {
        first.moves.pop();
    }
    if first.moves.is_empty() {
        history.remove(0);
    }
}

/// Yazimi suren dosyalari bekletir: boyut ve mtime `STABLE_FOR` boyunca degismeyince hazirdir.
#[derive(Default)]
pub struct Pending {
    seen: HashMap<PathBuf, (u64, Option<SystemTime>, Instant)>,
}

impl Pending {
    pub fn observe(&mut self, path: PathBuf, now: Instant) {
        self.seen.entry(path).or_insert((u64::MAX, None, now));
    }

    pub fn len(&self) -> usize {
        self.seen.len()
    }

    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }

    /// Hazir dosyalari listeden cikarip dondurur. Kaybolan dosyalar unutulur,
    /// degisenlerin sayaci sifirlanir.
    pub fn take_ready(&mut self, now: Instant, stable_for: Duration) -> Vec<PathBuf> {
        let mut ready = Vec::new();
        self.seen.retain(|path, (size, mtime, since)| {
            let Ok(meta) = std::fs::metadata(path) else { return false };
            if !meta.is_file() {
                return false;
            }
            let (s, m) = (meta.len(), meta.modified().ok());
            if s != *size || m != *mtime {
                (*size, *mtime, *since) = (s, m, now);
                return true;
            }
            if now.duration_since(*since) >= stable_for {
                ready.push(path.clone());
                return false;
            }
            true
        });
        ready.sort();
        ready
    }
}

/// Tasimalari gecmise yazar. Ayni dakikadaki gelen kutusu tasimalari tek kayitta
/// toplanir; boylece gecmis (son `MAX_HISTORY` islem) yuzlerce satirla dolmaz.
pub fn record_moves(history: &mut Vec<UndoRecord>, moves: &[InboxMove], now_ms: u64) {
    if moves.is_empty() {
        return;
    }
    let records = moves.iter().map(|m| MoveRecord { from: m.from.clone(), to: m.to.clone() });
    if let Some(first) = history.first_mut()
        && first.tag == HISTORY_TAG
        && first.timestamp / 60_000 == now_ms / 60_000
    {
        first.moves.extend(records);
        return;
    }
    crate::history::push(history, UndoRecord { id: format!("IN-{now_ms}"), timestamp: now_ms, tag: HISTORY_TAG.into(), moves: records.collect() });
}

#[cfg(test)]
#[path = "watcher_core_tests.rs"]
mod tests;

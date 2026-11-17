//! Klasor-Dosya Islemleri > Tarihe Gore: klasordeki dosyalari tarihlerine gore
//! `Yıl\Ay` (ör. `2026\01`) alt klasorlerine yerlestirme plani. Tarih: cekim
//! tarihi (fotograf/video), yoksa olusturma tarihi, o da yoksa `Tarihsiz`.
//! Plan diske dokunmaz; ayni ad cakisirsa `ad (1).uzanti`.

use crate::models::{normalize_exts, MoveRecord, INBOX_DIR, QUARANTINE_DIR};
use crate::rename::FileStamps;
use crate::selection_paths::free_dest;
use crate::watcher_core::eligible;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const UNDATED: &str = "Tarihsiz";
const SAMPLES: usize = 3;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateGroupOptions {
    /// Bos ise tum uzantilar.
    #[serde(default)]
    pub extensions: Vec<String>,
    /// Alt klasorlerdeki dosyalar da toplansin mi?
    #[serde(default)]
    pub recursive: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DateBucket {
    pub folder: String,
    pub count: usize,
    pub samples: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DateGroupPlan {
    pub id: String,
    pub root: String,
    pub entries: Vec<MoveRecord>,
    pub buckets: Vec<DateBucket>,
    /// Zaten dogru klasorde olan dosyalar.
    pub in_place: usize,
    /// Tarihi cekim tarihinden alinan dosyalar.
    pub from_taken: usize,
    pub renamed: usize,
    pub created_dirs: Vec<String>,
}

/// Unix saniyeden yerel `YYYY\MM`; tarih yoksa `Tarihsiz`.
pub fn folder_for(secs: Option<u64>) -> String {
    secs.and_then(|s| DateTime::from_timestamp(s as i64, 0))
        .map(|d| d.with_timezone(&Local).format("%Y\\%m").to_string())
        .unwrap_or_else(|| UNDATED.to_string())
}

fn created_secs(meta: &fs::Metadata) -> Option<u64> {
    meta.created().ok()?.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs()).filter(|s| *s > 0)
}

fn skip_dir(name: &str) -> bool {
    name.starts_with('.') || name.eq_ignore_ascii_case(INBOX_DIR) || name.eq_ignore_ascii_case(QUARANTINE_DIR)
}

fn collect(dir: &Path, recursive: bool, exts: &[String], out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut entries: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            if recursive && !skip_dir(&name) {
                collect(&p, recursive, exts, out);
            }
        } else if eligible(&p) {
            let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
            if exts.is_empty() || exts.contains(&ext) {
                out.push(p);
            }
        }
    }
}

/// `taken`: cekim tarihi okuyucu (testlerde sahte); `progress(done, total)`.
pub fn plan_date_group(root: &Path, opts: &DateGroupOptions, taken: &dyn Fn(&Path) -> Option<u64>, progress: &dyn Fn(u64, u64)) -> Result<(DateGroupPlan, FileStamps), String> {
    if !root.is_dir() {
        return Err("Önce var olan bir klasör seçin.".into());
    }
    let exts = normalize_exts(&opts.extensions);
    let mut files = Vec::new();
    collect(root, opts.recursive, &exts, &mut files);
    let total = files.len() as u64;
    let (mut entries, mut stamps, mut reserved) = (Vec::new(), Vec::new(), HashSet::new());
    let mut buckets: BTreeMap<String, DateBucket> = BTreeMap::new();
    let (mut in_place, mut from_taken, mut renamed) = (0, 0, 0);
    let mut created: Vec<String> = Vec::new();
    for (i, p) in files.iter().enumerate() {
        progress(i as u64, total);
        let Ok(meta) = fs::metadata(p) else { continue };
        let shot = taken(p);
        from_taken += usize::from(shot.is_some());
        let folder = folder_for(shot.or_else(|| created_secs(&meta)));
        let dir = folder.split('\\').fold(root.to_path_buf(), |d, seg| d.join(seg));
        if p.parent() == Some(dir.as_path()) {
            in_place += 1;
            continue;
        }
        let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let dest = free_dest(&dir, &name, &mut reserved);
        renamed += usize::from(dest.file_name().is_some_and(|n| n.to_string_lossy() != name));
        for d in [dir.parent().filter(|d| *d != root), Some(dir.as_path())].into_iter().flatten() {
            let s = d.to_string_lossy().into_owned();
            if !d.exists() && !created.contains(&s) {
                created.push(s);
            }
        }
        let b = buckets.entry(folder.clone()).or_insert_with(|| DateBucket { folder, count: 0, samples: Vec::new() });
        b.count += 1;
        if b.samples.len() < SAMPLES {
            b.samples.push(name);
        }
        stamps.push((meta.len(), meta.modified().ok()));
        entries.push(MoveRecord { from: p.to_string_lossy().into_owned(), to: dest.to_string_lossy().into_owned() });
    }
    progress(total, total);
    let id = format!("DG-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos());
    let plan = DateGroupPlan { id, root: root.to_string_lossy().into_owned(), entries, buckets: buckets.into_values().collect(), in_place, from_taken, renamed, created_dirs: created };
    Ok((plan, stamps))
}

#[cfg(test)]
#[path = "date_group_tests.rs"]
mod tests;

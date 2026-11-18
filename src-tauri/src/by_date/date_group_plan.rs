//! Klasor-Dosya Islemleri > Tarihe Gore: klasordeki dosyalari tarihlerine gore
//! `Yıl\Ay` (ör. `2026\01`) alt klasorlerine yerlestirme plani. Tarih: cekim
//! tarihi (fotograf/video), yoksa olusturma tarihi, o da yoksa `Tarihsiz`.
//! Plan diske dokunmaz; ayni ad cakisirsa `ad (1).uzanti`.

use crate::date_group_index::{INDEX_FILE, LEGACY_INDEX_FILE};
use crate::date_group_layout::{year_month, DateLayout, FolderNamer};
use crate::models::{normalize_exts, MoveRecord, INBOX_DIR, QUARANTINE_DIR};
use crate::rename::FileStamps;
use crate::selection_paths::free_dest;
use crate::watcher_core::eligible;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub use crate::date_group_layout::UNDATED;
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
    /// Klasor duzeni: Yıl\Ay (varsayilan), yalniz Yıl-Ay, sira numarali yil.
    #[serde(default)]
    pub layout: DateLayout,
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
    FolderNamer::new(DateLayout::YearMonth, Path::new("")).folder(secs)
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
        } else if eligible(&p) && !p.file_name().is_some_and(|n| n.eq_ignore_ascii_case(INDEX_FILE) || n.eq_ignore_ascii_case(LEGACY_INDEX_FILE)) {
            let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
            if exts.is_empty() || exts.contains(&ext) {
                out.push(p);
            }
        }
    }
}

/// `taken`: cekim tarihi okuyucu (testlerde sahte); `progress(done, total)`.
pub fn plan_date_group(root: &Path, opts: &DateGroupOptions, taken: &(dyn Fn(&Path) -> Option<u64> + Sync), progress: &(dyn Fn(u64, u64) + Sync)) -> Result<(DateGroupPlan, FileStamps), String> {
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
    // Cekim tarihi okumasi (Windows shell) dosya basina yavas: paralel okunur, sira korunur.
    let done = AtomicU64::new(0);
    let read: Vec<_> = files
        .par_iter()
        .filter_map(|p| {
            let meta = fs::metadata(p).ok()?;
            let shot = taken(p);
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            if n % 8 == 0 {
                progress(n, total);
            }
            Some((p, meta, shot))
        })
        .collect();
    let mut dated = Vec::with_capacity(read.len());
    for (p, meta, shot) in read {
        from_taken += usize::from(shot.is_some());
        let secs = shot.or_else(|| created_secs(&meta));
        dated.push((p, meta, secs));
    }
    let mut namer = FolderNamer::new(opts.layout, root);
    namer.register(dated.iter().filter_map(|(_, _, s)| year_month(*s).map(|(y, _)| y)).collect());
    for (p, meta, secs) in dated {
        let folder = namer.folder(secs);
        let dir = folder.split('\\').fold(root.to_path_buf(), |d, seg| d.join(seg));
        let p = p.as_path();
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

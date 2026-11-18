//! Otomatik tasima kapaliyken izlenen (kaynak) klasorlerde bekleyen dosyalar:
//! secili gunun dosyalari, son gunlerin sayilari ve elle tasima / Geri Donusum
//! Kutusu icin kaynak dogrulamasi. Yalnizca kaynak klasorlerin ilk seviyesi.

use crate::inbox_list::InboxDay;
use crate::inspector::{self, FileDetail};
use crate::path_guard::canonical;
use crate::watcher_core::{eligible, same_dir};
use chrono::{DateTime, Local, NaiveDate};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Gun seciciye eklenecek en eski gun (bugunden geriye).
const DAY_WINDOW: i64 = 30;

/// Bekleyen dosyalarin grup adi: "İzlenen · Desktop".
pub fn group_name(source: &Path) -> String {
    let leaf = source.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| source.to_string_lossy().into_owned());
    format!("İzlenen · {leaf}")
}

/// Dosyanin olusturulma gunu (yerel saat).
pub fn created_day(path: &Path) -> Option<NaiveDate> {
    let created = fs::metadata(path).ok()?.created().ok()?;
    Some(DateTime::<Local>::from(created).date_naive())
}

fn top_files(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(dir).map(|rd| rd.flatten().map(|e| e.path()).filter(|p| eligible(p)).collect()).unwrap_or_default();
    v.sort();
    v
}

/// Kaynak klasorlerde bekleyen dosyalar (`pending = true`); `day` verilirse yalnizca o gun olusturulanlar.
pub fn source_files(sources: &[PathBuf], day: Option<NaiveDate>) -> Vec<FileDetail> {
    let mut out = Vec::new();
    for src in sources {
        let group = group_name(src);
        for p in top_files(src).into_iter().filter(|p| day.is_none() || created_day(p) == day) {
            if let Some(mut d) = inspector::detail_of(&p, &group, None) {
                d.arrived_ms = d.created_ms;
                d.pending = true;
                out.push(d);
            }
        }
    }
    out
}

/// Son 30 gunde kaynak klasorlere gelen dosya sayilari, gune gore.
pub fn source_days(sources: &[PathBuf], today: NaiveDate) -> Vec<InboxDay> {
    let mut counts: BTreeMap<NaiveDate, usize> = BTreeMap::new();
    for p in sources.iter().flat_map(|s| top_files(s)) {
        if let Some(d) = created_day(&p).filter(|d| (today - *d).num_days() <= DAY_WINDOW) {
            *counts.entry(d).or_default() += 1;
        }
    }
    counts.into_iter().map(|(d, files)| InboxDay { date: d.format("%Y-%m-%d").to_string(), files }).collect()
}

/// Iki gun listesini birlestirir (ayni gunun sayilari toplanir), yeniden eskiye.
pub fn merge_days(mut a: Vec<InboxDay>, b: Vec<InboxDay>) -> Vec<InboxDay> {
    for d in b {
        match a.iter_mut().find(|x| x.date == d.date) {
            Some(x) => x.files += d.files,
            None => a.push(d),
        }
    }
    a.sort_by(|x, y| y.date.cmp(&x.date));
    a
}

/// Yol bir kaynak klasorun dogrudan icindeki uygun bir dosya mi?
pub fn is_pending_source(sources: &[PathBuf], path: &Path) -> bool {
    let Some(parent) = path.parent() else { return false };
    let parent_real = canonical(parent).ok();
    eligible(path) && sources.iter().any(|s| same_dir(s, parent) || (parent_real.is_some() && canonical(s).ok() == parent_real))
}

#[cfg(test)]
#[path = "inbox_sources_tests.rs"]
mod tests;

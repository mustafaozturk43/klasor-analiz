//! Gelen kutusu dosyalarinin birebir kopyasi: once ayni boyut, sonra huni (ilk 4 KB, sonra tam
//! BLAKE3; 500 MB ve ustu icin orneklemeli). Once yedek klasorundeki diger dosyalara, sonra
//! gelen kutusundaki dosyalarin kendi aralarina bakilir.

use crate::hasher;
use crate::inspector::FileDetail;
use crate::models::{INBOX_DIR, LARGE_FILE_THRESHOLD, QUARANTINE_DIR};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Yedek klasorunde en fazla bu kadar girdiye bakilir (cok buyuk agaclarda listeyi yavaslatmasin).
const MAX_VISITED: usize = 50_000;
const MAX_DEPTH: usize = 8;

/// Yedek klasorunde, istenen boyutlardaki dosyalar (`_GelenKutusu` ve karantina haric).
fn index_sizes(root: &Path, wanted: &HashSet<u64>) -> HashMap<u64, Vec<PathBuf>> {
    let mut found: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    let mut visited = 0usize;
    while let Some((dir, depth)) = stack.pop() {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for entry in rd.flatten() {
            visited += 1;
            if visited > MAX_VISITED {
                return found;
            }
            let Ok(kind) = entry.file_type() else { continue };
            if kind.is_symlink() {
                continue;
            }
            let path = entry.path();
            if kind.is_dir() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if depth < MAX_DEPTH && name != INBOX_DIR.to_lowercase() && name != QUARANTINE_DIR.to_lowercase() {
                    stack.push((path, depth + 1));
                }
            } else if let Some(len) = entry.metadata().ok().map(|m| m.len()).filter(|l| wanted.contains(l)) {
                found.entry(len).or_default().push(path);
            }
        }
    }
    found
}

/// Listelemeler arasi kalici (oturum boyu) hash onbellegi: gelen kutusu her
/// `inbox-updated` olayinda yeniden listelenir; degismeyen dosyalar yeniden okunmaz.
static SHARED: std::sync::OnceLock<crate::hash_cache::HashCache> = std::sync::OnceLock::new();

fn mtime_secs(path: &Path) -> Option<u64> {
    fs::metadata(path).ok()?.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs())
}

/// Tam icerik kimligi (buyuk dosyada ornekleme); sonuc onbellege yazilir.
fn identity(cache: &mut HashMap<PathBuf, Option<String>>, path: &Path, size: u64) -> Option<String> {
    cache
        .entry(path.to_path_buf())
        .or_insert_with(|| {
            let shared = SHARED.get_or_init(crate::hash_cache::HashCache::new);
            let (key, mtime) = (path.to_string_lossy(), mtime_secs(path));
            if let Some(hit) = mtime.and_then(|m| shared.get_full(&key, size, m)) {
                return Some(hit);
            }
            let hash = if size >= LARGE_FILE_THRESHOLD { hasher::hash_large(path, size) } else { hasher::hash_full(path) }.ok();
            if let (Some(h), Some(m)) = (&hash, mtime) {
                shared.set_full(&key, size, m, h.clone());
            }
            hash
        })
        .clone()
}

/// Ayni boyuttaki iki dosya birebir mi? Ucuz 4 KB on-hash tutmazsa tam hash'e inilmez.
fn same(cache: &mut HashMap<PathBuf, Option<String>>, a: &Path, b: &Path, size: u64) -> bool {
    if size < LARGE_FILE_THRESHOLD && hasher::hash_preview(a).ok() != hasher::hash_preview(b).ok() {
        return false;
    }
    match (identity(cache, a, size), identity(cache, b, size)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

/// Yedek klasorune gore goreli ust klasor adi ("Arsiv" ya da "Projeler/2026").
fn folder_label(root: &Path, file: &Path) -> String {
    let parent = file.parent().unwrap_or(root);
    let rel = parent.strip_prefix(root).unwrap_or(parent);
    let text = rel.to_string_lossy().replace('\\', "/");
    if text.is_empty() { root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default() } else { text }
}

/// Birebir kopyalari isaretler. Yedek klasorunde ayni icerik varsa `duplicate_in` o klasorun adidir;
/// yoksa gelen kutusundaki dosyalar kendi aralarinda eslesir (adi en kisa olan asil sayilir,
/// digerlerinin `duplicate_of` alani ona isaret eder, `duplicate_in` bos kalir).
pub fn mark_exact(files: &mut [FileDetail], backup_root: &Path) {
    let wanted: HashSet<u64> = files.iter().map(|f| f.size).filter(|s| *s > 0).collect();
    if wanted.is_empty() {
        return;
    }
    let outside = index_sizes(backup_root, &wanted);
    let mut cache = HashMap::new();
    for f in files.iter_mut() {
        let own = PathBuf::from(&f.path);
        let hit = outside.get(&f.size).and_then(|c| c.iter().find(|p| **p != own && same(&mut cache, &own, p, f.size)));
        if let Some(p) = hit {
            f.duplicate_of = Some(p.to_string_lossy().into_owned());
            f.duplicate_in = Some(folder_label(backup_root, p));
        }
    }
    let mut order: Vec<usize> = (0..files.len()).filter(|i| files[*i].size > 0).collect();
    order.sort_by(|a, b| (files[*a].name.chars().count(), &files[*a].path).cmp(&(files[*b].name.chars().count(), &files[*b].path)));
    let mut originals: Vec<usize> = Vec::new();
    for i in order {
        if files[i].duplicate_of.is_some() {
            continue;
        }
        let own = PathBuf::from(&files[i].path);
        let size = files[i].size;
        let twin = originals.iter().copied().find(|o| files[*o].size == size && same(&mut cache, &own, Path::new(&files[*o].path), size));
        match twin {
            Some(o) => files[i].duplicate_of = Some(files[o].path.clone()),
            None => originals.push(i),
        }
    }
}

#[cfg(test)]
#[path = "inbox_dupes_tests.rs"]
mod tests;

//! Benzer gorseller: her resmin 64 bitlik "fark parmak izi" (dHash) hesaplanir; parmak izleri
//! arasindaki Hamming mesafesi `max_distance` ya da daha azsa resimler ayni gruba girer
//! (yeniden boyutlandirilmis, sikistirilmis ya da hafifce duzenlenmis kopyalar). Yalniz okur;
//! hicbir dosyaya dokunmaz. Taşıma/karantina mevcut karantina komutlariyla yapilir.

use crate::models::{ScanProgress, QUARANTINE_DIR};
use crate::scanner::walker;
use image::imageops::FilterType;
use image::DynamicImage;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::UNIX_EPOCH;
use tauri::ipc::Channel;
use tauri::State;

/// Cozulebilen bicimler (`Cargo.toml` image ozellikleriyle ayni; heic yok).
const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "bmp", "tif", "tiff"];
/// O(n^2) karsilastirma icin ust sinir.
pub const MAX_IMAGES: usize = 30_000;
const MAX_GROUPS: usize = 500;
const MAX_DISTANCE: u32 = 16;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualFile {
    pub path: String,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub mtime: u64,
    /// Grubun ilk (en iyi) dosyasina Hamming mesafesi; ilk dosya icin 0.
    pub distance: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualGroup {
    pub id: String,
    pub files: Vec<VisualFile>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualReport {
    pub root_path: String,
    pub scanned: usize,
    /// Cozulemeyen (bozuk ya da desteklenmeyen) resimler.
    pub unreadable: usize,
    pub groups: Vec<VisualGroup>,
}

/// (yol, boyut, degisiklik sn) -> (dHash, genislik, yukseklik); ayni dosya tekrar cozulmez.
#[derive(Default)]
pub struct VisualState {
    cache: Mutex<HashMap<(String, u64, u64), (u64, u32, u32)>>,
}

/// 9x8 gri tonlamaya kucultup her satirda komsu pikselleri karsilastirir (64 bit).
pub fn dhash(img: &DynamicImage) -> u64 {
    let g = img.resize_exact(9, 8, FilterType::Triangle).to_luma8();
    let mut h = 0u64;
    for y in 0..8 {
        for x in 0..8 {
            h = (h << 1) | u64::from(g.get_pixel(x, y)[0] > g.get_pixel(x + 1, y)[0]);
        }
    }
    h
}

struct Found {
    path: String,
    size: u64,
    mtime: u64,
    hash: u64,
    width: u32,
    height: u32,
}

fn collect(root: &Path, include_subdirs: bool, exclude: &[String]) -> Vec<(String, u64, u64)> {
    let mut excluded: HashSet<String> = exclude.iter().map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
    excluded.insert(QUARANTINE_DIR.to_lowercase());
    let walk = walker(root).skip_hidden(true).max_depth(if include_subdirs { usize::MAX } else { 1 }).process_read_dir(move |_, _, _, children| {
        children.retain(|c| match c {
            Ok(e) => !(e.file_type.is_dir() && excluded.contains(&e.file_name.to_string_lossy().to_lowercase())),
            Err(_) => true,
        });
    });
    let mut out = Vec::new();
    for entry in walk.into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        if !IMAGE_EXTS.contains(&ext.as_str()) {
            continue;
        }
        let Ok(meta) = entry.metadata() else { continue };
        let mtime = meta.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs());
        out.push((path.to_string_lossy().into_owned(), meta.len(), mtime));
    }
    out
}

/// Birlesik-ayrik kume: `a` ve `b` ayni gruba.
fn find(parent: &mut [usize], mut x: usize) -> usize {
    while parent[x] != x {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    x
}

pub fn build(
    root: &Path,
    include_subdirs: bool,
    exclude: &[String],
    max_distance: u32,
    state: &VisualState,
    progress: &(dyn Fn(u64, u64) + Sync),
) -> Result<VisualReport, String> {
    if !root.is_dir() {
        return Err(format!("Klasör bulunamadı: {}", root.display()));
    }
    let max_distance = max_distance.min(MAX_DISTANCE);
    let files = collect(root, include_subdirs, exclude);
    if files.len() > MAX_IMAGES {
        return Err(format!("{} görsel var; en fazla {MAX_IMAGES} görsel karşılaştırılabilir. Daha dar bir klasör seçin.", files.len()));
    }
    let total = files.len() as u64;
    let done = AtomicU64::new(0);
    let found: Vec<Option<Found>> = files
        .par_iter()
        .map(|(path, size, mtime)| {
            let key = (path.clone(), *size, *mtime);
            let cached = state.cache.lock().ok().and_then(|c| c.get(&key).copied());
            let hit = cached.or_else(|| {
                let img = image::open(path).ok()?;
                let v = (dhash(&img), img.width(), img.height());
                if let Ok(mut c) = state.cache.lock() {
                    c.insert(key, v);
                }
                Some(v)
            });
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            if n % 8 == 0 {
                progress(n, total);
            }
            hit.map(|(hash, width, height)| Found { path: path.clone(), size: *size, mtime: *mtime, hash, width, height })
        })
        .collect();
    progress(total, total);
    let unreadable = found.iter().filter(|f| f.is_none()).count();
    let items: Vec<Found> = found.into_iter().flatten().collect();

    let mut parent: Vec<usize> = (0..items.len()).collect();
    for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            if (items[i].hash ^ items[j].hash).count_ones() <= max_distance {
                let (a, b) = (find(&mut parent, i), find(&mut parent, j));
                if a != b {
                    parent[b] = a;
                }
            }
        }
    }
    let mut buckets: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..items.len() {
        let r = find(&mut parent, i);
        buckets.entry(r).or_default().push(i);
    }
    let mut groups: Vec<Vec<usize>> = buckets.into_values().filter(|g| g.len() > 1).collect();
    for g in &mut groups {
        // En iyi (en yuksek cozunurluk, sonra en buyuk dosya) basta: varsayilan olarak o tutulur.
        g.sort_by(|&a, &b| {
            let (x, y) = (&items[a], &items[b]);
            (u64::from(y.width) * u64::from(y.height), y.size, &x.path).cmp(&(u64::from(x.width) * u64::from(x.height), x.size, &y.path))
        });
    }
    groups.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| items[a[0]].path.cmp(&items[b[0]].path)));
    groups.truncate(MAX_GROUPS);
    let groups = groups
        .into_iter()
        .enumerate()
        .map(|(n, g)| {
            let best = items[g[0]].hash;
            VisualGroup {
                id: format!("VG-{}", n + 1),
                files: g
                    .iter()
                    .map(|&i| {
                        let f = &items[i];
                        VisualFile { path: f.path.clone(), size: f.size, width: f.width, height: f.height, mtime: f.mtime, distance: (f.hash ^ best).count_ones() }
                    })
                    .collect(),
            }
        })
        .collect();
    Ok(VisualReport { root_path: root.to_string_lossy().into_owned(), scanned: items.len() + unreadable, unreadable, groups })
}

#[tauri::command(async)]
pub fn find_similar_images(
    state: State<'_, VisualState>,
    root_path: String,
    include_subdirs: bool,
    exclude_folders: Vec<String>,
    max_distance: u32,
    on_progress: Channel<ScanProgress>,
) -> Result<VisualReport, String> {
    let send = |done: u64, total: u64| {
        let _ = on_progress.send(ScanProgress { phase: "hashing".into(), files_done: done, total_files: total });
    };
    build(Path::new(&root_path), include_subdirs, &exclude_folders, max_distance, &state, &send)
}

#[cfg(test)]
#[path = "visual_tests.rs"]
mod tests;

//! Gruplama motoru: rayon ile paralel Exact (birebir kopya) ve Similar (isim benzerligi).
//! Similar kumeleme `similarity` modulundedir (optimize).
//! Uzun islemler `is_cancelled` bayragi ile kullanici tarafindan durdurulabilir.

use crate::hash_cache::HashCache;
use crate::hasher;
use crate::models::{
    CancelFn, FileCategory, FileGroup, FileItem, GroupType, ProgressFn, CANCEL_MSG,
    LARGE_FILE_THRESHOLD,
};
use crate::similarity;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// `group_reporting` ile ayni; uyarilari atar.
pub fn group(
    files: Vec<FileItem>, similarity_threshold: f64, cache: &HashCache,
    progress: ProgressFn<'_>, is_cancelled: CancelFn<'_>,
) -> Result<Vec<FileGroup>, String> {
    group_reporting(files, similarity_threshold, cache, progress, is_cancelled).map(|(groups, _)| groups)
}

/// Dosyalari hash'ler (huni yaklasimi) ve gruplara ayirir.
/// Onbellekte gecerli hash'i olan dosyalar yeniden hash'lenmez. Okunamayan
/// (kilitli, erisim izni olmayan) dosyalar hicbir gruba girmez ve uyari olarak
/// dondurulur: yanlis eslesme uretmez, taramayi da durdurmaz.
pub fn group_reporting(
    files: Vec<FileItem>,
    similarity_threshold: f64,
    cache: &HashCache,
    progress: ProgressFn<'_>,
    is_cancelled: CancelFn<'_>,
) -> Result<(Vec<FileGroup>, Vec<String>), String> {
    // Yalnizca cevrimici (bulut) dosyalar hash'lenmez: okumak onlari indirirdi.
    let (online, files): (Vec<FileItem>, Vec<FileItem>) = files.into_par_iter().partition(|f| crate::cloud::is_online_only(Path::new(&f.path)));
    let (files, unreadable): (Vec<FileItem>, Vec<FileItem>) =
        hash_all(files, cache, progress, is_cancelled)?.into_iter().partition(|f| !f.hash_id.is_empty());
    let mut warnings: Vec<String> = online.into_iter().map(|f| format!("Yalnızca çevrimiçi (bulut) dosya, indirilmemesi için atlandı: {}", f.path)).collect();
    warnings.extend(unreadable.into_iter().map(|f| format!("Okunamadı, atlandı: {}", f.path)));

    // (size, hash_id) kovalarina ayir (paralel, sahiplik ile klon yok).
    let buckets = bucket_exact(files);

    let mut exact_groups: Vec<FileGroup> = Vec::new();
    let mut similar_candidates: Vec<FileItem> = Vec::new();

    for items in buckets.into_values() {
        if items.len() >= 2 {
            let wasted_bytes = items[0].size * (items.len() as u64 - 1);
            let category = items[0].category;
            exact_groups.push(FileGroup {
                id: String::new(),
                group_type: GroupType::Exact,
                category,
                files: items,
                wasted_bytes,
            });
        } else {
            similar_candidates.push(items.into_iter().next().unwrap());
        }
    }

    let similar_groups = group_similar(similar_candidates, similarity_threshold, is_cancelled)?;

    let mut all = exact_groups;
    all.extend(similar_groups);

    // GRP-XXXX kimliklerini tek geciste ata.
    for (i, g) in all.iter_mut().enumerate() {
        g.id = format!("GRP-{:04}", i + 1);
    }

    Ok((all, warnings))
}

/// Huni yaklasimi ile her dosyaya hash_id atar; her dosyanin son hash'i
/// tamamlaninda ilerleme bildirilir (her dosya tam bir kez sayilir).
fn hash_all(
    files: Vec<FileItem>,
    cache: &HashCache,
    progress: ProgressFn<'_>,
    is_cancelled: CancelFn<'_>,
) -> Result<Vec<FileItem>, String> {
    let total = files.len() as u64;
    let counter = AtomicU64::new(0);
    let report = |done: u64| {
        if done.is_multiple_of(16) || done == total {
            progress(done, total);
        }
    };

    let (small, large): (Vec<FileItem>, Vec<FileItem>) = files
        .into_iter()
        .partition(|f| f.size < LARGE_FILE_THRESHOLD);

    // Buyuk dosyalar: dogrudan akilli ornekleme hash'i (paralel).
    let large_hashed: Vec<FileItem> = large
        .into_par_iter()
        .map(|mut f| {
            if is_cancelled() {
                return f;
            }
            if let Some(hit) = cache.get_full(&f.path, f.size, f.mtime) {
                f.hash_id = hit;
            } else {
                f.hash_id = hasher::hash_large(Path::new(&f.path), f.size).unwrap_or_default();
                if !f.hash_id.is_empty() {
                    cache.set_full(&f.path, f.size, f.mtime, f.hash_id.clone());
                }
            }
            report(counter.fetch_add(1, Ordering::Relaxed) + 1);
            f
        })
        .collect();
    if is_cancelled() {
        return Err(CANCEL_MSG.to_string());
    }

    // Kucuk dosyalar: once 4 KB on-hash (paralel).
    let small_preview: Vec<(FileItem, String)> = small
        .into_par_iter()
        .map(|f| {
            if is_cancelled() {
                return (f, String::new());
            }
            let preview = cache
                .get_preview(&f.path, f.size, f.mtime)
                .unwrap_or_else(|| hasher::hash_preview(Path::new(&f.path)).unwrap_or_default());
            if !preview.is_empty() {
                cache.set_preview(&f.path, f.size, f.mtime, preview.clone());
            }
            (f, preview)
        })
        .collect();
    if is_cancelled() {
        return Err(CANCEL_MSG.to_string());
    }

    // (size, preview) kovalari.
    let mut buckets: HashMap<(u64, String), Vec<FileItem>> = HashMap::new();
    for (f, preview) in small_preview {
        buckets.entry((f.size, preview)).or_default().push(f);
    }

    let mut small_hashed: Vec<FileItem> = Vec::new();
    for ((_size, preview), items) in buckets {
        if items.len() == 1 {
            // Eslesme yok: on-hash kimlik olarak yeterli (benzersiz).
            let mut f = items.into_iter().next().unwrap();
            f.hash_id = preview;
            report(counter.fetch_add(1, Ordering::Relaxed) + 1);
            small_hashed.push(f);
        } else {
            // Eslesme var: tam hash hesapla (paralel); onbellek vursa bile
            // tamakinlik icin her dosya icin tam hash'e bakilir.
            let hashed: Vec<FileItem> = items
                .into_par_iter()
                .map(|mut f| {
                    if is_cancelled() {
                        return f;
                    }
                    if let Some(hit) = cache.get_full(&f.path, f.size, f.mtime) {
                        f.hash_id = hit;
                    } else {
                        f.hash_id = hasher::hash_full(Path::new(&f.path)).unwrap_or_default();
                        if !f.hash_id.is_empty() {
                            cache.set_full(&f.path, f.size, f.mtime, f.hash_id.clone());
                        }
                    }
                    report(counter.fetch_add(1, Ordering::Relaxed) + 1);
                    f
                })
                .collect();
            small_hashed.extend(hashed);
        }
    }
    if is_cancelled() {
        return Err(CANCEL_MSG.to_string());
    }

    let mut out = small_hashed;
    out.extend(large_hashed);
    Ok(out)
}

/// (size, hash_id) kovalarini paralel olarak olusturur; dosyalar sahiplenilir
/// (her FileItem yalnizca bir kez tasinir, klon atlanir).
fn bucket_exact(files: Vec<FileItem>) -> HashMap<(u64, String), Vec<FileItem>> {
    type Bucket = HashMap<(u64, String), Vec<FileItem>>;

    files
        .into_par_iter()
        .fold(Bucket::new, |mut acc, f| {
            let key = (f.size, f.hash_id.clone());
            acc.entry(key).or_default().push(f);
            acc
        })
        .reduce(Bucket::new, |mut a, b| {
            for (k, mut v) in b {
                a.entry(k).or_default().append(&mut v);
            }
            a
        })
}

/// Benzer adaylarini kategoriye ayirip benzerlik kumeleri olusturur.
fn group_similar(
    candidates: Vec<FileItem>,
    threshold: f64,
    is_cancelled: CancelFn<'_>,
) -> Result<Vec<FileGroup>, String> {
    let mut by_category: HashMap<FileCategory, Vec<FileItem>> = HashMap::new();
    for f in candidates {
        by_category.entry(f.category).or_default().push(f);
    }

    let mut groups: Vec<FileGroup> = Vec::new();
    for items in by_category.into_values() {
        if is_cancelled() {
            return Err(CANCEL_MSG.to_string());
        }
        groups.extend(similarity::cluster_by_similarity(items, threshold, is_cancelled));
        if is_cancelled() {
            return Err(CANCEL_MSG.to_string());
        }
    }
    Ok(groups)
}

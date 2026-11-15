//! Plan uretimi, 1. adim: her dosyayi siniflandirir (tasi / yerinde / atla)
//! ve birebir kopyalari BLAKE3 ile bulup karantinaya ayirir.

use crate::hash_cache::HashCache;
use crate::hasher;
use crate::models::{normalize_exts, CancelFn, KeepPolicy, ProgressFn, CANCEL_MSG, LARGE_FILE_THRESHOLD};
use crate::organize_models::{OrganizeAction, OrganizeEntry, OrganizeOptions};
use crate::organize_rules::{is_temp_or_system, year_folder, RuleSet};
use crate::organize_walk::Candidate;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Plan uretilirken her dosyanin ara durumu.
pub struct Work {
    pub c: Candidate,
    pub action: OrganizeAction,
    pub category: Option<String>,
    pub kind: Option<String>,
    /// Koke gore hedef klasor (`Dokümanlar/PDF` veya duz modda `PDF`).
    pub rel: Option<PathBuf>,
    pub to: Option<PathBuf>,
    pub renamed: bool,
    pub similar_to: Option<String>,
    pub note: Option<String>,
}

/// Iki klasor yolu ayni mi (Windows gibi buyuk/kucuk harf duyarsiz).
pub fn same_dir(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| -> Vec<String> {
        p.components().map(|c| c.as_os_str().to_string_lossy().to_lowercase()).collect()
    };
    norm(a) == norm(b)
}

pub fn build(
    root: &Path,
    files: Vec<Candidate>,
    opts: &OrganizeOptions,
    cache: &HashCache,
    progress: ProgressFn<'_>,
    is_cancelled: CancelFn<'_>,
) -> Result<Vec<OrganizeEntry>, String> {
    let rules = RuleSet::new(&opts.rules)?;
    let banned = normalize_exts(&opts.banned_extensions);
    let mut work: Vec<Work> = files.into_iter().map(|c| classify(root, c, opts, &rules, &banned)).collect();
    if opts.quarantine_duplicates {
        mark_duplicates(&mut work, opts.keep_policy, cache, progress, is_cancelled)?;
    }
    Ok(crate::organize_names::finalize(root, work, opts.similarity_threshold, is_cancelled))
}

fn classify(root: &Path, c: Candidate, opts: &OrganizeOptions, rules: &RuleSet, banned: &[String]) -> Work {
    let mut w = Work {
        c, action: OrganizeAction::Skip, category: None, kind: None, rel: None, to: None,
        renamed: false, similar_to: None, note: None,
    };
    if banned.contains(&w.c.ext) {
        w.note = Some("Yasaklı uzantı".into());
        return w;
    }
    if is_temp_or_system(&w.c.name) {
        w.note = Some("Geçici veya sistem dosyası".into());
        return w;
    }
    let Some((category, kind)) = rules.target_of(&w.c.ext, opts.include_other) else {
        w.note = Some("Kurallarda olmayan tür (Diğer dosyalar kapalı)".into());
        return w;
    };
    let mut rel = if opts.flat { PathBuf::from(&kind) } else { PathBuf::from(&category).join(&kind) };
    if opts.by_year {
        rel.push(year_folder(w.c.mtime));
    }
    let dest = root.join(&rel);
    let in_place = w.c.path.parent().is_some_and(|p| same_dir(p, &dest));
    w.action = if in_place { OrganizeAction::InPlace } else { OrganizeAction::Move };
    w.category = Some(category);
    w.kind = Some(kind);
    w.rel = Some(rel);
    w
}

/// Ayni boyuttaki dosyalarin tam BLAKE3 kimligini (500 MB+ icin ornekleme) hesaplar;
/// birebir ayni olanlardan biri kalir, digerleri `Quarantine` olarak isaretlenir.
/// 0 baytlik dosyalar kopya sayilmaz (bos dosyalarin hepsi "ayni" olurdu).
fn mark_duplicates(
    work: &mut [Work],
    policy: KeepPolicy,
    cache: &HashCache,
    progress: ProgressFn<'_>,
    is_cancelled: CancelFn<'_>,
) -> Result<(), String> {
    let mut by_size: HashMap<u64, Vec<usize>> = HashMap::new();
    for (i, w) in work.iter().enumerate() {
        if matches!(w.action, OrganizeAction::Move | OrganizeAction::InPlace) && w.c.size > 0 {
            by_size.entry(w.c.size).or_default().push(i);
        }
    }
    let candidates: Vec<usize> = by_size.into_values().filter(|v| v.len() > 1).flatten().collect();
    let total = candidates.len() as u64;
    let done = AtomicU64::new(0);
    let shared: &[Work] = work;
    let hashed: Vec<(usize, Option<String>)> = candidates
        .par_iter()
        .map(|&i| {
            if is_cancelled() {
                return (i, None);
            }
            let c = &shared[i].c;
            let key = c.path.to_string_lossy();
            let hash = cache.get_full(&key, c.size, c.mtime).or_else(|| {
                let h = if c.size >= LARGE_FILE_THRESHOLD {
                    hasher::hash_large(&c.path, c.size)
                } else {
                    hasher::hash_full(&c.path)
                }
                .ok()?;
                cache.set_full(&key, c.size, c.mtime, h.clone());
                Some(h)
            });
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            if n.is_multiple_of(16) || n == total {
                progress(n, total);
            }
            (i, hash)
        })
        .collect();
    if is_cancelled() {
        return Err(CANCEL_MSG.to_string());
    }

    let mut buckets: HashMap<(u64, String), Vec<usize>> = HashMap::new();
    for (i, hash) in hashed {
        // Okunamayan (kilitli) dosyalar kopya sayilmaz; normal tasinir.
        if let Some(h) = hash {
            buckets.entry((work[i].c.size, h)).or_default().push(i);
        }
    }
    for mut members in buckets.into_values().filter(|m| m.len() > 1) {
        members.sort_unstable();
        let keep = keeper(work, &members, policy);
        let keep_name = work[keep].c.path.to_string_lossy().into_owned();
        for &i in members.iter().filter(|&&i| i != keep) {
            work[i].action = OrganizeAction::Quarantine;
            work[i].note = Some(format!("Birebir kopya · korunan: {keep_name}"));
        }
    }
    Ok(())
}

/// Zaten yerinde olan kopya varsa o korunur (tekrar calistirmada dosya oynamaz);
/// yoksa secili "tutulacak dosya" politikasi uygulanir.
fn keeper(work: &[Work], members: &[usize], policy: KeepPolicy) -> usize {
    let in_place: Vec<usize> =
        members.iter().copied().filter(|&i| work[i].action == OrganizeAction::InPlace).collect();
    if let Some(&i) = in_place.iter().min_by_key(|&&i| (work[i].c.mtime, i)) {
        return i;
    }
    let it = members.iter().copied();
    match policy {
        KeepPolicy::First => members[0],
        KeepPolicy::Newest => it.max_by_key(|&i| (work[i].c.mtime, std::cmp::Reverse(i))).unwrap_or(members[0]),
        KeepPolicy::Oldest => it.min_by_key(|&i| (work[i].c.mtime, i)).unwrap_or(members[0]),
        KeepPolicy::ShortestPath => it
            .min_by_key(|&i| (work[i].c.path.to_string_lossy().chars().count(), i))
            .unwrap_or(members[0]),
    }
}

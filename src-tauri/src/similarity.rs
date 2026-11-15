//! Jaro-Winkler isim benzerligi kumeleme. Referans naif surum `tests/support/naive.rs` icindedir.
//!
//! Optimizasyonlar:
//! 1) Ilk-2-harf bloklama (capraz kova karsilastirmalari atlanir),
//! 2) Birebir ayni stem'ler JW hesaplamadan birlestirilir,
//! 3) Uzunluk eleme: matematiksel ust sinirin altindaki ciftler atlanir
//!    (yanlis negatif uretmez).

use crate::models::{CancelFn, FileGroup, FileItem, GroupType};
use rayon::prelude::*;
use std::collections::HashMap;

/// Iki (karakter) uzunlugu icin JW ust sinirini hesaplar; esik altindaysa false.
/// Bu bir UST SINIR kontroludur: gercek JW >= esik olan hicbir cift elenmez.
#[inline]
pub fn could_match(a: usize, b: usize, threshold: f64) -> bool {
    if a == 0 || b == 0 {
        // Bir taraf bos: JW = 0 (iki bos stem tekillestirmede birlestirilir).
        return false;
    }
    let min_len = a.min(b) as f64;
    // Jaro ust siniri: eslesme m <= min_len, transpozisyon t >= 0.
    let jaro_max = (min_len / a as f64 + min_len / b as f64 + 1.0) / 3.0;
    // JW ust siniri: prefix bonusu en fazla l=4 * p=0.1.
    let jw_max = jaro_max + 0.4 * (1.0 - jaro_max);
    jw_max >= threshold
}

/// Stem'i rakam disi iskelete (`img_#`) ve rakam dizilerine ayirir.
fn split_digits(s: &str) -> (String, Vec<String>) {
    let (mut skeleton, mut runs, mut cur) = (String::new(), Vec::new(), String::new());
    for c in s.chars() {
        if c.is_ascii_digit() {
            cur.push(c);
            continue;
        }
        if !cur.is_empty() {
            runs.push(std::mem::take(&mut cur));
            skeleton.push('#');
        }
        skeleton.push(c);
    }
    if !cur.is_empty() {
        runs.push(cur);
        skeleton.push('#');
    }
    (skeleton, runs)
}

/// Iki ad yalnizca bir sayac/zaman damgasiyla mi ayriliyor (`IMG_0001`/`IMG_0002`,
/// `scan001`/`scan002`, `Screenshot_20240101-101500`)? Boyle adlar surum degil, ayri
/// dosyalardir; benzer sayilsalardi binlerce fotograf tek gruba zincirlenirdi.
/// Kisa numaralar (`v1`/`v2`, `(1)`/`(2)`) ve yillar (`fatura_2024`/`fatura_2025`) benzer kalir.
pub fn is_sequence_pair(a: &str, b: &str) -> bool {
    let ((sa, ra), (sb, rb)) = (split_digits(a), split_digits(b));
    let year = |r: &str| r.len() == 4 && matches!(r.parse::<u32>(), Ok(1900..=2099));
    // Iskeletlerden biri digerinin on eki olabilir: `IMG_0007 (1)` da `IMG_0008`'in surumu degildir.
    (sa.starts_with(&sb) || sb.starts_with(&sa)) && ra.iter().zip(&rb).any(|(x, y)| x != y && x.len() >= 3 && y.len() >= 3 && !(year(x) && year(y)))
}

/// Esik ustu benzerlik ve sayac farki olmayan cift mi?
#[inline]
fn similar(a: &str, b: &str, threshold: f64) -> bool {
    strsim::jaro_winkler(a, b) >= threshold && !is_sequence_pair(a, b)
}

/// Bloklama anahtari: stem'in ilk 2 karakteri.
fn prefix_key(stem: &str) -> String {
    stem.chars().take(2).collect()
}

/// Optimize kumeleme: bloklama + tekillestirme + uzunluk eleme + kova ici paralellik.
pub fn cluster_by_similarity(
    items: Vec<FileItem>,
    threshold: f64,
    is_cancelled: CancelFn<'_>,
) -> Vec<FileGroup> {
    let n = items.len();
    if n < 2 {
        return Vec::new();
    }

    let stems: Vec<String> = items.iter().map(|f| f.stem.to_lowercase()).collect();
    let lens: Vec<usize> = stems.iter().map(|s| s.chars().count()).collect();

    // 1) Ilk-2-harf kovalari.
    let mut buckets: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, s) in stems.iter().enumerate() {
        buckets.entry(prefix_key(s)).or_default().push(i);
    }

    let mut uf = UnionFind::new(n);
    let stems_slice: &[String] = &stems;
    let lens_slice: &[usize] = &lens;

    for idxs in buckets.values() {
        if is_cancelled() {
            break;
        }
        if idxs.len() < 2 {
            continue;
        }

        // 2) Birebir ayni stem'leri JW hesaplamadan birlestir.
        let mut by_stem: HashMap<&str, Vec<usize>> = HashMap::new();
        for &i in idxs {
            by_stem.entry(stems[i].as_str()).or_default().push(i);
        }
        let mut uniq: Vec<usize> = Vec::with_capacity(by_stem.len());
        for members in by_stem.values() {
            let rep = members[0];
            uniq.push(rep);
            for &m in &members[1..] {
                uf.union(rep, m);
            }
        }

        let u = uniq.len();
        if u < 2 {
            continue;
        }

        // 3) Benzersiz stem'ler arasinda: uzunluk eleme + JW (paralel).
        let uniq_slice: &[usize] = &uniq;
        let edges: Vec<(usize, usize)> = (0..u)
            .into_par_iter()
            .flat_map_iter(|a| {
                (a + 1..u).filter_map(move |b| {
                    let ia = uniq_slice[a];
                    let ib = uniq_slice[b];
                    if !could_match(lens_slice[ia], lens_slice[ib], threshold) {
                        return None;
                    }
                    if similar(&stems_slice[ia], &stems_slice[ib], threshold) {
                        Some((ia, ib))
                    } else {
                        None
                    }
                })
            })
            .collect();

        for (ia, ib) in edges {
            uf.union(ia, ib);
        }
    }

    build_groups(items, uf)
}

/// Union-find koklerinden FileGroup'lari uretir.
fn build_groups(items: Vec<FileItem>, mut uf: UnionFind) -> Vec<FileGroup> {
    let n = items.len();
    let mut clusters: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        clusters.entry(uf.find(i)).or_default().push(i);
    }

    let mut groups = Vec::new();
    for idxs in clusters.into_values() {
        if idxs.len() < 2 {
            continue;
        }
        let files: Vec<FileItem> = idxs.into_iter().map(|i| items[i].clone()).collect();
        groups.push(FileGroup {
            id: String::new(),
            group_type: GroupType::Similar,
            category: files[0].category,
            files,
            wasted_bytes: 0,
        });
    }
    groups
}

/// Yol kisaltmali union-find.
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return;
        }
        if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
        } else if self.rank[ra] > self.rank[rb] {
            self.parent[rb] = ra;
        } else {
            self.parent[rb] = ra;
            self.rank[ra] += 1;
        }
    }
}

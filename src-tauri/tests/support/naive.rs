//! Referans naif kumeleme (tum ciftler). Yalnizca dogruluk testi ve benchmark icindir;
//! uretim kodu `similarity::cluster_by_similarity` kullanir.

use smart_file_organizer_lib::models::{FileGroup, FileItem, GroupType};
use smart_file_organizer_lib::similarity::is_sequence_pair;

pub fn cluster_naive(items: Vec<FileItem>, threshold: f64) -> Vec<FileGroup> {
    let stems: Vec<String> = items.iter().map(|f| f.stem.to_lowercase()).collect();
    let n = items.len();
    // label[i]: i'nin ait oldugu kumenin en kucuk uyesi.
    let mut label: Vec<usize> = (0..n).collect();
    for i in 0..n {
        for j in i + 1..n {
            let (a, b) = (label[i], label[j]);
            if a == b || strsim::jaro_winkler(&stems[i], &stems[j]) < threshold || is_sequence_pair(&stems[i], &stems[j]) {
                continue;
            }
            let (keep, drop) = (a.min(b), a.max(b));
            label.iter_mut().filter(|l| **l == drop).for_each(|l| *l = keep);
        }
    }
    let mut groups = Vec::new();
    for root in 0..n {
        let files: Vec<FileItem> = (0..n).filter(|&i| label[i] == root).map(|i| items[i].clone()).collect();
        if files.len() > 1 {
            groups.push(FileGroup { id: String::new(), group_type: GroupType::Similar, category: files[0].category, files, wasted_bytes: 0 });
        }
    }
    groups
}

//! Plan uretimi, 2. adim: hedef adlarini belirler. Ayni klasore ayni adla gelen
//! dosyalardan en eski tarihli adini korur, digerleri `ad (1).uzanti` olur.
//! Hedefte zaten bulunan adlar asla ezilmez. Benzer adlar bilgi olarak isaretlenir.

use crate::models::{CancelFn, FileCategory, FileItem, QUARANTINE_DIR};
use crate::organize_models::{OrganizeAction, OrganizeEntry};
use crate::organize_plan::Work;
use crate::similarity::cluster_by_similarity;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Windows MAX_PATH (260) icinde kalmak icin hedef yolun azami karakter sayisi.
const MAX_PATH_CHARS: usize = 259;

pub fn finalize(root: &Path, mut work: Vec<Work>, threshold: f64, is_cancelled: CancelFn<'_>) -> Vec<OrganizeEntry> {
    assign_names(root, &mut work);
    mark_similar(&mut work, threshold, is_cancelled);
    work.into_iter().map(into_entry).collect()
}

fn assign_names(root: &Path, work: &mut [Work]) {
    // hedef klasor (kucuk harf anahtar) -> (klasor, uye indeksleri)
    let mut by_dir: HashMap<String, (PathBuf, Vec<usize>)> = HashMap::new();
    for (i, w) in work.iter().enumerate() {
        let Some(rel) = &w.rel else { continue };
        let dir = match w.action {
            OrganizeAction::Move => root.join(rel),
            OrganizeAction::Quarantine => root.join(QUARANTINE_DIR).join(rel),
            _ => continue,
        };
        let key = dir.to_string_lossy().to_lowercase();
        by_dir.entry(key).or_insert_with(|| (dir, Vec::new())).1.push(i);
    }
    for (dir, mut members) in by_dir.into_values() {
        // En eski dosya adini korur; esitlikte yol sirasi (deterministik).
        members.sort_by(|&a, &b| (work[a].c.mtime, &work[a].c.path).cmp(&(work[b].c.mtime, &work[b].c.path)));
        let mut reserved = existing_names(&dir);
        for i in members {
            let (name, renamed) = reserve(&mut reserved, &work[i].c.name);
            let to = dir.join(&name);
            if to.to_string_lossy().chars().count() > MAX_PATH_CHARS {
                work[i].action = OrganizeAction::Skip;
                work[i].note = Some("Hedef yol 260 karakteri aşıyor".into());
                continue;
            }
            work[i].to = Some(to);
            work[i].renamed = renamed;
        }
    }
}

/// Klasorde mevcut adlar (kucuk harf). Klasor yoksa bos.
fn existing_names(dir: &Path) -> HashSet<String> {
    std::fs::read_dir(dir)
        .map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_lowercase()).collect())
        .unwrap_or_default()
}

/// Ad bossa ayirir; doluysa `ad (n).uzanti` ile ilk bos numarayi bulur.
pub fn reserve(reserved: &mut HashSet<String>, name: &str) -> (String, bool) {
    if reserved.insert(name.to_lowercase()) {
        return (name.to_string(), false);
    }
    let p = Path::new(name);
    let stem = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| name.to_string());
    let ext = p.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let mut n = 1usize;
    loop {
        let candidate = format!("{stem} ({n}){ext}");
        if reserved.insert(candidate.to_lowercase()) {
            return (candidate, true);
        }
        n += 1;
    }
}

/// Ayni tur klasorune dusen, adi farkli ama benzer (v1/v2, "son" ...) dosyalari isaretler.
fn mark_similar(work: &mut [Work], threshold: f64, is_cancelled: CancelFn<'_>) {
    let mut by_rel: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, w) in work.iter().enumerate() {
        if matches!(w.action, OrganizeAction::Move | OrganizeAction::InPlace)
            && let Some(rel) = &w.rel
        {
            by_rel.entry(rel.to_string_lossy().to_lowercase()).or_default().push(i);
        }
    }
    for members in by_rel.into_values().filter(|m| m.len() > 1) {
        let items: Vec<FileItem> = members.iter().map(|&i| as_file_item(&work[i])).collect();
        let index: HashMap<String, usize> = members.iter().map(|&i| (work[i].c.path.to_string_lossy().into_owned(), i)).collect();
        for group in cluster_by_similarity(items, threshold, is_cancelled) {
            for file in &group.files {
                let other = group.files.iter().find(|o| o.stem.to_lowercase() != file.stem.to_lowercase());
                if let (Some(&i), Some(o)) = (index.get(&file.path), other) {
                    work[i].similar_to = Some(o.name.clone());
                }
            }
        }
    }
}

fn as_file_item(w: &Work) -> FileItem {
    let stem = Path::new(&w.c.name).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    FileItem {
        path: w.c.path.to_string_lossy().into_owned(),
        name: w.c.name.clone(),
        stem,
        extension: w.c.ext.clone(),
        size: w.c.size,
        mtime: w.c.mtime,
        category: FileCategory::Other,
        hash_id: String::new(),
    }
}

fn into_entry(w: Work) -> OrganizeEntry {
    OrganizeEntry {
        from: w.c.path.to_string_lossy().into_owned(),
        to: w.to.map(|p| p.to_string_lossy().into_owned()),
        action: w.action,
        category: w.category,
        kind: w.kind,
        size: w.c.size,
        mtime: w.c.mtime,
        renamed: w.renamed,
        similar_to: w.similar_to,
        note: w.note,
    }
}

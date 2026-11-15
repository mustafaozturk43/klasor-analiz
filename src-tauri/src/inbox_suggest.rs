//! Gelen kutusu onerileri: her dosya icin Tut / Tasi / At onerisi, hedef klasor ve kisa gerekce.
//! Kural sirasi: birebir kopya -> At; adi bir hedef klasordeki dosyaya benziyorsa -> Tasi;
//! adi gelen kutusundaki baska bir dosyaya benziyor ve o tasiniyorsa -> birlikte Tasi; yoksa Tut.

use crate::inspector::FileDetail;
use crate::models::DEFAULT_SIMILARITY_THRESHOLD;
use crate::operations::target_folders;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Her hedef klasorden en fazla bu kadar dosya adi okunur.
const MAX_STEMS: usize = 400;
const MAX_TARGETS: usize = 60;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    /// "keep" | "move" | "bin"
    pub verdict: String,
    /// Tasinacaksa hedef klasorun adi (yedek klasorunun dogrudan alt klasoru).
    pub target: Option<String>,
    pub reason: String,
}

struct Folder {
    name: String,
    /// (uzantisiz kucuk harfli ad, gercek dosya adi)
    stems: Vec<(String, String)>,
}

fn stem_of(name: &str) -> String {
    Path::new(name).file_stem().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default()
}

fn read_folders(backup: &Path) -> Vec<Folder> {
    target_folders(backup)
        .into_iter()
        .take(MAX_TARGETS)
        .map(|t| {
            let mut seen = HashSet::new();
            let mut stems = Vec::new();
            for e in fs::read_dir(&t.path).into_iter().flatten().flatten() {
                if stems.len() >= MAX_STEMS {
                    break;
                }
                if !e.file_type().map(|k| k.is_file()).unwrap_or(false) {
                    continue;
                }
                let name = e.file_name().to_string_lossy().into_owned();
                let stem = stem_of(&name);
                if seen.insert(stem.clone()) {
                    stems.push((stem, name));
                }
            }
            Folder { name: t.name, stems }
        })
        .collect()
}

/// Adi en cok benzeyen hedef klasor dosyasi: (klasor sirasi, dosya adi, puan). Ilk harfi ayni olanlara bakilir.
fn best_match(stem: &str, folders: &[Folder]) -> Option<(usize, String, f64)> {
    let first = stem.chars().next()?;
    let mut best: Option<(usize, String, f64)> = None;
    for (i, f) in folders.iter().enumerate() {
        for (s, name) in &f.stems {
            if !s.starts_with(first) {
                continue;
            }
            let score = strsim::jaro_winkler(stem, s);
            if score >= DEFAULT_SIMILARITY_THRESHOLD && best.as_ref().is_none_or(|b| score > b.2) {
                best = Some((i, name.clone(), score));
            }
        }
    }
    best
}

fn first_pass(f: &FileDetail, folders: &[Folder]) -> Suggestion {
    if let Some(orig) = &f.duplicate_of {
        let name = Path::new(orig).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| orig.clone());
        let reason = match &f.duplicate_in {
            Some(folder) => format!("İçeriği {folder} klasöründeki {name} ile birebir aynı."),
            None => format!("{name} dosyasının birebir kopyası, içeriği özdeş."),
        };
        return Suggestion { verdict: "bin".into(), target: None, reason };
    }
    if let Some((i, file, score)) = best_match(&stem_of(&f.name), folders) {
        let reason = format!("{} klasöründeki {file} ile adı %{} benzer.", folders[i].name, (score * 100.0).round() as u8);
        return Suggestion { verdict: "move".into(), target: Some(folders[i].name.clone()), reason };
    }
    Suggestion { verdict: "keep".into(), target: None, reason: "Yeni bir dosya, benzeri yok. Yerinde kalması yeterli.".into() }
}

/// Her dosyaya bir oneri yazar (`suggestion`). Oneri yalnizca baslangic kararidir; kullanici degistirebilir.
pub fn suggest(files: &mut [FileDetail], backup: &Path) {
    let folders = read_folders(backup);
    for f in files.iter_mut() {
        f.suggestion = Some(first_pass(f, &folders));
    }
    // Hedefi bulunamayan ama adi tasinan bir dosyaya benzeyen dosya, onunla birlikte gider.
    let moving: HashMap<String, String> = files
        .iter()
        .filter_map(|f| f.suggestion.as_ref().filter(|s| s.verdict == "move").and_then(|s| Some((f.name.clone(), s.target.clone()?))))
        .collect();
    for f in files.iter_mut() {
        let keeps = f.suggestion.as_ref().is_some_and(|s| s.verdict == "keep");
        let with = f.similar_to.as_ref().filter(|_| keeps).and_then(|sim| moving.get(sim).map(|t| (sim.clone(), t.clone())));
        if let Some((sim, target)) = with {
            f.suggestion = Some(Suggestion { verdict: "move".into(), target: Some(target), reason: format!("Adı {sim} ile benzer, ikisi birlikte gitmeli.") });
        }
    }
}

#[cfg(test)]
#[path = "inbox_suggest_tests.rs"]
mod tests;

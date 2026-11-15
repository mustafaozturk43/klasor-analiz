//! Hedef klasor secicisi icin kokun altindaki klasor agaci (karantina, gizli ve
//! haric tutulan klasorler alt agaclariyla birlikte atlanir).

use crate::models::QUARANTINE_DIR;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubfolderList {
    pub paths: Vec<String>,
    pub truncated: bool,
}

fn skipped(name: &str, excludes: &[String]) -> bool {
    name.starts_with('.') || name.eq_ignore_ascii_case(QUARANTINE_DIR) || excludes.iter().any(|e| e.trim().eq_ignore_ascii_case(name))
}

/// Kokun altindaki klasorler (kok haric), en cok `max_depth` derinlik ve `limit` adet.
pub fn collect_subfolders(root: &Path, excludes: &[String], max_depth: usize, limit: usize) -> SubfolderList {
    let (mut found, mut truncated) = (Vec::new(), false);
    let mut level: Vec<PathBuf> = vec![root.to_path_buf()];
    for _ in 0..max_depth {
        let mut next = Vec::new();
        for dir in &level {
            let Ok(rd) = std::fs::read_dir(dir) else { continue };
            for entry in rd.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if !entry.file_type().is_ok_and(|t| t.is_dir()) || skipped(&name, excludes) {
                    continue;
                }
                if found.len() >= limit {
                    truncated = true;
                    break;
                }
                found.push(entry.path());
                next.push(entry.path());
            }
        }
        if truncated || next.is_empty() {
            break;
        }
        level = next;
    }
    let mut paths: Vec<String> = found.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    paths.sort_by_key(|p| p.to_lowercase());
    SubfolderList { paths, truncated }
}

#[tauri::command(async)]
pub fn list_subfolders(root: String, exclude_folders: Vec<String>) -> Result<SubfolderList, String> {
    let root = PathBuf::from(root);
    if !root.is_dir() {
        return Err(format!("Klasör bulunamadı: {}", root.display()));
    }
    Ok(collect_subfolders(&root, &exclude_folders, 6, 2000))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tree(tag: &str) -> std::path::PathBuf {
        let r = std::env::temp_dir().join(format!("sfo_subf_{}_{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&r);
        for d in ["A/B", "_Kopyalar_Karantina/x", "node_modules/y", ".git"] {
            fs::create_dir_all(r.join(d)).unwrap();
        }
        fs::write(r.join("A/dosya.txt"), b"x").unwrap();
        r
    }

    #[test]
    fn lists_nested_folders_skipping_quarantine_and_excludes() {
        let r = tree("a");
        let out = collect_subfolders(&r, &["node_modules".into()], 6, 2000);
        let want: Vec<String> = vec![r.join("A"), r.join("A").join("B")].iter().map(|p| p.to_string_lossy().into_owned()).collect();
        assert_eq!(out.paths, want);
        assert!(!out.truncated);
    }

    #[test]
    fn marks_truncated_at_limit() {
        let r = tree("b");
        let out = collect_subfolders(&r, &["node_modules".into()], 6, 1);
        assert_eq!(out.paths.len(), 1);
        assert!(out.truncated);
    }
}

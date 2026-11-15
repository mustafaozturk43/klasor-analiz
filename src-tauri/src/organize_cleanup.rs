//! Bos klasorlerin kaldirilmasi.
//!
//! Verilen klasorler ve onlarin (sinir klasorune kadar) ust klasorleri denenir;
//! `fs::remove_dir` dolu klasoru silmez, bu yuzden icinde tek bir dosya (gizli
//! olanlar dahil) kalan klasor yerinde kalir. Sinir klasorunun kendisi asla
//! kaldirilmaz. Geri alma, gereken klasorleri dosyalari geri tasirken yeniden olusturur.

use crate::models::QUARANTINE_DIR;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// `dirs` ve bosalan ust klasorlerini (boundary haric) en derinden baslayarak kaldirir.
/// `keep` true donen klasorlere dokunulmaz.
pub fn prune_empty_dirs(boundary: &Path, dirs: impl IntoIterator<Item = PathBuf>, keep: &dyn Fn(&Path) -> bool) -> u64 {
    let mut all: BTreeSet<PathBuf> = BTreeSet::new();
    for start in dirs {
        let mut cur = Some(start.as_path());
        while let Some(dir) = cur {
            if dir == boundary || !dir.starts_with(boundary) {
                break;
            }
            all.insert(dir.to_path_buf());
            cur = dir.parent();
        }
    }
    let mut ordered: Vec<PathBuf> = all.into_iter().collect();
    // En derin klasorler once: alt klasor kalkinca ust klasor bosalabilir.
    ordered.sort_by_key(|d| std::cmp::Reverse(d.components().count()));
    ordered.into_iter().filter(|d| !keep(d) && fs::remove_dir(d).is_ok()).count() as u64
}

/// Karantina (ana klasorun hemen altindaki `_Kopyalar_Karantina`) icinde mi?
pub fn is_quarantine_path(root: &Path, dir: &Path) -> bool {
    dir.strip_prefix(root)
        .ok()
        .and_then(|rel| rel.components().next())
        .is_some_and(|c| c.as_os_str().to_string_lossy().eq_ignore_ascii_case(QUARANTINE_DIR))
}

/// Tasinan dosyalarin eski klasorlerinden bosalanlari kaldirir (ana klasor ve karantina haric).
pub fn remove_emptied_dirs<'a>(root: &Path, moved_from: impl IntoIterator<Item = &'a Path>) -> u64 {
    let parents = moved_from.into_iter().filter_map(|p| p.parent().map(Path::to_path_buf));
    prune_empty_dirs(root, parents, &|d| is_quarantine_path(root, d))
}

/// Iki yolun ortak ust klasoru.
fn common_ancestor(a: &Path, b: &Path) -> PathBuf {
    a.components().zip(b.components()).take_while(|(x, y)| x == y).map(|(x, _)| x).collect()
}

/// Geri almadan sonra dosyalarin tasindigi hedef klasorlerden bosalanlari kaldirir.
/// Her tasima icin yalnizca eski ve yeni konumun ortak ust klasorunun ALTINDAKI
/// klasorler denenir (ornegin `Downloads\Dokümanlar\PDF`, `Downloads` degil);
/// ayni klasorde yeniden adlandirmada hicbir klasore dokunulmaz.
pub fn prune_after_undo(moves: &[crate::models::MoveRecord]) -> u64 {
    moves
        .iter()
        .filter_map(|m| {
            let (from, to) = (Path::new(&m.from).parent()?, Path::new(&m.to).parent()?);
            let boundary = common_ancestor(from, to);
            (!boundary.as_os_str().is_empty()).then(|| prune_empty_dirs(&boundary, [to.to_path_buf()], &|_| false))
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prunes_emptied_targets_after_undo_but_not_the_common_root() {
        let root = std::env::temp_dir().join(format!("sfo_undo_clean_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("Dokümanlar/PDF")).unwrap();
        fs::create_dir_all(root.join("Resimler/JPG")).unwrap();
        fs::write(root.join("Resimler/JPG/kalan.jpg"), b"x").unwrap();
        let mv = |to: &str| crate::models::MoveRecord { from: root.join("a.pdf").to_string_lossy().into(), to: root.join(to).to_string_lossy().into() };
        let moves = [mv("Dokümanlar/PDF/a.pdf"), mv("Resimler/JPG/a.jpg"), mv("yeni_ad.pdf")];
        assert_eq!(prune_after_undo(&moves), 2, "Dokümanlar/PDF ve Dokümanlar");
        assert!(!root.join("Dokümanlar").exists());
        assert!(root.join("Resimler/JPG/kalan.jpg").exists(), "dolu klasore dokunulmaz");
        assert!(root.exists(), "ortak ust klasor kalir");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn removes_only_emptied_dirs_below_root() {
        let root = std::env::temp_dir().join(format!("sfo_clean_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("a/b/c")).unwrap();
        fs::create_dir_all(root.join("keep")).unwrap();
        fs::create_dir_all(root.join(QUARANTINE_DIR).join("x")).unwrap();
        fs::write(root.join("keep/x.txt"), b"x").unwrap();
        let moved = [root.join("a/b/c/f.pdf"), root.join("keep/g.pdf"), root.join("h.pdf"), root.join(QUARANTINE_DIR).join("x/q.pdf")];
        let removed = remove_emptied_dirs(&root, moved.iter().map(PathBuf::as_path));
        assert_eq!(removed, 3, "a, a/b, a/b/c");
        assert!(!root.join("a").exists());
        assert!(root.join("keep").exists() && root.exists());
        assert!(root.join(QUARANTINE_DIR).join("x").exists(), "karantinaya dokunulmaz");
        fs::remove_dir_all(&root).unwrap();
    }
}

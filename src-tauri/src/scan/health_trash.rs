//! Klasor sagligi: secilen buyuk/eski/bos dosyalari Windows Geri Donusum Kutusu'na gonderir.
//! Kalici silme yoktur; dosyalar Geri Donusum Kutusu'ndan geri yuklenir (Islem Gecmisi'nden
//! geri alinamaz). Once HEPSI dogrulanir: kok icinde, karantinada degil, duz dosya ve rapordan
//! sonra degismemis. Tek bir sorun varsa hicbir dosyaya dokunulmaz.

use crate::fileops::quarantine_manage::BatchOutcome;
use crate::organize_models::FailedMove;
use crate::path_guard::{ensure_inside, in_quarantine};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// Raporda gorunen dosya: yol + o andaki boyut ve degisiklik tarihi (unix sn).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashItem {
    pub path: String,
    pub size: u64,
    pub mtime: u64,
}

const CHANGED: &str = "Dosya rapordan sonra değişti; raporu yenileyin";

fn verify(root: &Path, item: &TrashItem) -> Result<PathBuf, String> {
    let p = PathBuf::from(&item.path);
    ensure_inside(root, &p)?;
    if in_quarantine(root, &p) {
        return Err(format!("Karantinadaki dosyalar bu ekrandan gönderilemez: {}", p.display()));
    }
    let meta = std::fs::symlink_metadata(&p).map_err(|_| format!("{CHANGED}: {}", p.display()))?;
    if !meta.is_file() {
        return Err(format!("Yalnızca dosyalar gönderilebilir: {}", p.display()));
    }
    let mtime = meta.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs());
    if meta.len() != item.size || mtime != item.mtime {
        return Err(format!("{CHANGED}: {}", p.display()));
    }
    Ok(p)
}

/// `send`: uretimde `trash::delete`, testlerde sahte.
pub fn trash_core(root: &Path, items: &[TrashItem], send: &dyn Fn(&Path) -> Result<(), String>) -> Result<BatchOutcome, String> {
    if items.is_empty() {
        return Err("Önce dosya seçin.".into());
    }
    let files: Vec<PathBuf> = items.iter().map(|i| verify(root, i)).collect::<Result<_, _>>()?;
    let mut out = BatchOutcome { done: 0, failed: Vec::new() };
    for p in files {
        match send(&p) {
            Ok(()) => out.done += 1,
            Err(error) => out.failed.push(FailedMove { path: p.to_string_lossy().into_owned(), error }),
        }
    }
    Ok(out)
}

#[tauri::command(async)]
pub(crate) fn trash_health_files(root_path: String, items: Vec<TrashItem>) -> Result<BatchOutcome, String> {
    trash_core(Path::new(&root_path), &items, &|p| trash::delete(p).map_err(|e| e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::QUARANTINE_DIR;
    use std::fs;
    use std::sync::Mutex;

    fn scratch(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("sfo_htrash_{tag}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }
    fn item(p: &Path) -> TrashItem {
        let m = fs::metadata(p).unwrap();
        TrashItem { path: p.to_string_lossy().into_owned(), size: m.len(), mtime: m.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs() }
    }

    #[test]
    fn sends_verified_files_and_reports_individual_failures() {
        let root = scratch("ok");
        let (a, b) = (root.join("a.bin"), root.join("b.bin"));
        fs::write(&a, b"aa").unwrap();
        fs::write(&b, b"bb").unwrap();
        let sent = Mutex::new(Vec::new());
        let send = |p: &Path| {
            if p.ends_with("b.bin") {
                return Err("kilitli".to_string());
            }
            sent.lock().unwrap().push(p.to_path_buf());
            Ok(())
        };
        let out = trash_core(&root, &[item(&a), item(&b)], &send).unwrap();
        assert_eq!((out.done, out.failed.len(), out.failed[0].error.as_str()), (1, 1, "kilitli"));
        assert_eq!(sent.lock().unwrap().len(), 1);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn one_bad_item_blocks_everything() {
        let root = scratch("bad");
        let good = root.join("iyi.bin");
        fs::write(&good, b"x").unwrap();
        let outside = scratch("outside").join("disari.bin");
        fs::write(&outside, b"x").unwrap();
        let q = root.join(QUARANTINE_DIR);
        fs::create_dir_all(&q).unwrap();
        fs::write(q.join("k.bin"), b"x").unwrap();
        let dir = root.join("klasor");
        fs::create_dir_all(&dir).unwrap();
        let sent = Mutex::new(0);
        let send = |_: &Path| {
            *sent.lock().unwrap() += 1;
            Ok(())
        };
        let ti = |p: &Path| TrashItem { path: p.to_string_lossy().into_owned(), size: 1, mtime: 0 };
        for bad in [ti(&outside), ti(&q.join("k.bin")), ti(&dir), ti(&root.join("yok.bin"))] {
            assert!(trash_core(&root, &[item(&good), bad], &send).is_err());
        }
        assert_eq!(*sent.lock().unwrap(), 0, "hicbir dosya gonderilmedi");
        assert!(trash_core(&root, &[], &send).is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn changed_file_after_report_is_rejected() {
        let root = scratch("changed");
        let f = root.join("r.bin");
        fs::write(&f, b"kisa").unwrap();
        let it = item(&f);
        fs::write(&f, b"cok daha uzun icerik").unwrap();
        let err = trash_core(&root, &[it], &|_| Ok(())).unwrap_err();
        assert!(err.contains("rapordan sonra değişti"), "{err}");
        let _ = fs::remove_dir_all(&root);
    }
}

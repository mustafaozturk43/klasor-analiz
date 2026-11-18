//! Planli toplu tasima: dosyalar partiler halinde, her parti tasinmadan ONCE Gecmis'e
//! yazilarak tasinir (islem yarida kesilse de geri alinabilir). Tarihe Gore gruplama ve
//! kural motoru ortak kullanir; ust katman plani dogrular, bu islev yalnizca uygular.

use crate::models::{MoveRecord, UndoRecord};
use crate::operations::MoveOutcome;
use crate::organize_cleanup::prune_empty_dirs;
use crate::organize_models::FailedMove;
use crate::rename::FileStamps;
use crate::{file_ops, history, AppState};
use std::fs;
use std::path::{Path, PathBuf};

pub const CHANGED: &str = "Dosyalar ön izlemeden sonra değişti; yeniden ön izleyin.";

/// Gecmis dosyasi parti basina bir kez yazilir; buyuk islerde yazim sayisi ~40 ile sinirlanir (25..250 dosya).
pub fn batch_size(total: usize) -> usize {
    (total / 40).clamp(25, 250)
}

/// Her kaynagin boyut ve degisiklik tarihi onizlemedeki gibi mi, hedef hala bos mu?
pub fn ensure_unchanged(entries: &[MoveRecord], stamps: &FileStamps) -> Result<(), String> {
    for (entry, st) in entries.iter().zip(stamps) {
        let meta = fs::metadata(&entry.from).map_err(|_| CHANGED.to_string())?;
        if (meta.len(), meta.modified().ok()) != *st || Path::new(&entry.to).exists() {
            return Err(CHANGED.into());
        }
    }
    Ok(())
}

/// `created_dirs`: plan sirasinda olmayan, bu islemle olusacak klasorler (basarisiz tasimadan bos kalirsa kaldirilir).
pub(crate) fn run_batches(
    app: &AppState,
    id: &str,
    tag: String,
    entries: &[MoveRecord],
    root: &Path,
    created_dirs: &[String],
    progress: &dyn Fn(u64, u64),
) -> Result<MoveOutcome, String> {
    let mut guard = app.history.lock().map_err(|e| e.to_string())?;
    history::push(&mut guard, UndoRecord { id: id.to_string(), timestamp: crate::now_ms(), tag, moves: Vec::new() });
    history::save(&app.dir, &guard)?;
    let mut out = MoveOutcome { moved: Vec::new(), failed: Vec::new(), removed_dirs: 0 };
    let total = entries.len() as u64;
    progress(0, total);
    for chunk in entries.chunks(batch_size(entries.len())) {
        let mut ready = Vec::with_capacity(chunk.len());
        for entry in chunk {
            match Path::new(&entry.to).parent().map(fs::create_dir_all).transpose() {
                Ok(_) => ready.push(entry),
                Err(e) => out.failed.push(FailedMove { path: entry.from.clone(), error: e.to_string() }),
            }
        }
        // Once gecmise yaz (parti basina tek dosya yazimi), sonra tasi.
        guard[0].moves.extend(ready.iter().map(|e| (*e).clone()));
        history::save(&app.dir, &guard)?;
        let mut failed_from = Vec::new();
        for entry in ready {
            match file_ops::transfer(Path::new(&entry.from), Path::new(&entry.to)) {
                Ok(()) => out.moved.push(entry.clone()),
                Err(error) => {
                    failed_from.push(entry.from.clone());
                    out.failed.push(FailedMove { path: entry.from.clone(), error });
                }
            }
        }
        if !failed_from.is_empty() {
            guard[0].moves.retain(|m| !failed_from.contains(&m.from));
            history::save(&app.dir, &guard)?;
        }
        progress((out.moved.len() + out.failed.len()) as u64, total);
    }
    if guard[0].moves.is_empty() {
        guard.remove(0);
    }
    history::save(&app.dir, &guard)?;
    out.removed_dirs = prune_empty_dirs(root, created_dirs.iter().map(PathBuf::from), &|_| false);
    Ok(out)
}

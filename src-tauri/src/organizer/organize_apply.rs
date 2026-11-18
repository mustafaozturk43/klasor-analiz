//! Bir duzenleme planini diske uygular. Hem elle onizlenen duzenleme hem de
//! izlenen klasorun otomatik yerlestirmesi bu yolu kullanir: once tum kaynak ve
//! hedefler dogrulanir, tasimalar dosyaya dokunmadan once tek bir gecmis
//! kaydina yazilir, basarisiz olanlar kayittan cikarilir.

use crate::models::{MoveRecord, UndoRecord};
use crate::organize_models::{FailedMove, OrganizeAction, OrganizeEntry, OrganizeOutcome, OrganizePlan};
use crate::{file_ops, history, AppState};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn mtime_of(meta: &fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub(crate) fn run_plan(app: &AppState, plan: &OrganizePlan, tag: &str) -> Result<OrganizeOutcome, String> {
    let work: Vec<&OrganizeEntry> = plan
        .entries
        .iter()
        .filter(|e| matches!(e.action, OrganizeAction::Move | OrganizeAction::Quarantine) && e.to.is_some())
        .collect();
    // Hicbir sey tasinmadan once tum kaynak ve hedefler dogrulanir.
    for e in &work {
        let meta = fs::metadata(&e.from).map_err(|_| format!("Dosya bulunamadı: {}. Yeniden önizleyin.", e.from))?;
        if !meta.is_file() || meta.len() != e.size || mtime_of(&meta) != e.mtime {
            return Err(format!("Dosya önizlemeden sonra değişti: {}. Yeniden önizleyin.", e.from));
        }
        if Path::new(e.to.as_deref().unwrap_or_default()).exists() {
            return Err(format!("Hedefte artık aynı adlı bir dosya var: {}. Yeniden önizleyin.", e.to.as_deref().unwrap_or_default()));
        }
    }
    let mut outcome = OrganizeOutcome { moved: 0, moved_bytes: 0, quarantined: 0, quarantined_bytes: 0, failed: Vec::new(), removed_dirs: 0 };
    if work.is_empty() {
        return Ok(outcome);
    }

    // Planlanan tum tasimalar once gecmise yazilir: uygulama yarida kapanirsa
    // geri alma, tasinmamis (hedefi olmayan) kayitlari atlar.
    let mut guard = app.history.lock().map_err(|e| e.to_string())?;
    let moves: Vec<MoveRecord> = work
        .iter()
        .map(|e| MoveRecord { from: e.from.clone(), to: e.to.clone().unwrap_or_default() })
        .collect();
    history::push(&mut guard, UndoRecord { id: plan.id.clone(), timestamp: crate::now_ms(), tag: tag.into(), moves });
    history::save(&app.dir, &guard)?;

    let mut done: Vec<bool> = Vec::with_capacity(work.len());
    for e in &work {
        let to = PathBuf::from(e.to.as_deref().unwrap_or_default());
        let result = to
            .parent()
            .map_or(Ok(()), |p| fs::create_dir_all(p).map_err(|err| err.to_string()))
            .and_then(|_| file_ops::transfer(Path::new(&e.from), &to));
        let ok = match result {
            Ok(()) if e.action == OrganizeAction::Quarantine => {
                outcome.quarantined += 1;
                outcome.quarantined_bytes += e.size;
                true
            }
            Ok(()) => {
                outcome.moved += 1;
                outcome.moved_bytes += e.size;
                true
            }
            Err(error) => {
                outcome.failed.push(FailedMove { path: e.from.clone(), error });
                false
            }
        };
        done.push(ok);
    }
    if plan.remove_empty_dirs {
        let root = PathBuf::from(&plan.root_path);
        let moved = work.iter().zip(&done).filter(|(_, ok)| **ok).map(|(e, _)| Path::new(e.from.as_str()));
        outcome.removed_dirs = crate::organize_cleanup::remove_emptied_dirs(&root, moved);
    }
    // Basarisiz tasimalar kayittan cikarilir; hic tasima olmadiysa kayit silinir.
    let mut flags = done.into_iter();
    guard[0].moves.retain(|_| flags.next().unwrap_or(false));
    if guard[0].moves.is_empty() {
        guard.remove(0);
    }
    history::save(&app.dir, &guard)?;
    Ok(outcome)
}

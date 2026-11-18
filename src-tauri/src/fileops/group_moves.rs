//! Validated group batches, with history written before each no-clobber move.
use crate::{AppState, history, models::*, mover, naming};
use serde::Deserialize;
use std::{collections::HashSet, fs, path::PathBuf};
use tauri::State;

#[derive(Deserialize)]
pub struct NamedGroup { pub group: FileGroup, pub tag: String }

#[tauri::command(async)]
pub(crate) fn move_named_groups(
    state: State<'_, AppState>, root_path: String, root_b: Option<String>, batches: Vec<NamedGroup>,
    keep_policy: KeepPolicy, verify_large: bool,
) -> Result<usize, String> {
    let root = fs::canonicalize(root_path).map_err(|e| e.to_string())?;
    let b = root_b.filter(|r| !r.is_empty()).map(fs::canonicalize).transpose().map_err(|e| e.to_string())?;
    let mut seen = HashSet::new();
    for batch in &batches {
        if batch.tag.trim().is_empty() { return Err("Grup adı boş olamaz.".into()); }
        for file in &batch.group.files {
            let path = fs::canonicalize(&file.path).map_err(|e| e.to_string())?;
            if !(path.starts_with(&root) || b.as_ref().is_some_and(|b| path.starts_with(b)))
                || !seen.insert(path.clone()) { return Err("Geçersiz veya tekrarlanan dosya.".into()); }
            let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
            let mtime = metadata.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
            if !metadata.is_file() || metadata.len() != file.size || mtime != file.mtime {
                return Err("Dosyalar değişti; yeniden tarayın.".into());
            }
            if path.components().any(|p| p.as_os_str().to_string_lossy().eq_ignore_ascii_case(QUARANTINE_DIR)) {
                return Err("Karantina kaynak olarak kullanılamaz.".into());
            }
        }
    }
    let mut guard = state.history.lock().map_err(|e| e.to_string())?;
    let id = crate::now_ms();
    history::push(&mut guard, UndoRecord { id: format!("MV-{id}"), timestamp: id, tag: "Grup taşıma".into(), moves: Vec::new() });
    history::save(&state.dir, &guard)?;
    let mut count = 0;
    for batch in batches {
        let tag = naming::sanitize_tag(&batch.tag);
        let mut log = |record: MoveRecord| {
            guard[0].moves.push(record);
            history::save(&state.dir, &guard)
        };
        match mover::apply_move_logged(&PathBuf::from(&root), &[batch.group], &tag, keep_policy, verify_large, &mut log) {
            Ok(result) => { count += result.moved_to_tag.len() + result.moved_to_quarantine.len();
                if !result.skipped.is_empty() { return Err(format!("{count} dosya taşındı; {} dosya tam hash doğrulamasında atlandı. Geri al kullanılabilir.", result.skipped.len())); }
            }
            Err(e) => return Err(format!("{count} dosya önceki gruplarda taşındı. İşlem durdu: {e}. Geri al kullanılabilir.")),
        }
    }
    Ok(count)
}

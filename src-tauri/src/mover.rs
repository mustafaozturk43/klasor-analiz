//! Etiket klasoru olusturma, dosya tasima, isim catismasi cozumu ve geri alma.
//! Tutulacak dosya secimi `KeepPolicy` ile yapilir; buyuk dosyalar icin
//! tam hash dogrulamasi `verify_large` secenegiyle (F3) acilabilir.

use crate::hasher;
use crate::models::{
    FileGroup, FileItem, GroupType, KeepPolicy, MoveRecord, MoveResult, UndoRecord,
    LARGE_FILE_THRESHOLD, QUARANTINE_DIR,
};
use crate::naming::{sanitize_tag, unique_dest};
use std::fs;
use std::path::{Path, PathBuf};

/// Tutulacak dosyanin indeksini politikaya gore secer.
fn kept_index(files: &[FileItem], policy: KeepPolicy) -> usize {
    match policy {
        KeepPolicy::First => 0,
        KeepPolicy::Newest => (0..files.len())
            .max_by_key(|&i| files[i].mtime)
            .unwrap_or(0),
        KeepPolicy::Oldest => (0..files.len())
            .min_by_key(|&i| files[i].mtime)
            .unwrap_or(0),
        KeepPolicy::ShortestPath => (0..files.len())
            .min_by_key(|&i| files[i].path.chars().count())
            .unwrap_or(0),
    }
}

/// Secili gruplari etiket klasorune tasir.
///
/// - Exact grup: politikaya gore tutulan dosya `<root>/<tag>/`'a, kopyalar
///   `<root>/_Kopyalar_Karantina/<tag>/`'a.
/// - Similar grup: tum dosyalar `<root>/<tag>/`'a.
/// - `verify_large = true` ise 500 MB+ dosyalarin tam BLAKE3 hash'i tasima
///   oncesi yeniden hesaplanir; tutulan dosyayla hash'i eslesmeyenler
///   dokunulmadan birakilir (F3, opt-in).
pub fn apply_move(
    root: &Path,
    groups: &[FileGroup],
    tag: &str,
    policy: KeepPolicy,
    verify_large: bool,
) -> Result<MoveResult, String> {
    apply_move_logged(root, groups, tag, policy, verify_large, &mut |_| Ok(()))
}

pub fn apply_move_logged(
    root: &Path, groups: &[FileGroup], tag: &str, policy: KeepPolicy, verify_large: bool,
    log: &mut dyn FnMut(MoveRecord) -> Result<(), String>,
) -> Result<MoveResult, String> {
    let safe_tag = sanitize_tag(tag);
    let tag_dir = root.join(&safe_tag);
    let quarantine_dir = root.join(QUARANTINE_DIR).join(&safe_tag);

    fs::create_dir_all(&tag_dir).map_err(|e| e.to_string())?;

    let mut moved_to_tag: Vec<MoveRecord> = Vec::new();
    let mut moved_to_quarantine: Vec<MoveRecord> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    for group in groups {
        if group.files.is_empty() { continue; }
        match group.group_type {
            GroupType::Exact => {
                // Tutulacak dosya politikayla secilir (F3 dogrulamasinin referansi).
                let keep = kept_index(&group.files, policy);
                // F3: buyuk dosyalar icin tam hash dogrulamasi.
                if verify_large && large_verification_needed(group) {
                    let kept_path = group.files[keep].path.clone();
                    let skip = verify_and_skip_mismatch(&kept_path, &group.files);
                    if !skip.is_empty() {
                        skipped.extend(skip);
                    }
                }
                for (i, file) in group.files.iter().enumerate() {
                    if skipped.contains(&file.path) {
                        continue;
                    }
                    let is_kept = i == keep;
                    let dest_dir = if is_kept { &tag_dir } else { &quarantine_dir };
                    fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;
                    let dest = unique_dest(dest_dir, &file.name);
                    log(MoveRecord { from: file.path.clone(), to: dest.to_string_lossy().into_owned() })?;
                    crate::file_ops::transfer(Path::new(&file.path), &dest)?;

                    let rec = MoveRecord {
                        from: file.path.clone(),
                        to: dest.to_string_lossy().into_owned(),
                    };
                    if is_kept {
                        moved_to_tag.push(rec);
                    } else {
                        moved_to_quarantine.push(rec);
                    }
                }
            }
            GroupType::Similar => {
                for file in &group.files {
                    let dest = unique_dest(&tag_dir, &file.name);
                    log(MoveRecord { from: file.path.clone(), to: dest.to_string_lossy().into_owned() })?;
                    crate::file_ops::transfer(Path::new(&file.path), &dest)?;
                    moved_to_tag.push(MoveRecord {
                        from: file.path.clone(),
                        to: dest.to_string_lossy().into_owned(),
                    });
                }
            }
        }
    }

    Ok(MoveResult {
        tag: safe_tag,
        moved_to_tag,
        moved_to_quarantine,
        skipped,
    })
}

/// Grupta 500 MB+ dosya varsa tasima oncesi tam hash dogrulamasi gerektigi.
fn large_verification_needed(group: &FileGroup) -> bool {
    group.files.iter().any(|f| f.size >= LARGE_FILE_THRESHOLD)
}

/// Grup uyelerinin tam BLAKE3 hash'ini hesaplar; tutulan dosyanin hash'iyle
/// eslesmeyenlerin yollarini dondurur (bu dosyalar tasimadan muaf tutulur).
fn verify_and_skip_mismatch(kept_path: &str, files: &[FileItem]) -> Vec<String> {
    let kept_full = hasher::hash_full(Path::new(kept_path)).unwrap_or_default();
    if kept_full.is_empty() {
        // Tutulan dosya okunamiyorsa dogrulamada bulunamaz; hicbirini atla.
        return files.iter().map(|f| f.path.clone()).collect();
    }
    files
        .iter()
        .filter(|f| f.path != kept_path)
        .filter(|f| {
            let full = hasher::hash_full(Path::new(&f.path)).unwrap_or_default();
            full.is_empty() || full != kept_full
        })
        .map(|f| f.path.clone())
        .collect()
}

/// Son tasima islemini ters sirada geri alir.
pub fn undo_move(undo: &UndoRecord) -> Result<(), String> {
    for rec in &undo.moves {
        if !Path::new(&rec.to).exists() && !Path::new(&rec.from).exists() {
            return Err(format!("Geri alınacak dosya bulunamadı: {}", rec.to));
        }
        if Path::new(&rec.to).exists() && Path::new(&rec.from).exists() {
            return Err(format!("Geri alma hedefi dolu: {}", rec.from));
        }
    }
    for rec in undo.moves.iter().rev() {
        let from = PathBuf::from(&rec.from);
        let to = PathBuf::from(&rec.to);

        if to.exists() {
            if let Some(parent) = from.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            crate::file_ops::transfer(&to, &from)?;
        }
    }
    Ok(())
}

/// Karantina klasorunde bekleyen dosya sayisini dondurur (jwalk, tum alt klasorler).
pub fn quarantine_count(root: &Path) -> Result<u64, String> {
    let q = root.join(QUARANTINE_DIR);
    if !q.exists() {
        return Ok(0);
    }
    let mut count = 0u64;
    for entry in jwalk::WalkDir::new(&q).skip_hidden(false) {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().is_file() {
            count += 1;
        }
    }
    Ok(count)
}

/// Belirli bir etiketin karantinadaki kopyalarini `<root>/<tag>/` klasorune geri
/// tasir (isim catismasi yine `dosya (1).uzanti` ile cozulur). Donus: tasinan adet.
pub fn restore_from_quarantine(root: &Path, tag: &str) -> Result<usize, String> {
    let safe_tag = sanitize_tag(tag);
    let quarantine_dir = root.join(QUARANTINE_DIR).join(&safe_tag);
    if !quarantine_dir.exists() {
        return Ok(0);
    }
    let tag_dir = root.join(&safe_tag);
    fs::create_dir_all(&tag_dir).map_err(|e| e.to_string())?;

    let mut restored = 0usize;
    for entry in jwalk::WalkDir::new(&quarantine_dir)
        .skip_hidden(false)
        .max_depth(1)
    {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let dest = unique_dest(&tag_dir, &name);
        fs::rename(&path, &dest).map_err(|e| e.to_string())?;
        restored += 1;
    }
    Ok(restored)
}

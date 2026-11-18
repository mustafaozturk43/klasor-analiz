//! Gelen kutusunda secili dosyalari sablonla adlandirma. Ornek:
//! `{OzelMetin}_{Grup}_{tarih}_{sıra}` -> `Kasim_Faturalari_Dokümanlar_2026-11-10_01.pdf`.
//! Mevcut sablon motorunu (`rename_template::render`) ve onizle -> uygula -> geri al
//! akisini (`rename::apply_pending`) kullanir. Sira numarasi verilen liste sirasidir.
//! Dosyalar bulunduklari klasorde kalir; ayni ad varsa `ad_1.uzanti`. Otomatik tasima
//! kapaliyken izlenen klasorde bekleyen dosyalar da adlandirilabilir.

use crate::models::INBOX_DIR;
use crate::inbox_sources::is_pending_source;
use crate::operations::{backup_root, source_folders};
use crate::path_guard::ensure_inside;
use crate::rename::{FileStamps, Pending, RenameEntry, RenamePlan, RenameState};
use crate::rename_template::{render, Fields};
use crate::settings::SettingsState;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

const MAX_PATH_CHARS: usize = 259;
pub const HISTORY_LABEL: &str = "Gelen kutusu adlandırma";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRenameOptions {
    pub template: String,
    #[serde(default)]
    pub custom_text: String,
    pub start: u64,
    pub pad: usize,
    /// Doluysa `{Grup}` her dosyada bu degeri alir (yoksa grup klasorunden turetilir).
    #[serde(default)]
    pub group: Option<String>,
    /// `paths` ile ayni sirada; `Some` ise tarih alanlari mtime yerine bunu kullanir.
    #[serde(default)]
    pub dates: Vec<Option<u64>>,
    /// Bos alanlarin biraktigi `__` ve bastaki/sondaki ayraclari temizler.
    #[serde(default)]
    pub tidy: bool,
}

fn free_target(dir: &Path, stem: &str, ext: &str, reserved: &mut HashSet<String>) -> PathBuf {
    let mut n = 0;
    loop {
        let extra = if n == 0 { String::new() } else { format!("_{n}") };
        let candidate = dir.join(format!("{stem}{extra}{ext}"));
        if !candidate.exists() && reserved.insert(candidate.to_string_lossy().to_lowercase()) {
            return candidate;
        }
        n += 1;
    }
}

pub fn plan_list_rename(inbox: &Path, paths: &[String], opts: &ListRenameOptions) -> Result<(RenamePlan, FileStamps), String> {
    plan_rename_in(&[inbox.to_path_buf()], "Yalnızca gelen kutusundaki dosyalar adlandırılabilir", paths, opts)
}

/// Gelen kutusu dosyalari ve izlenen klasorde bekleyenler (kaynak klasorun dogrudan icindekiler).
pub fn plan_inbox_or_pending(inbox: &Path, sources: &[PathBuf], paths: &[String], opts: &ListRenameOptions) -> Result<(RenamePlan, FileStamps), String> {
    for raw in paths {
        if ensure_inside(inbox, Path::new(raw)).is_err() && !is_pending_source(sources, Path::new(raw)) {
            return Err(format!("Yalnızca gelen kutusundaki ya da izlenen klasörde bekleyen dosyalar adlandırılabilir: {raw}"));
        }
    }
    let bases: Vec<PathBuf> = std::iter::once(inbox.to_path_buf()).chain(sources.iter().cloned()).collect();
    plan_rename_in(&bases, "Yalnızca gelen kutusundaki dosyalar adlandırılabilir", paths, opts)
}

fn tidy(stem: &str) -> String {
    let mut out = String::new();
    for c in stem.chars() {
        if !(c == '_' && out.ends_with('_')) {
            out.push(c);
        }
    }
    out.trim_matches(|c: char| c == '_' || c == '-' || c.is_whitespace()).trim_end_matches('.').to_string()
}

/// Sablonla adlandirma plani; dosyalar `bases` klasorlerinden birinin icinde olmali.
pub fn plan_rename_in(bases: &[PathBuf], outside_msg: &str, paths: &[String], opts: &ListRenameOptions) -> Result<(RenamePlan, FileStamps), String> {
    if paths.is_empty() {
        return Err("Önce dosya seçin.".into());
    }
    if opts.template.trim().is_empty() {
        return Err("Şablon boş olamaz.".into());
    }
    let custom = opts.custom_text.trim();
    render(&opts.template, &Fields { stem: "a", n: 1, pad: 1, mtime: 0, folder: "a", group: "g", custom })?;
    let mut seen = HashSet::new();
    let mut reserved = HashSet::new();
    let (mut entries, mut stamps) = (Vec::new(), Vec::new());
    for (i, raw) in paths.iter().enumerate() {
        let path = PathBuf::from(raw);
        if !bases.iter().any(|b| ensure_inside(b, &path).is_ok()) {
            return Err(format!("{outside_msg}: {raw}"));
        }
        if !seen.insert(raw.to_lowercase()) {
            return Err(format!("Aynı dosya iki kez seçildi: {raw}"));
        }
        let meta = std::fs::metadata(&path).map_err(|e| format!("{raw}: {e}"))?;
        if !meta.is_file() {
            return Err(format!("Yalnızca dosyalar adlandırılabilir: {raw}"));
        }
        stamps.push((meta.len(), meta.modified().ok()));
        let mtime = meta.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
        let mtime = opts.dates.get(i).copied().flatten().unwrap_or(mtime);
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let ext = path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        let dir = path.parent().map(Path::to_path_buf).unwrap_or_default();
        let folder = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        // Grup klasoru gun klasorunun altindadir; dosya dogrudan gun klasorundeyse grup bos.
        let group = if let Some(g) = &opts.group { g.trim().to_string() } else if dir.parent().and_then(Path::parent).is_some_and(|p| p.ends_with(INBOX_DIR)) { folder.clone() } else { String::new() };
        let fields = Fields { stem: &stem, n: opts.start + i as u64, pad: opts.pad.min(9), mtime, folder: &folder, group: &group, custom };
        let new_stem = render(&opts.template, &fields)?;
        let new_stem = if opts.tidy { tidy(&new_stem) } else { new_stem };
        let mut entry = RenameEntry { from: raw.clone(), to: None, warning: None };
        let wanted = format!("{new_stem}{ext}");
        if new_stem.is_empty() {
            entry.warning = Some("Şablon bu dosya için boş ad üretti; atlanacak.".into());
        } else if wanted.to_lowercase() == name.to_lowercase() {
            entry.warning = Some("Ad değişmiyor; atlanacak.".into());
        } else {
            let target = free_target(&dir, &new_stem, &ext, &mut reserved);
            if target.to_string_lossy().chars().count() > MAX_PATH_CHARS {
                entry.warning = Some("Yeni yol 260 karakteri aşıyor; atlanacak.".into());
            } else {
                if target.file_name().is_some_and(|n| n.to_string_lossy() != wanted.as_str()) {
                    entry.warning = Some("Bu ad dolu; numara eklendi.".into());
                }
                entry.to = Some(target.to_string_lossy().into_owned());
            }
        }
        entries.push(entry);
    }
    let id = format!("RN-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos());
    Ok((RenamePlan { id, entries }, stamps))
}

/// Onizleme planini olusturur ve uygulama icin bekletir (`apply_rename_plan` ile uygulanir).
#[tauri::command(async)]
pub(crate) fn preview_inbox_rename(state: State<'_, RenameState>, settings: State<'_, SettingsState>, paths: Vec<String>, options: ListRenameOptions) -> Result<RenamePlan, String> {
    let inbox = backup_root(&settings)?.join(INBOX_DIR);
    let (plan, stamps) = plan_inbox_or_pending(&inbox, &source_folders(&settings)?, &paths, &options)?;
    *state.pending.lock().map_err(|e| e.to_string())? = Some(Pending { plan: plan.clone(), stamps, label: HISTORY_LABEL });
    Ok(plan)
}

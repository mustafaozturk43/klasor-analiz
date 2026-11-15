//! Sablonla toplu yeniden adlandirma. Ornek: `Fatura_{yıl}-{ay}_{sıra}` ->
//! `Fatura_2026-03_001.pdf`. Uzanti her zaman korunur, dosya klasorunde kalir.
//! Onizleme sunucuda bekletilir; uygulama `rename::apply_pending` ile yapilir
//! (dosya degistiyse veya hedef dolduysa hicbir sey yapilmaz, islem geri alinabilir).

use crate::models::normalize_exts;
use crate::organize_rules::is_temp_or_system;
use crate::rename::{FileStamps, Pending, RenameEntry, RenamePlan, RenameState};
use crate::{file_ops, organize_walk};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

const MAX_PATH_CHARS: usize = 259;
const BAD_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateOptions {
    pub template: String,
    pub start: u64,
    /// Sira numarasinin en az basamak sayisi (3 -> 001).
    pub pad: usize,
    /// "name" (ada gore) veya "date" (degisiklik tarihine gore) siralama.
    pub sort: String,
    /// Bos ise tum dosyalar (gecici/sistem dosyalari haric).
    pub extensions: Vec<String>,
    pub include_subdirs: bool,
    pub exclude_folders: Vec<String>,
}

/// Sablon alanlari: dosya basina degisen degerler.
/// `group` ve `custom` yalnizca gelen kutusu adlandirmasinda doludur.
pub struct Fields<'a> { pub stem: &'a str, pub n: u64, pub pad: usize, pub mtime: u64, pub folder: &'a str, pub group: &'a str, pub custom: &'a str }

/// Sablonu doldurur. Bilinmeyen alan veya kapanmamis `{` hata verir.
pub fn render(template: &str, f: &Fields) -> Result<String, String> {
    let date = file_ops::local_date_suffix(f.mtime); // gg-aa-yyyy (yerel saat)
    let (day, month, year) = (&date[0..2], &date[3..5], &date[6..]);
    let mut out = String::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let close = after.find('}').ok_or("Şablonda kapanmamış “{” var.")?;
        let token = after[..close].trim().to_lowercase();
        match token.as_str() {
            "ad" => out.push_str(f.stem),
            "sıra" | "sira" | "n" => out.push_str(&format!("{:0width$}", f.n, width = f.pad)),
            "tarih" => out.push_str(&format!("{year}-{month}-{day}")),
            "yıl" | "yil" => out.push_str(year),
            "ay" => out.push_str(month),
            "gün" | "gun" => out.push_str(day),
            "klasör" | "klasor" => out.push_str(f.folder),
            "grup" => out.push_str(f.group),
            "özelmetin" | "ozelmetin" | "metin" => out.push_str(f.custom),
            other => return Err(format!("Bilinmeyen alan: {{{other}}}. Kullanılabilenler: {{ad}} {{sıra}} {{tarih}} {{yıl}} {{ay}} {{gün}} {{klasör}} {{Grup}} {{OzelMetin}}")),
        }
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    let cleaned: String = out.chars().map(|c| if BAD_CHARS.contains(&c) || c.is_control() { '_' } else { c }).collect();
    Ok(cleaned.trim().trim_end_matches('.').to_string())
}

pub fn plan_template_rename(root: &Path, opts: &TemplateOptions) -> Result<(RenamePlan, FileStamps), String> {
    if opts.template.trim().is_empty() {
        return Err("Şablon boş olamaz.".into());
    }
    // Soz dizimi hatasini dosya taramadan once bildir.
    render(&opts.template, &Fields { stem: "a", n: 1, pad: 1, mtime: 0, folder: "a", group: "", custom: "" })?;
    let never = || false;
    let walked = organize_walk::collect(root, opts.include_subdirs, &opts.exclude_folders, true, &|_| {}, &never)?;
    let exts = normalize_exts(&opts.extensions);
    let mut files: Vec<_> = walked.files.into_iter()
        .filter(|c| !is_temp_or_system(&c.name) && (exts.is_empty() || exts.contains(&c.ext)))
        .collect();
    if files.is_empty() {
        return Err("Bu filtreyle adlandırılacak dosya bulunamadı.".into());
    }
    if opts.sort == "date" {
        files.sort_by(|a, b| (a.mtime, &a.path).cmp(&(b.mtime, &b.path)));
    } else {
        files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()).then_with(|| a.path.cmp(&b.path)));
    }

    let mut reserved: HashSet<String> = HashSet::new();
    let mut entries = Vec::new();
    let mut stamps = Vec::new();
    for (i, c) in files.iter().enumerate() {
        // Uygulama adimi tam zaman damgasini karsilastirir; saniyeye yuvarlanmis deger yetmez.
        let meta = std::fs::metadata(&c.path).map_err(|e| format!("{}: {e}", c.path.display()))?;
        stamps.push((meta.len(), meta.modified().ok()));
        let stem = Path::new(&c.name).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let ext = Path::new(&c.name).extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        let dir = c.path.parent().map(Path::to_path_buf).unwrap_or_default();
        let folder = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let fields = Fields { stem: &stem, n: opts.start + i as u64, pad: opts.pad.min(9), mtime: c.mtime, folder: &folder, group: "", custom: "" };
        let mut entry = RenameEntry { from: c.path.to_string_lossy().into_owned(), to: None, warning: None };
        let new_stem = render(&opts.template, &fields)?;
        let wanted = format!("{new_stem}{ext}");
        if new_stem.is_empty() {
            entry.warning = Some("Şablon bu dosya için boş ad üretti; atlanacak.".into());
        } else if wanted == c.name {
            entry.warning = Some("Ad değişmiyor; atlanacak.".into());
        } else if wanted.to_lowercase() == c.name.to_lowercase() {
            entry.warning = Some("Yalnızca büyük/küçük harf değişiyor; Windows'ta güvenli değil, atlanacak.".into());
        } else {
            let target = free_target(&dir, &new_stem, &ext, &mut reserved);
            if target.to_string_lossy().chars().count() > MAX_PATH_CHARS {
                entry.warning = Some("Yeni yol 260 karakteri aşıyor; atlanacak.".into());
            } else {
                if target.file_name().map(|n| n.to_string_lossy() != wanted.as_str()).unwrap_or(false) {
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

/// Diskte ve bu planda bos olan ilk adi bulur: `ad.uzt`, `ad (1).uzt`, ...
fn free_target(dir: &Path, stem: &str, ext: &str, reserved: &mut HashSet<String>) -> PathBuf {
    let mut n = 0;
    loop {
        let extra = if n == 0 { String::new() } else { format!(" ({n})") };
        let candidate = dir.join(format!("{stem}{extra}{ext}"));
        if !candidate.exists() && reserved.insert(candidate.to_string_lossy().to_lowercase()) {
            return candidate;
        }
        n += 1;
    }
}

#[tauri::command(async)]
pub fn preview_template_rename(state: State<'_, RenameState>, root_path: String, options: TemplateOptions) -> Result<RenamePlan, String> {
    let (plan, stamps) = plan_template_rename(Path::new(&root_path), &options)?;
    *state.pending.lock().map_err(|e| e.to_string())? = Some(Pending { plan: plan.clone(), stamps, label: "Şablonla adlandırma" });
    Ok(plan)
}

#[cfg(test)]
#[path = "rename_template_tests.rs"]
mod tests;

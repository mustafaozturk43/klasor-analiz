//! Grup adi parcalama ("Faturalar\Mayıs") ve hedef klasorde bos dosya adi bulma.

use crate::organize_rules::check_folder_name;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Windows'un aygit adlari: uzantili olsalar da klasor/dosya adi olamazlar.
fn is_device_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or("").trim().to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((stem.starts_with("COM") || stem.starts_with("LPT")) && stem.len() == 4 && stem.as_bytes()[3].is_ascii_digit() && stem.as_bytes()[3] != b'0')
}

/// Grup adini `\` veya `/` ile alt klasor parcalarina ayirir. Bos metin, hedefin
/// kendisine tasima demektir. `.`/`..`, bos parca, yasak karakter ve aygit adlari reddedilir.
pub fn split_group(group: &str) -> Result<Vec<String>, String> {
    let text = group.trim();
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for raw in text.split(['\\', '/']) {
        let seg = raw.trim();
        if seg.is_empty() {
            return Err("Klasör adları arasında boş bölüm var.".into());
        }
        if seg.chars().all(|c| c == '.') {
            return Err(format!("“{seg}” klasör adı olarak kullanılamaz."));
        }
        let name = check_folder_name("Klasör", seg)?;
        if is_device_name(&name) {
            return Err(format!("“{name}” Windows'ta ayrılmış bir ad."));
        }
        out.push(name);
    }
    Ok(out)
}

/// `dir` icinde ne diskte ne de `reserved` icinde olan bir yol: `ad.uzanti`,
/// sonra `ad (1).uzanti`, `ad (2).uzanti`... (`reserved` kucuk harfli tam yol tutar).
pub fn free_dest(dir: &Path, name: &str, reserved: &mut HashSet<String>) -> PathBuf {
    let p = Path::new(name);
    let stem = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| name.to_string());
    let ext = p.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let mut n = 0;
    loop {
        let candidate = if n == 0 { dir.join(name) } else { dir.join(format!("{stem} ({n}){ext}")) };
        if !candidate.exists() && reserved.insert(candidate.to_string_lossy().to_lowercase()) {
            return candidate;
        }
        n += 1;
    }
}

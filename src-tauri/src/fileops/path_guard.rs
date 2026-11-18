//! Frontend'den gelen yollarin guvenlik kontrolleri: yol gercekten verilen
//! klasorun icinde mi, karantinada mi? Kontrol kanonik yolla yapilir, islem ise
//! kullanicinin gordugu yolla (gecmis kayitlari okunur kalsin diye).

use crate::models::QUARANTINE_DIR;
use std::fs;
use std::path::{Path, PathBuf};

pub fn canonical(p: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(p).map_err(|e| format!("{}: {e}", p.display()))
}

/// `path` mevcut ve `base`'in (kendisi degil) icinde olmali.
pub fn ensure_inside(base: &Path, path: &Path) -> Result<(), String> {
    let b = canonical(base)?;
    let p = canonical(path)?;
    if p == b || !p.starts_with(&b) {
        return Err(format!("İzin verilmeyen konum: {}", path.display()));
    }
    Ok(())
}

/// Yol, kokun karantina klasorunde mi? (kanonik karsilastirma)
pub fn in_quarantine(root: &Path, path: &Path) -> bool {
    match (canonical(&root.join(QUARANTINE_DIR)), canonical(path)) {
        (Ok(q), Ok(p)) => p.starts_with(q),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_outside_and_base_itself() {
        let base = std::env::temp_dir().join(format!("sfo_guard_{}", std::process::id()));
        fs::create_dir_all(base.join("in")).unwrap();
        fs::write(base.join("in/a.txt"), b"a").unwrap();
        assert!(ensure_inside(&base, &base.join("in/a.txt")).is_ok());
        assert!(ensure_inside(&base, &base).is_err());
        assert!(ensure_inside(&base.join("in"), &base.join("in/../in/../in/a.txt")).is_ok());
        assert!(ensure_inside(&base.join("in"), &std::env::temp_dir()).is_err());
        fs::remove_dir_all(&base).unwrap();
    }
}

//! Dosya adi/etiket adindirme yardimcilari: etiket temizleme + isim catismasi cozumu.

use std::path::{Path, PathBuf};

/// Etiket adini guvenli bir klasor adina cevirir.
pub fn sanitize_tag(tag: &str) -> String {
    let cleaned: String = tag
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c => c,
        })
        .collect();

    let trimmed = cleaned.trim().trim_matches('.').to_string();
    if trimmed.is_empty() {
        "Etiket".to_string()
    } else {
        trimmed
    }
}

/// Cakisma numarasinin bicimi.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Suffix {
    /// `dosya (1).pdf` — duzenleme, etiket ve karantina islemleri.
    Paren,
    /// `dosya_1.pdf` — gelen kutusu.
    Underscore,
}

/// Hedefte ayni isimde dosya varsa `dosya (1).uzanti` formatinda benzersiz yol uretir.
pub fn unique_dest(dir: &Path, name: &str) -> PathBuf {
    unique_dest_with(dir, name, Suffix::Paren)
}

/// Hedefte ayni isimde dosya varsa secilen bicimde numaralandirilmis benzersiz yol uretir.
pub fn unique_dest_with(dir: &Path, name: &str, suffix: Suffix) -> PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }

    let p = Path::new(name);
    let stem = p
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "dosya".to_string());
    let ext = p.extension().map(|s| s.to_string_lossy().into_owned());

    let mut i = 1usize;
    loop {
        let numbered = match suffix {
            Suffix::Paren => format!("{stem} ({i})"),
            Suffix::Underscore => format!("{stem}_{i}"),
        };
        let new_name = match &ext {
            Some(e) => format!("{numbered}.{e}"),
            None => numbered,
        };
        let candidate = dir.join(&new_name);
        if !candidate.exists() {
            return candidate;
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("sfo_naming_{tag}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn numbers_conflicts_in_both_styles_and_never_reuses_a_name() {
        let d = tmp("seq");
        assert_eq!(unique_dest_with(&d, "dosya.pdf", Suffix::Underscore), d.join("dosya.pdf"));
        for n in ["dosya.pdf", "dosya_1.pdf"] {
            fs::write(d.join(n), b"x").unwrap();
        }
        assert_eq!(unique_dest_with(&d, "dosya.pdf", Suffix::Underscore), d.join("dosya_2.pdf"));
        assert_eq!(unique_dest(&d, "dosya.pdf"), d.join("dosya (1).pdf"));

        fs::write(d.join("README"), b"x").unwrap();
        assert_eq!(unique_dest_with(&d, "README", Suffix::Underscore), d.join("README_1"), "uzantisiz ad");
        let _ = fs::remove_dir_all(&d);
    }

    #[cfg(windows)]
    #[test]
    fn case_only_difference_counts_as_a_conflict_on_windows() {
        let d = tmp("case");
        fs::write(d.join("Fatura.PDF"), b"x").unwrap();
        assert_eq!(unique_dest_with(&d, "fatura.pdf", Suffix::Underscore), d.join("fatura_1.pdf"), "NTFS buyuk/kucuk harf ayirmaz");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn sanitize_tag_replaces_forbidden_characters() {
        assert_eq!(sanitize_tag(" Q4: Teklif/Son? "), "Q4_ Teklif_Son_");
        assert_eq!(sanitize_tag(" .. "), "Etiket");
    }
}

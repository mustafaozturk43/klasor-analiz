//! Kural motoru: bir kural, hepsi birlikte saglanmasi gereken kosullar (uzanti, ad, boyut,
//! yas) ve tek bir eylem (klasore tasi, Yil\Ay klasorune tasi, Geri Donusum Kutusu'na gonder).

use crate::models::normalize_exts;
use crate::organize_rules::check_folder_name;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuleAction {
    /// `<kok>\<hedef>`
    Move,
    /// `<kok>\<hedef>\<yil>\<ay>` (dosyanin degisiklik tarihine gore)
    MoveYearMonth,
    /// Windows Geri Donusum Kutusu (yalnizca acik onayla; Gecmis'ten geri alinamaz).
    Trash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub name: String,
    #[serde(default = "yes")]
    pub enabled: bool,
    /// Bos ise uzanti kosulu yok.
    #[serde(default)]
    pub exts: Vec<String>,
    /// Bos ise ad kosulu yok; buyuk/kucuk harf duyarsiz.
    #[serde(default)]
    pub name_contains: String,
    #[serde(default)]
    pub min_mb: Option<f64>,
    #[serde(default)]
    pub max_mb: Option<f64>,
    /// Degistirilmesinden bu yana en az bu kadar gun gecmis olmali.
    #[serde(default)]
    pub min_age_days: Option<u32>,
    pub action: RuleAction,
    /// Secili klasore gore goreli hedef (`Faturalar` ya da `Arsiv\Eski`); `Trash` icin yok sayilir.
    #[serde(default)]
    pub target: String,
}

fn yes() -> bool {
    true
}

/// Bir dosyanin kurallara bakilan ozellikleri.
pub struct Facts<'a> {
    /// Kucuk harf, noktasiz.
    pub ext: &'a str,
    /// Kucuk harf dosya adi (uzantili).
    pub name_lower: &'a str,
    pub size: u64,
    pub age_days: u64,
}

impl Rule {
    /// Kosullarin HEPSI saglaniyorsa true.
    pub fn matches(&self, f: &Facts) -> bool {
        let mb = f.size as f64 / 1_048_576.0;
        (self.exts.is_empty() || self.exts.iter().any(|e| e == f.ext))
            && (self.name_contains.is_empty() || f.name_lower.contains(&self.name_contains))
            && self.min_mb.is_none_or(|m| mb >= m)
            && self.max_mb.is_none_or(|m| mb <= m)
            && self.min_age_days.is_none_or(|d| f.age_days >= u64::from(d))
    }

    /// Temizlenmis, dogrulanmis kopya: her kuralin en az bir kosulu, tasima eylemlerinin gecerli
    /// goreli hedefi olmali (tum dosyalari yanlislikla eslemesin, kok disina cikmasin).
    pub fn validated(&self) -> Result<Rule, String> {
        let mut r = self.clone();
        r.name = r.name.trim().to_string();
        if r.name.is_empty() {
            return Err("Kural adı boş olamaz.".into());
        }
        let label = |msg: &str| format!("“{}”: {msg}", r.name);
        r.exts = normalize_exts(&r.exts);
        r.name_contains = r.name_contains.trim().to_lowercase();
        if r.exts.is_empty() && r.name_contains.is_empty() && r.min_mb.is_none() && r.max_mb.is_none() && r.min_age_days.is_none() {
            return Err(label("en az bir koşul gerekir (uzantı, ad, boyut ya da yaş); koşulsuz kural her dosyayı eşler."));
        }
        if r.min_mb.is_some_and(|m| m < 0.0) || r.max_mb.is_some_and(|m| m < 0.0) {
            return Err(label("boyut eksi olamaz."));
        }
        if let (Some(lo), Some(hi)) = (r.min_mb, r.max_mb) {
            if lo > hi {
                return Err(label("en az boyut, en çok boyuttan büyük olamaz."));
            }
        }
        if r.action == RuleAction::Trash {
            r.target.clear();
            return Ok(r);
        }
        let segs: Vec<String> = r.target.split(['\\', '/']).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
        if segs.is_empty() {
            return Err(label("hedef klasör yazın."));
        }
        let mut clean = Vec::new();
        for seg in &segs {
            if seg == "." || seg == ".." || seg.ends_with(':') {
                return Err(label("hedef, seçili klasörün altında göreli bir yol olmalı."));
            }
            clean.push(check_folder_name("Hedef klasör", seg).map_err(|e| label(&e))?);
        }
        r.target = clean.join("\\");
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(action: RuleAction) -> Rule {
        Rule { name: "Kural".into(), enabled: true, exts: vec![".PDF".into()], name_contains: " Fatura ".into(), min_mb: None, max_mb: None, min_age_days: None, action, target: "Faturalar".into() }
    }
    fn facts<'a>(ext: &'a str, name: &'a str, size: u64, age: u64) -> Facts<'a> {
        Facts { ext, name_lower: name, size, age_days: age }
    }

    #[test]
    fn conditions_are_all_required_and_case_insensitive() {
        let r = rule(RuleAction::Move).validated().unwrap();
        assert_eq!(r.exts, ["pdf"]);
        assert!(r.matches(&facts("pdf", "ocak fatura.pdf", 10, 0)));
        assert!(!r.matches(&facts("pdf", "rapor.pdf", 10, 0)), "ad tutmuyor");
        assert!(!r.matches(&facts("docx", "fatura.docx", 10, 0)), "uzanti tutmuyor");
    }

    #[test]
    fn size_and_age_bounds_are_inclusive() {
        let mut r = rule(RuleAction::Move);
        r.exts.clear();
        r.name_contains.clear();
        (r.min_mb, r.max_mb, r.min_age_days) = (Some(1.0), Some(2.0), Some(30));
        let r = r.validated().unwrap();
        assert!(r.matches(&facts("x", "a", 1_048_576, 30)));
        assert!(r.matches(&facts("x", "a", 2 * 1_048_576, 99)));
        assert!(!r.matches(&facts("x", "a", 1_048_575, 30)), "cok kucuk");
        assert!(!r.matches(&facts("x", "a", 2 * 1_048_576 + 1, 30)), "cok buyuk");
        assert!(!r.matches(&facts("x", "a", 1_048_576, 29)), "cok yeni");
    }

    #[test]
    fn validation_rejects_unsafe_or_empty_rules() {
        let mut r = rule(RuleAction::Move);
        r.exts.clear();
        r.name_contains.clear();
        assert!(r.validated().unwrap_err().contains("en az bir koşul"));
        for bad in ["", "..\\Disari", "C:\\Baska", "a:b", "_Kopyalar_Karantina"] {
            let mut r = rule(RuleAction::Move);
            r.target = bad.into();
            assert!(r.validated().is_err(), "{bad}");
        }
        let mut r = rule(RuleAction::Move);
        (r.min_mb, r.max_mb) = (Some(5.0), Some(1.0));
        assert!(r.validated().is_err());
    }

    #[test]
    fn target_is_normalized_and_trash_ignores_it() {
        let mut r = rule(RuleAction::MoveYearMonth);
        r.target = " Arsiv / Eski ".into();
        assert_eq!(r.validated().unwrap().target, "Arsiv\\Eski");
        let mut t = rule(RuleAction::Trash);
        t.target = "..\\x".into();
        assert_eq!(t.validated().unwrap().target, "");
    }
}

//! "Türüne Göre Düzenle" veri modelleri (frontend'e camelCase aktarilir).

use crate::models::KeepPolicy;
use serde::{Deserialize, Serialize};

/// Bir dosya icin planlanan islem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrganizeAction {
    /// Tur klasorune tasinacak.
    Move,
    /// Zaten dogru klasorde; dokunulmayacak.
    InPlace,
    /// Birebir kopya; karantinaya tasinacak.
    Quarantine,
    /// Tasinmayacak (gecici dosya, yasakli uzanti, uzun yol ...).
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizeEntry {
    pub from: String,
    /// Move/Quarantine icin hedef yol; digerlerinde None.
    pub to: Option<String>,
    pub action: OrganizeAction,
    /// Kategori klasoru (Dokümanlar, Resimler ...); taninmayan turde None.
    pub category: Option<String>,
    /// Tur klasoru (PDF, Excel ...).
    pub kind: Option<String>,
    pub size: u64,
    /// Son degisiklik zamani (unix saniye) — uygulamadan once dogrulanir.
    pub mtime: u64,
    /// Ayni ad hedefte oldugu icin numara eklendi mi?
    pub renamed: bool,
    /// Isim benzerligi bulunan diger dosyanin adi (bilgi amacli).
    pub similar_to: Option<String>,
    /// Kullaniciya gosterilen aciklama (atlama nedeni, kopyanin asli ...).
    pub note: Option<String>,
}

/// Tur klasoru basina ozet (grafik icin).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KindCount {
    pub category: String,
    pub kind: String,
    pub count: u64,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizePlan {
    pub id: String,
    pub root_path: String,
    pub total_files: u64,
    pub entries: Vec<OrganizeEntry>,
    pub kinds: Vec<KindCount>,
    /// Proje isareti tasidigi icin butunuyle korunan klasorler.
    pub protected_dirs: Vec<String>,
    /// Okunamayan klasor/dosya uyarilari.
    pub warnings: Vec<String>,
    /// Uygulamadan sonra bosalan kaynak klasorler kaldirilacak mi?
    pub remove_empty_dirs: bool,
}

/// Kullanicinin duzenleyebildigi tek bir tur klasoru kurali.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleDef {
    pub category: String,
    pub kind: String,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizeOptions {
    /// Bos ise varsayilan kurallar kullanilir.
    #[serde(default)]
    pub rules: Vec<RuleDef>,
    /// Tur klasorunun altinda degisiklik yilina gore klasor (PDF/2026).
    #[serde(default)]
    pub by_year: bool,
    /// Tasima sonrasi bosalan kaynak klasorleri kaldir.
    #[serde(default)]
    pub remove_empty_dirs: bool,
    pub include_subdirs: bool,
    pub quarantine_duplicates: bool,
    pub protect_projects: bool,
    pub include_other: bool,
    /// true: `<root>/PDF`; false: `<root>/Dokümanlar/PDF`.
    pub flat: bool,
    pub keep_policy: KeepPolicy,
    pub exclude_folders: Vec<String>,
    pub banned_extensions: Vec<String>,
    /// 0-1 arasi Jaro-Winkler esigi (benzer ad bilgisi icin).
    pub similarity_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedMove {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizeOutcome {
    pub moved: u64,
    pub moved_bytes: u64,
    pub quarantined: u64,
    pub quarantined_bytes: u64,
    /// Kilitli/izinsiz oldugu icin tasinamayan dosyalar.
    pub failed: Vec<FailedMove>,
    /// Tasima sonrasi kaldirilan bos klasor sayisi.
    pub removed_dirs: u64,
}

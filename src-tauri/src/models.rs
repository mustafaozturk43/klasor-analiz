//! Uygulama genelinde kullanilan veri modelleri (Serde ile frontend'e aktarilir).

use serde::{Deserialize, Serialize};

/// Hem taramada atlanan hem de tasimada kullanilan karantina klasoru adi.
pub const QUARANTINE_DIR: &str = "_Kopyalar_Karantina";

/// Izlenen kaynaklardan gelen dosyalarin gunluk toplandigi klasor (yedek kokunun altinda).
pub const INBOX_DIR: &str = "_GelenKutusu";

/// 500 MB esik degeri (byte cinsinden).
pub const LARGE_FILE_THRESHOLD: u64 = 500 * 1024 * 1024;

/// Varsayilan Jaro-Winkler esigi.
pub const DEFAULT_SIMILARITY_THRESHOLD: f64 = 0.82;

/// Sistem/calistirilabilir dosya uzantilari — varsayilan yasakli liste.
/// Taranan kategori listesini gecersiz kilar (orn. csv yasaklanirsa CSV taranmaz).
pub const DEFAULT_BANNED_EXTS: &[&str] = &[
    "exe", "com", "dll", "sys", "msi", "bat", "cmd", "scr", "ps1", "vbs", "lnk", "reg",
];

/// Uzanti listesini normalize eder: kucuk harf, basindaki nokta ve bosluk atilir.
pub fn normalize_exts(list: &[String]) -> Vec<String> {
    list.iter()
        .map(|s| s.trim().trim_start_matches('.').to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Taramanin kullanici tarafindan durduruldugunu belirten hata mesaji (sentinel).
pub const CANCEL_MSG: &str = "TARAMA_DURDURULDU";

/// Hash/tarama ilerleme bildirimi: (tamamlanan, toplam).
pub type ProgressFn<'a> = &'a (dyn Fn(u64, u64) + Sync + 'a);

/// Iptal bayragi sorgusu.
pub type CancelFn<'a> = &'a (dyn Fn() -> bool + Sync + 'a);

/// Exact gruplarda hangi dosyanin tutulacagina karar veren politika.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum KeepPolicy {
    /// Taranan ilk dosya (tanimli davranis).
    #[default]
    First,
    /// En yeni degisiklik tarihine sahip dosya.
    Newest,
    /// En eski degisiklik tarihine sahip dosya.
    Oldest,
    /// En kisa yola sahip dosya.
    ShortestPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileCategory {
    Document,
    Image,
    Archive,
    /// Tarama kategorilerinin disindaki dosyalar (FAZ 2: "Diğer dosyaları da tara").
    Other,
}

/// Tek bir dosyanin taranmis hali.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileItem {
    /// Tam dosya yolu.
    pub path: String,
    /// Uzanti dahil dosya adi.
    pub name: String,
    /// Uzantisiz dosya adi (isim benzerligi icin).
    pub stem: String,
    /// Noktasiz, kucuk harfe cevrilmis uzanti.
    pub extension: String,
    /// Byte cinsinden boyut.
    pub size: u64,
    /// Son degisiklik zamani (unix saniye) — onbellek gecerliligi icin.
    pub mtime: u64,
    pub category: FileCategory,
    /// BLAKE3 tabanli kimlik (grup anahtari).
    pub hash_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupType {
    Exact,
    Similar,
}

/// Bir dosya grubu (birebir kopya veya isim benzeri).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileGroup {
    /// GRP-XXXX formatinda grup kimligi.
    pub id: String,
    pub group_type: GroupType,
    pub category: FileCategory,
    pub files: Vec<FileItem>,
    /// Exact gruplarda kazanilacak alan (kopyalar disarda tutuldugunda).
    pub wasted_bytes: u64,
}

/// Tek bir tasima kaydi (from -> to).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveRecord {
    pub from: String,
    pub to: String,
}

/// Etiketleme ve tasima isleminin sonucu.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveResult {
    pub tag: String,
    pub moved_to_tag: Vec<MoveRecord>,
    pub moved_to_quarantine: Vec<MoveRecord>,
    /// Tam hash dogrulamasi basarisiz oldugu icin tasima yerine birakilan
    /// dosyalarin yollari (F3 opt-in).
    pub skipped: Vec<String>,
}

/// Geri alma (undo) icin saklanan bir islem kaydi.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoRecord {
    /// Kayit kimligi (MV-<unix_ms>).
    pub id: String,
    /// Islem zamani (unix ms).
    pub timestamp: u64,
    /// Islemde kullanilan etiket adi.
    pub tag: String,
    pub moves: Vec<MoveRecord>,
}

/// Tarama ve gruplama sonucu (frontend'e gonderilir).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub root_path: String,
    pub total_files: usize,
    pub groups: Vec<FileGroup>,
    pub comparison: Option<ComparisonDetails>,
    /// Okunamadigi icin atlanan klasor/dosyalar (tarama bunlar yuzunden durmaz).
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparedFile {
    pub file: FolderFile,
    pub source: String,
    pub group_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonDetails {
    pub root_b: String,
    pub files: Vec<ComparedFile>,
}

/// Canli tarama ilerleme olayi (Channel ile gonderilir).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    /// "scanning" | "hashing" | "done"
    pub phase: String,
    pub files_done: u64,
    pub total_files: u64,
}

/// Bir klasordeki tek bir dosyanin hafif ozeti (dosya listeleme icin).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderFile {
    pub name: String,
    pub path: String,
    pub size: u64,
    /// Hedef kategorilerden biri; degilse None (diger dosyalar).
    pub category: Option<FileCategory>,
    pub created_at: Option<u64>,
    /// Son degisiklik zamani (unix saniye).
    pub modified_at: Option<u64>,
}

/// Bir alt klasorun boyut ozeti.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderStat {
    pub name: String,
    pub path: String,
    pub total_bytes: u64,
    pub file_count: u64,
    pub document_bytes: u64,
    pub image_bytes: u64,
    pub archive_bytes: u64,
    pub other_bytes: u64,
    /// Bu klasordeki direkt dosyalar (boyut sirali).
    pub files: Vec<FolderFile>,
}

/// Ana klasor + alt klasorlerin boyut ozeti (grafik icin).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderStatsResult {
    pub root_path: String,
    /// Ana klasorun toplam boyutu (alt klasorler dahil).
    pub root_total_bytes: u64,
    pub root_file_count: u64,
    pub folders: Vec<FolderStat>,
    pub category_bytes: [u64; 4],
    pub warnings: Vec<String>,
}

//! Bulut yer tutuculari (OneDrive "yalnizca cevrimici" dosyalar). Bu dosyalarin icerigi
//! diskte degildir; okumak (hash almak) dosyayi indirir. Tarama onlari atlar ve bildirir.

use std::os::windows::fs::MetadataExt;
use std::path::Path;

const OFFLINE: u32 = 0x0000_1000;
const RECALL_ON_OPEN: u32 = 0x0004_0000;
const RECALL_ON_DATA_ACCESS: u32 = 0x0040_0000;

/// Dosya ozniteligi "icerik okununca indirilir" diyor mu?
pub fn is_placeholder(attributes: u32) -> bool {
    attributes & (OFFLINE | RECALL_ON_OPEN | RECALL_ON_DATA_ACCESS) != 0
}

/// Dosya yalnizca cevrimici mi? Oznitelik okunamazsa false (normal dosya gibi islenir).
pub fn is_online_only(path: &Path) -> bool {
    std::fs::symlink_metadata(path).map(|m| is_placeholder(m.file_attributes())).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_recall_attributes_but_not_ordinary_files() {
        assert!(is_placeholder(0x0040_0020), "arsiv + recall-on-data-access");
        assert!(is_placeholder(0x1000));
        assert!(!is_placeholder(0x20), "siradan arsiv ozniteligi");
        assert!(!is_online_only(&std::env::temp_dir()));
    }
}

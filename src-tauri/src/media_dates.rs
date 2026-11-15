//! Fotograf ve videolarin cekim tarihi (Windows ozellik sistemi): once
//! `System.Photo.DateTaken`, yoksa `System.Media.DateEncoded`. Deger UTC'dir.

use crate::shell_thumb::{shell_item, ComGuard};
use std::path::Path;
use windows::core::GUID;
use windows::Win32::Foundation::{FILETIME, PROPERTYKEY};
use windows::Win32::UI::Shell::IShellItem2;

const PKEY_PHOTO_DATE_TAKEN: PROPERTYKEY = PROPERTYKEY { fmtid: GUID::from_u128(0x14b81da1_0135_4d31_96d9_6cbfc9671a99), pid: 36867 };
const PKEY_MEDIA_DATE_ENCODED: PROPERTYKEY = PROPERTYKEY { fmtid: GUID::from_u128(0x2e4b640d_5019_46d8_8881_55414cc5caa0), pid: 100 };
/// 1601-01-01 ile 1970-01-01 arasi, 100 ns biriminde.
const EPOCH_DIFF: u64 = 116_444_736_000_000_000;

fn to_unix(ft: FILETIME) -> Option<u64> {
    let t = (u64::from(ft.dwHighDateTime) << 32) | u64::from(ft.dwLowDateTime);
    (t > EPOCH_DIFF).then(|| (t - EPOCH_DIFF) / 10_000_000)
}

/// Cekim tarihi (unix saniye, UTC); yoksa `None`.
pub fn date_taken(path: &Path) -> Option<u64> {
    let _com = ComGuard::new();
    let item: IShellItem2 = shell_item(path)?;
    [PKEY_PHOTO_DATE_TAKEN, PKEY_MEDIA_DATE_ENCODED].iter().find_map(|k| unsafe { item.GetFileTime(k) }.ok().and_then(to_unix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_without_exif_has_no_date_taken() {
        let d = std::env::temp_dir().join(format!("sfo_mdate_{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let img = d.join("a.png");
        image::RgbImage::new(8, 8).save(&img).unwrap();
        assert_eq!(date_taken(&img), None);
        let _ = std::fs::remove_dir_all(&d);
    }
}

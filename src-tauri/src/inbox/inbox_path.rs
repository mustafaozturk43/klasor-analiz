//! Gelen kutusu yollari: `<yedek>\_GelenKutusu\<YYYY-MM-DD>_Bakilacaklar\<Grup>\`.
//! Gun YEREL tarihle belirlenir; UTC kullanilsaydi gece yarisindan sonra gelen
//! dosyalar (Turkiye UTC+3) bir onceki gunun klasorune duserdi.

use crate::models::INBOX_DIR;
use chrono::{Local, NaiveDate};
use std::path::{Path, PathBuf};

pub const DAY_SUFFIX: &str = "_Bakilacaklar";

pub fn day_folder_name(date: NaiveDate) -> String {
    format!("{}{DAY_SUFFIX}", date.format("%Y-%m-%d"))
}

pub fn day_dir(backup_root: &Path, date: NaiveDate) -> PathBuf {
    backup_root.join(INBOX_DIR).join(day_folder_name(date))
}

/// Bugunun (yerel saat) gelen kutusu klasoru.
pub fn today_dir(backup_root: &Path) -> PathBuf {
    day_dir(backup_root, Local::now().date_naive())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};

    #[test]
    fn day_folder_uses_local_calendar_day() {
        let d = NaiveDate::from_ymd_opt(2026, 11, 12).unwrap();
        assert_eq!(day_folder_name(d), "2026-11-12_Bakilacaklar");
        assert_eq!(day_dir(Path::new(r"D:\Yedek"), d), PathBuf::from(r"D:\Yedek\_GelenKutusu\2026-11-12_Bakilacaklar"));

        // 12 Kasim 00:30 Istanbul = 11 Kasim 21:30 UTC: klasor 12 Kasim olmali.
        let ist = FixedOffset::east_opt(3 * 3600).unwrap();
        let t = ist.with_ymd_and_hms(2026, 11, 12, 0, 30, 0).unwrap();
        assert_eq!(day_folder_name(t.date_naive()), "2026-11-12_Bakilacaklar");
        assert_eq!(day_folder_name(t.naive_utc().date()), "2026-11-11_Bakilacaklar", "UTC bir onceki gune duserdi");
    }
}

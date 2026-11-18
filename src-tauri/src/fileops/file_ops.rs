//! No-clobber transfer, including cross-volume moves. Source is removed only after success.
use std::fs::{self, OpenOptions};
use std::path::Path;

pub fn transfer(from: &Path, to: &Path) -> Result<(), String> {
    if to.exists() { return Err(format!("Hedef zaten var: {}", to.display())); }
    // Hard link ayni birimde tarihleri oldugu gibi korur; desteklenmiyorsa (farkli
    // disk, FAT/exFAT) kopyalanir ve tarihler kaynaktan aktarilir.
    if fs::hard_link(from, to).is_err() {
        copy_preserving(from, to)?;
    }
    if let Err(e) = fs::remove_file(from) {
        let _ = fs::remove_file(to);
        return Err(e.to_string());
    }
    Ok(())
}

/// Hedefi yeni dosya olarak olusturup icerigi kopyalar (uzerine yazmaz) ve kaynagin
/// degisiklik, erisim ve (Windows'ta) olusturulma tarihlerini hedefe aktarir. Tarihli
/// adlandirma, "en yeni/en eski" saklama politikasi ve yil klasorleri bu tarihlere dayanir.
pub fn copy_preserving(from: &Path, to: &Path) -> Result<(), String> {
    let mut input = fs::File::open(from).map_err(|e| e.to_string())?;
    let stamps = input.metadata().ok();
    let mut output = OpenOptions::new().write(true).create_new(true).open(to).map_err(|e| e.to_string())?;
    let copied = std::io::copy(&mut input, &mut output).and_then(|_| {
        if let Some(meta) = &stamps {
            // Tarih aktarimi desteklenmeyen dosya sistemlerinde tasima yine de tamamlanir.
            let _ = output.set_times(times_of(meta));
        }
        output.sync_all()
    });
    if let Err(e) = copied {
        drop(output);
        let _ = fs::remove_file(to);
        return Err(e.to_string());
    }
    Ok(())
}

fn times_of(meta: &fs::Metadata) -> fs::FileTimes {
    let mut times = fs::FileTimes::new();
    if let Ok(t) = meta.modified() {
        times = times.set_modified(t);
    }
    if let Ok(t) = meta.accessed() {
        times = times.set_accessed(t);
    }
    #[cfg(windows)]
    if let Ok(t) = meta.created() {
        use std::os::windows::fs::FileTimesExt;
        times = times.set_created(t);
    }
    times
}

/// Yerel saatle takvim gunu (`gg-aa-yyyy`): gelen kutusu gun klasoruyle ayni saat dilimi.
/// Gece yarisindan hemen sonra olusturulan dosya bir onceki gunun adini almaz.
pub fn local_date_suffix(seconds: u64) -> String {
    use chrono::{Local, Offset, TimeZone};
    let offset = Local.timestamp_opt(seconds as i64, 0).single().map(|d| d.offset().fix().local_minus_utc()).unwrap_or(0);
    date_suffix((seconds as i64 + offset as i64).max(0) as u64)
}

/// Calendar date (UTC) from Unix seconds. No fallback to modified time.
pub fn date_suffix(seconds: u64) -> String {
    let z = (seconds / 86400) as i64 + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };
    format!("{day:02}-{month:02}-{year:04}")
}

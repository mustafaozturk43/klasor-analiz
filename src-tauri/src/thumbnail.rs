//! Gelen kutusu kucuk resimleri. 2 MB'tan kucuk gorseller oldugu gibi gosterilir;
//! buyukleri 96 px'lik PNG'ye kucultulup uygulama onbelleginde saklanir. Onbellek
//! anahtari yol + boyut + mtime'dir: dosya degisirse yeni kucuk resim uretilir.

use crate::inspector::FileDetail;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub const DIRECT_LIMIT: u64 = 2 * 1024 * 1024;
pub const SIZE_PX: u32 = 96;
/// Decode edilecek en buyuk dosya (bellegi korumak icin).
const MAX_DECODE: u64 = 80 * 1024 * 1024;
const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "bmp", "tif", "tiff"];

pub fn is_image(ext: &str) -> bool {
    IMAGE_EXTS.contains(&ext)
}

fn cache_key(f: &FileDetail) -> String {
    let raw = format!("{}|{}|{}", f.path.to_lowercase(), f.size, f.modified_ms.unwrap_or(0));
    blake3::hash(raw.as_bytes()).to_hex()[..24].to_string()
}

/// Buyuk gorselden kucuk resim uretir (yoksa); onbellekteki yolu dondurur.
pub fn cached_thumbnail(cache_dir: &Path, f: &FileDetail) -> Option<PathBuf> {
    let out = cache_dir.join(format!("{}.png", cache_key(f)));
    if out.exists() {
        return Some(out);
    }
    if f.size > MAX_DECODE {
        return None;
    }
    let img = image::open(&f.path).ok()?;
    let thumb = img.thumbnail(SIZE_PX, SIZE_PX);
    std::fs::create_dir_all(cache_dir).ok()?;
    let tmp = out.with_extension("tmp.png");
    thumb.save_with_format(&tmp, image::ImageFormat::Png).ok()?;
    std::fs::rename(&tmp, &out).ok()?;
    Some(out)
}

/// Onbellekte bu sureden eski kucuk resimler silinir (dosyasi coktan tasinmis olanlar).
pub const MAX_CACHE_AGE: std::time::Duration = std::time::Duration::from_secs(30 * 24 * 3600);

/// Eski kucuk resimleri onbellekten kaldirir; silinen adedi dondurur. Yalnizca
/// uygulamanin kendi urettigi `.png` dosyalarina dokunur; gerekirse yeniden uretilir.
pub fn prune_cache(cache_dir: &Path, max_age: std::time::Duration) -> u64 {
    let Ok(rd) = std::fs::read_dir(cache_dir) else { return 0 };
    rd.flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "png"))
        .filter(|e| e.metadata().ok().and_then(|m| m.modified().ok()).and_then(|t| t.elapsed().ok()).is_some_and(|age| age > max_age))
        .filter(|e| std::fs::remove_file(e.path()).is_ok())
        .count() as u64
}

/// Gorsel dosyalarin `thumbnail_path` alanini paralel doldurur.
pub fn fill(files: &mut [FileDetail], cache_dir: &Path) {
    files.par_iter_mut().filter(|f| is_image(&f.ext)).for_each(|f| {
        f.thumbnail_path = if f.size < DIRECT_LIMIT {
            Some(f.path.clone())
        } else {
            cached_thumbnail(cache_dir, f).map(|p| p.to_string_lossy().into_owned())
        };
    });
}

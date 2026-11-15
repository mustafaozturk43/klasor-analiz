//! Windows Shell kucuk resimleri (Dosya Gezgini ile ayni kaynak): fotograf, video
//! karesi, PDF/Office sayfasi; on izleme yoksa Windows uzanti ikonunu dondurur.
//! PNG olarak uygulama onbellegine yazilir; anahtar yol + boyut + mtime + piksel.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use windows::core::HSTRING;
use windows::Win32::Foundation::SIZE;
use windows::Win32::Graphics::Gdi::{DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, IBindCtx, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Shell::{IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_RESIZETOFIT};

/// Bu is parcaciginda COM'u baslatir; yalnizca kendisi baslattiysa kapatir.
pub(crate) struct ComGuard(bool);

impl ComGuard {
    pub(crate) fn new() -> Self {
        ComGuard(unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.is_ok())
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() };
        }
    }
}

pub(crate) fn shell_item<T: windows::core::Interface>(path: &Path) -> Option<T> {
    unsafe { SHCreateItemFromParsingName(&HSTRING::from(path.as_os_str()), None::<&IBindCtx>) }.ok()
}

fn cache_key(path: &Path, size: u32) -> Option<String> {
    let meta = std::fs::metadata(path).ok().filter(|m| m.is_file())?;
    let mtime = meta.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
    let raw = format!("{}|{}|{}|{}", path.to_string_lossy().to_lowercase(), meta.len(), mtime, size);
    Some(blake3::hash(raw.as_bytes()).to_hex()[..24].to_string())
}

/// HBITMAP'i ustten-alta 32 bit okuyup RGBA'ya cevirir (alfa hep 0 ise opak yapar).
fn bitmap_to_rgba(hbmp: HBITMAP) -> Option<image::RgbaImage> {
    let mut bm = BITMAP::default();
    let got = unsafe { GetObjectW(hbmp.into(), std::mem::size_of::<BITMAP>() as i32, Some(&mut bm as *mut BITMAP as *mut _)) };
    if got == 0 || bm.bmWidth <= 0 || bm.bmHeight <= 0 {
        return None;
    }
    let (w, h) = (bm.bmWidth, bm.bmHeight);
    let header = BITMAPINFOHEADER { biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32, biWidth: w, biHeight: -h, biPlanes: 1, biBitCount: 32, biCompression: BI_RGB.0, ..Default::default() };
    let mut info = BITMAPINFO { bmiHeader: header, ..Default::default() };
    let mut buf = vec![0u8; (w as usize) * (h as usize) * 4];
    let lines = unsafe {
        let dc = GetDC(None);
        let n = GetDIBits(dc, hbmp, 0, h as u32, Some(buf.as_mut_ptr() as *mut _), &mut info, DIB_RGB_COLORS);
        ReleaseDC(None, dc);
        n
    };
    if lines == 0 {
        return None;
    }
    let opaque = buf.chunks_exact(4).all(|p| p[3] == 0);
    for px in buf.chunks_exact_mut(4) {
        px.swap(0, 2);
        if opaque {
            px[3] = 255;
        }
    }
    image::RgbaImage::from_raw(w as u32, h as u32, buf)
}

fn render(path: &Path, size: u32) -> Option<image::RgbaImage> {
    let _com = ComGuard::new();
    let factory: IShellItemImageFactory = shell_item(path)?;
    let hbmp = unsafe { factory.GetImage(SIZE { cx: size as i32, cy: size as i32 }, SIIGBF_RESIZETOFIT) }.ok()?;
    let img = bitmap_to_rgba(hbmp);
    unsafe {
        let _ = DeleteObject(hbmp.into());
    }
    img
}

/// Dosyanin kucuk resmini (yoksa ikonunu) onbellege PNG olarak yazar; yolunu dondurur.
pub fn shell_thumbnail(path: &Path, size: u32, cache_dir: &Path) -> Option<PathBuf> {
    let size = size.clamp(32, 1024);
    let out = cache_dir.join(format!("{}.png", cache_key(path, size)?));
    if out.exists() {
        return Some(out);
    }
    let img = render(path, size)?;
    std::fs::create_dir_all(cache_dir).ok()?;
    let tmp = out.with_extension("tmp.png");
    img.save_with_format(&tmp, image::ImageFormat::Png).ok()?;
    std::fs::rename(&tmp, &out).ok()?;
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn dir(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("sfo_shthumb_{}_{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn thumbnail_for_png_is_written_as_png() {
        let d = dir("png");
        let img = d.join("foto.png");
        image::RgbImage::from_pixel(64, 48, image::Rgb([200, 40, 40])).save(&img).unwrap();
        let out = shell_thumbnail(&img, 96, &d.join("cache")).expect("kucuk resim olmali");
        assert!(fs::read(&out).unwrap().starts_with(b"\x89PNG"));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn thumbnail_for_text_file_falls_back_to_icon() {
        let d = dir("txt");
        let f = d.join("not.txt");
        fs::write(&f, b"merhaba").unwrap();
        assert!(shell_thumbnail(&f, 96, &d.join("cache")).is_some());
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn thumbnail_for_missing_file_is_none() {
        let d = dir("missing");
        assert!(shell_thumbnail(&d.join("yok.jpg"), 96, &d.join("cache")).is_none());
        let _ = fs::remove_dir_all(&d);
    }
}

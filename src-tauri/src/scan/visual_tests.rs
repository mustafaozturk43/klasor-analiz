use super::*;
use image::{imageops, Rgb, RgbImage};
use std::fs;
use std::path::PathBuf;

fn scratch(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("sfo_visual_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

/// Soldan saga artan renk gecisi: dHash'i (komsu piksel karsilastirmasi) belirgin.
fn gradient(w: u32, h: u32) -> RgbImage {
    RgbImage::from_fn(w, h, |x, y| Rgb([(x * 255 / w) as u8, (y * 255 / h) as u8, 90]))
}

/// Parlak/koyu seritli desen: gradyandan cok farkli bir parmak izi.
fn stripes(w: u32, h: u32) -> RgbImage {
    RgbImage::from_fn(w, h, |x, _| if (x / (w / 9).max(1)) % 2 == 0 { Rgb([240, 240, 240]) } else { Rgb([10, 10, 10]) })
}

fn run(root: &Path, dist: u32) -> VisualReport {
    build(root, true, &[], dist, &VisualState::default(), &|_, _| {}).unwrap()
}

#[test]
fn resized_copy_groups_with_original_but_different_image_does_not() {
    let root = scratch("group");
    gradient(256, 256).save(root.join("orijinal.png")).unwrap();
    imageops::resize(&gradient(256, 256), 80, 80, FilterType::Lanczos3).save(root.join("kucuk.png")).unwrap();
    stripes(256, 256).save(root.join("baska.png")).unwrap();
    let r = run(&root, 6);
    assert_eq!((r.scanned, r.unreadable, r.groups.len()), (3, 0, 1));
    let names: Vec<String> = r.groups[0].files.iter().map(|f| Path::new(&f.path).file_name().unwrap().to_string_lossy().into_owned()).collect();
    assert_eq!(names, ["orijinal.png", "kucuk.png"], "en yuksek cozunurluk basta");
    assert_eq!(r.groups[0].files[0].distance, 0);
    assert!(r.groups[0].files[1].distance <= 6);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn threshold_zero_only_matches_identical_fingerprints() {
    let root = scratch("zero");
    let base = gradient(200, 200);
    base.save(root.join("a.png")).unwrap();
    base.save(root.join("a_kopya.png")).unwrap();
    let mut slight = base.clone();
    for x in 0..200 {
        slight.put_pixel(x, 100, Rgb([255, 255, 255]));
    }
    slight.save(root.join("a_cizgili.png")).unwrap();
    let r = run(&root, 0);
    assert_eq!(r.groups.len(), 1);
    assert!(r.groups[0].files.len() >= 2 && r.groups[0].files.iter().all(|f| f.distance == 0));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn unreadable_images_are_counted_and_quarantine_is_skipped() {
    let root = scratch("skip");
    gradient(64, 64).save(root.join("iyi.png")).unwrap();
    fs::write(root.join("bozuk.jpg"), b"bu bir resim degil").unwrap();
    fs::create_dir_all(root.join(QUARANTINE_DIR)).unwrap();
    gradient(64, 64).save(root.join(QUARANTINE_DIR).join("karantinada.png")).unwrap();
    let r = run(&root, 6);
    assert_eq!((r.scanned, r.unreadable, r.groups.len()), (2, 1, 0), "karantina taranmaz, bozuk dosya sayilir");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn cache_avoids_decoding_unchanged_files_twice() {
    let root = scratch("cache");
    gradient(64, 64).save(root.join("a.png")).unwrap();
    let state = VisualState::default();
    build(&root, true, &[], 6, &state, &|_, _| {}).unwrap();
    assert_eq!(state.cache.lock().unwrap().len(), 1);
    // Dosya bozulsa bile (boyut/tarih ayni kalirsa) onbellekten gelir; degisirse yeniden cozulur.
    fs::write(root.join("a.png"), b"degisti").unwrap();
    let r = build(&root, true, &[], 6, &state, &|_, _| {}).unwrap();
    assert_eq!(r.unreadable, 1, "boyut degisti: yeniden cozuldu ve basarisiz");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn dhash_is_stable_for_the_same_image_and_distance_is_symmetric() {
    let a = DynamicImage::ImageRgb8(gradient(100, 100));
    let b = DynamicImage::ImageRgb8(stripes(100, 100));
    assert_eq!(dhash(&a), dhash(&a));
    assert_eq!((dhash(&a) ^ dhash(&b)).count_ones(), (dhash(&b) ^ dhash(&a)).count_ones());
    assert!((dhash(&a) ^ dhash(&b)).count_ones() > 12, "farkli gorseller uzak olmali");
}

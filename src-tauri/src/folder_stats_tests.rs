use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(std::path::PathBuf);
impl Temp {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("sfo_partial_stats_{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir_all(&root).unwrap(); Self(root)
    }
}
impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

#[test]
fn unreadable_file_does_not_discard_readable_totals_or_following_files() {
    let temp = Temp::new();
    let first = temp.0.join("first.pdf");
    let last = temp.0.join("last.jpg");
    fs::write(&first, b"1234").unwrap();
    fs::write(&last, b"567").unwrap();
    let (mut stat, mut warnings) = folder_stat(&temp.0);
    assert_eq!(stat.total_bytes, 7);
    let missing = temp.0.join("cloud-only.txt");
    accumulate_file(&mut stat, &missing, Err("Bulut sağlayıcısı çalışmıyor (os error 362)".into()), true, &mut warnings);
    accumulate_file(&mut stat, &last, fs::metadata(&last).map_err(|e| e.to_string()), false, &mut warnings);
    assert_eq!(stat.total_bytes, 10);
    assert_eq!(stat.document_bytes, 4);
    assert_eq!(stat.image_bytes, 6);
    assert_eq!(stat.file_count, 3);
    assert_eq!(stat.files.len(), 2);
    let messages = warnings.finish();
    assert!(messages[0].contains("Analiz kısmi"));
    assert!(messages[1].contains("cloud-only.txt"));
    assert!(messages[1].contains("os error 362"));
}

#[test]
fn inaccessible_directory_returns_zero_stat_with_actual_error_not_none() {
    let temp = Temp::new();
    let missing = temp.0.join("missing");
    let (stat, warnings) = folder_stat(&missing);
    assert_eq!(stat.name, "missing");
    assert_eq!(stat.total_bytes, 0);
    let messages = warnings.finish();
    assert!(!messages.is_empty());
    assert!(messages.iter().any(|m| m.contains("missing")));
}

#[test]
fn clean_analysis_does_not_emit_partial_warning() {
    let temp = Temp::new();
    fs::create_dir(temp.0.join("EkÖdeme-Veri")).unwrap();
    fs::write(temp.0.join("EkÖdeme-Veri/test.pdf"), b"1234").unwrap();
    let result = collect(&temp.0).unwrap();
    assert_eq!(result.root_total_bytes, 4);
    assert_eq!(result.folders[0].name, "EkÖdeme-Veri");
    assert!(result.warnings.is_empty());
}

#[test]
fn warning_details_are_bounded_but_total_count_is_preserved() {
    let mut warnings = ReadWarnings::default();
    for _ in 0..70 { warnings.push(Path::new("cloud.txt"), "access denied"); }
    let mut other = ReadWarnings::default();
    for _ in 0..20 { other.push(Path::new("other.txt"), "not found"); }
    warnings.merge(other);
    let result = warnings.finish();
    assert_eq!(result.len(), 52);
    assert!(result[0].contains("90 okuma hatası"));
}

#[test]
fn analysis_inside_single_worker_rayon_pool_does_not_starve_walker() {
    let temp = Temp::new();
    for i in 0..8 {
        let dir = temp.0.join(format!("folder-{i}/nested"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("data.pdf"), b"1234").unwrap();
    }
    let pool = rayon::ThreadPoolBuilder::new().num_threads(1).build().unwrap();
    let result = pool.install(|| collect(&temp.0)).unwrap();
    assert_eq!(result.root_total_bytes, 32);
    assert_eq!(result.root_file_count, 8);
    assert_eq!(result.folders.len(), 8);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
}

#[test]
fn concurrent_analysis_and_scanning_use_independent_walking_pools() {
    let temp = Temp::new();
    for i in 0..6 {
        let dir = temp.0.join(format!("folder-{i}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("data.pdf"), b"1234").unwrap();
    }
    let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
    let (analysis, scan) = pool.install(|| rayon::join(
        || collect(&temp.0),
        || scanner::scan(&temp.0, true, &[], &[], false, &|_| {}, &|| false),
    ));
    let analysis = analysis.unwrap();
    assert_eq!(analysis.root_file_count, 6);
    assert!(analysis.warnings.is_empty(), "{:?}", analysis.warnings);
    assert_eq!(scan.unwrap().len(), 6);
}

//! Cekirdek is mantigi testleri: tarama filtreleri, iptal (durdurma),
//! gruplama (exact + benzerlik esigi) ve klasor boyut istatistikleri.

use smart_file_organizer_lib::models::{FileCategory, GroupType, CANCEL_MSG, QUARANTINE_DIR};
use smart_file_organizer_lib::{folder_stats, grouper, hash_cache::HashCache, scanner};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// Test basina benzersiz gecici klasor olusturur.
fn temp_root(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("sfo_test_{}_{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

/// Ornek agac:
/// root/a.pdf + root/b.pdf (ayni icerik -> Exact),
/// root/alt/c.pdf (tekil),
/// root/_Kopyalar_Karantina/d.pdf (atlanmali),
/// root/node_modules/e.pdf (haric tutulunca atlanmali).
fn build_tree(name: &str) -> PathBuf {
    let root = temp_root(name);
    fs::write(root.join("a.pdf"), b"AYNI-ICERIK").unwrap();
    fs::write(root.join("b.pdf"), b"AYNI-ICERIK").unwrap();

    let sub = root.join("alt");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("c.pdf"), b"BASKA-ICERIK").unwrap();

    let q = root.join(QUARANTINE_DIR);
    fs::create_dir_all(&q).unwrap();
    fs::write(q.join("d.pdf"), b"KARANTINA").unwrap();

    let nm = root.join("node_modules");
    fs::create_dir_all(&nm).unwrap();
    fs::write(nm.join("e.pdf"), b"HARIC").unwrap();

    root
}

fn scan(root: &Path, include_subdirs: bool, excludes: &[String], include_other: bool) -> Vec<String> {
    scanner::scan(root, include_subdirs, excludes, &[], include_other, &|_| {}, &|| false)
        .unwrap()
        .into_iter()
        .map(|f| f.name)
        .collect()
}

#[test]
fn scan_skips_quarantine_and_excluded_folders() {
    let root = build_tree("filters");
    let names = scan(&root, true, &["node_modules".to_string()], false);
    assert_eq!(names.len(), 3, "karantina + node_modules atlanmali: {names:?}");
    assert!(names.contains(&"a.pdf".to_string()));
    assert!(names.contains(&"b.pdf".to_string()));
    assert!(names.contains(&"c.pdf".to_string()));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn scan_without_subdirs_only_reads_root_files() {
    let root = build_tree("depth");
    let names = scan(&root, false, &["node_modules".to_string()], false);
    assert_eq!(names.len(), 2, "sadece kok dosyalar: {names:?}");
    assert!(!names.contains(&"c.pdf".to_string()));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn scan_aborts_when_cancel_flag_set() {
    let root = build_tree("cancel");
    let cancel = AtomicBool::new(true);
    let is_cancelled = || cancel.load(Ordering::Relaxed);
    let err = scanner::scan(&root, true, &[], &[], false, &|_| {}, &is_cancelled).unwrap_err();
    assert_eq!(err, CANCEL_MSG);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn group_finds_exact_and_similar_groups() {
    let root = build_tree("grouping");
    // Ayni kategori, farkli icerik, cok benzer isimler -> Similar.
    fs::write(root.join("rapor_v1.txt"), b"v1 icerik").unwrap();
    fs::write(root.join("rapor_v2.txt"), b"v2 baska icerik").unwrap();

    let files = scanner::scan(
        &root,
        true,
        &["node_modules".to_string()],
        &[],
        false,
        &|_| {},
        &|| false,
    )
    .unwrap();

    let groups = grouper::group(files, 0.82, &HashCache::new(), &|_, _| {}, &|| false).unwrap();

    let exact = groups
        .iter()
        .find(|g| matches!(g.group_type, GroupType::Exact))
        .expect("a.pdf/b.pdf icin Exact grup bekleniyor");
    assert_eq!(exact.files.len(), 2);
    assert!(exact.wasted_bytes > 0);
    assert!(exact.id.starts_with("GRP-"));

    let similar = groups
        .iter()
        .find(|g| matches!(g.group_type, GroupType::Similar))
        .expect("rapor_v1/v2 icin Similar grup bekleniyor");
    assert!(similar.files.iter().any(|f| f.name == "rapor_v1.txt"));
    assert!(similar.files.iter().any(|f| f.name == "rapor_v2.txt"));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn similarity_threshold_is_respected() {
    let root = build_tree("threshold");
    fs::write(root.join("rapor_v1.txt"), b"v1 icerik").unwrap();
    fs::write(root.join("rapor_v2.txt"), b"v2 baska icerik").unwrap();

    let files = scanner::scan(
        &root,
        true,
        &["node_modules".to_string()],
        &[],
        false,
        &|_| {},
        &|| false,
    )
    .unwrap();

    // Cok yuksek esik: isim benzerligi grubu olusmamali (Exact hala var).
    let groups = grouper::group(files, 0.999, &HashCache::new(), &|_, _| {}, &|| false).unwrap();
    assert!(
        !groups
            .iter()
            .any(|g| matches!(g.group_type, GroupType::Similar)),
        "0.999 esiginde Similar grup beklenmiyor"
    );
    assert!(groups
        .iter()
        .any(|g| matches!(g.group_type, GroupType::Exact)));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn group_aborts_when_cancelled() {
    let root = build_tree("group_cancel");
    let files = scanner::scan(&root, true, &[], &[], false, &|_| {}, &|| false).unwrap();
    let groups = grouper::group(files, 0.82, &HashCache::new(), &|_, _| {}, &|| true).unwrap_err();
    assert_eq!(groups, CANCEL_MSG);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn folder_stats_sums_root_and_subfolders() {
    let root = build_tree("stats");
    let stats = folder_stats::collect(&root).unwrap();

    // alt + _Kopyalar_Karantina + node_modules = 3 alt klasor
    assert_eq!(stats.folders.len(), 3);

    // Kok: a.pdf(11) + b.pdf(11); alt: c.pdf(12); karantina: d(9); node_modules: e(5)
    assert_eq!(stats.root_total_bytes, 11 + 11 + 12 + 9 + 5);
    assert_eq!(stats.root_file_count, 5);

    let alt = stats
        .folders
        .iter()
        .find(|f| f.name == "alt")
        .expect("alt klasoru");
    assert_eq!(alt.total_bytes, 12);
    assert_eq!(alt.file_count, 1);
    assert_eq!(alt.document_bytes, 12);

    // Alt klasordeki direkt dosyalar listelenir.
    assert_eq!(alt.files.len(), 1);
    assert_eq!(alt.files[0].name, "c.pdf");
    assert_eq!(alt.files[0].size, 12);
    assert_eq!(alt.files[0].category, Some(FileCategory::Document));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn banned_extensions_are_skipped_even_in_known_category() {
    let root = build_tree("banned");
    // CSV normalde Dokuman kategorisinde taranir.
    fs::write(root.join("veri.csv"), b"a,b,c").unwrap();
    fs::write(root.join("notlar.txt"), b"metin").unwrap();

    // Yasakli listesindeki csv taranmaz; txt taranmaya devam eder.
    let names: Vec<String> =
        scanner::scan(&root, true, &[], &["csv".to_string()], false, &|_| {}, &|| false)
            .unwrap()
            .into_iter()
            .map(|f| f.name)
            .collect();

    assert!(
        !names.contains(&"veri.csv".to_string()),
        "csv yasakli: {names:?}"
    );
    assert!(names.contains(&"notlar.txt".to_string()));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn normalize_exts_strips_dots_and_lowercases() {
    let out = smart_file_organizer_lib::models::normalize_exts(&[
        " .EXE ".to_string(),
        "Dll".to_string(),
        String::new(),
        "..com".to_string(),
    ]);
    assert_eq!(out, vec!["exe", "dll", "com"]);
}

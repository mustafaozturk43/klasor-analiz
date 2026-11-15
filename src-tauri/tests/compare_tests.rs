//! Iki klasor karsilastirma testleri (FAZ 3).

use smart_file_organizer_lib::compare::run_compare;
use smart_file_organizer_lib::hash_cache::HashCache;
use smart_file_organizer_lib::models::GroupType;
use std::fs;
use std::path::PathBuf;

/// Test basina benzersiz gecici klasor.
fn temp_dir(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("sfo_cmp_{}_{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn compare_finds_cross_folder_exact_duplicates() {
    let a = temp_dir("a_cross");
    let b = temp_dir("b_cross");

    // A ve B'de ayni icerik -> birebir kopya (klasorler arasi).
    fs::write(a.join("rapor.pdf"), b"ORTAK-ICERIK").unwrap();
    fs::write(b.join("rapor_kopya.pdf"), b"ORTAK-ICERIK").unwrap();
    // Yalniz A'da tekil dosya.
    fs::write(a.join("tekil.pdf"), b"A-ICERIK").unwrap();
    // Yalniz B'de tekil dosya.
    fs::write(b.join("baska.pdf"), b"B-ICERIK").unwrap();

    let result = run_compare(
        &a,
        &b,
        true,
        0.82,
        &[],
        &[],
        false,
        &HashCache::new(),
        &|_| {},
        &|_, _| {},
        &|| false,
    )
    .unwrap();

    assert_eq!(result.total_files, 4);
    assert_eq!(result.root_path, a.to_string_lossy());

    let exact = result
        .groups
        .iter()
        .find(|g| matches!(g.group_type, GroupType::Exact))
        .expect("A<->B birebir kopya grubu bekleniyor");
    assert_eq!(exact.files.len(), 2);

    // Grup iki klasore de yayilmis olmali (cross-folder).
    let from_a = exact
        .files
        .iter()
        .any(|f| f.path.starts_with(a.to_string_lossy().as_ref()));
    let from_b = exact
        .files
        .iter()
        .any(|f| f.path.starts_with(b.to_string_lossy().as_ref()));
    assert!(from_a && from_b, "kopyalar A ve B'ye dagilmali: {:?}", exact.files);

    fs::remove_dir_all(&a).unwrap();
    fs::remove_dir_all(&b).unwrap();
}

#[test]
fn compare_respects_banned_extensions_on_both_roots() {
    let a = temp_dir("a_ban");
    let b = temp_dir("b_ban");

    fs::write(a.join("a.pdf"), b"A").unwrap();
    fs::write(b.join("b.csv"), b"CSV").unwrap();
    fs::write(b.join("b.txt"), b"TXT").unwrap();

    let result = run_compare(
        &a,
        &b,
        true,
        0.82,
        &[],
        &["csv".to_string()],
        false,
        &HashCache::new(),
        &|_| {},
        &|_, _| {},
        &|| false,
    )
    .unwrap();

    // csv yasakli -> toplam 2 dosya taranir.
    assert_eq!(result.total_files, 2);

    fs::remove_dir_all(&a).unwrap();
    fs::remove_dir_all(&b).unwrap();
}

//! Tasima politikalari (F2) ve tam hash dogrulamasi (F3) + karantina testleri.

use smart_file_organizer_lib::models::{
    FileCategory, FileGroup, FileItem, GroupType, KeepPolicy, QUARANTINE_DIR,
};
use smart_file_organizer_lib::mover;
use std::fs;

/// Test basina benzersiz gecici klasor.
fn temp_root(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("sfo_move_{}_{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

/// Manuel FileItem uretir (tarama komutlarindan bagimsiz).
fn item(root: &std::path::Path, rel: &str, mtime: u64, size: u64) -> FileItem {
    let path = root.join(rel);
    let name = path.file_name().unwrap().to_string_lossy().into_owned();
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    FileItem {
        path: path.to_string_lossy().into_owned(),
        name,
        stem,
        extension: "pdf".to_string(),
        size,
        mtime,
        category: FileCategory::Document,
        hash_id: String::new(),
    }
}

fn exact_group(files: Vec<FileItem>) -> Vec<FileGroup> {
    vec![FileGroup {
        id: "GRP-0001".to_string(),
        group_type: GroupType::Exact,
        category: FileCategory::Document,
        wasted_bytes: 0,
        files,
    }]
}

#[test]
fn keep_policy_newest_keeps_newest_mtime() {
    let root = temp_root("policy");
    fs::write(root.join("eski.pdf"), b"icerik-eski").unwrap();
    fs::write(root.join("yeni.pdf"), b"icerik-yeni").unwrap();

    let files = vec![
        item(&root, "eski.pdf", 1000, 11),
        item(&root, "yeni.pdf", 2000, 11),
    ];
    let groups = exact_group(files);

    let result =
        mover::apply_move(&root, &groups, "Rapor", KeepPolicy::Newest, false).unwrap();

    assert_eq!(result.moved_to_tag.len(), 1);
    // En yeni (mtime=2000) etiket klasorune, eski karantinaya gitmelidir.
    assert!(root.join("Rapor").join("yeni.pdf").exists());
    assert!(
        root.join(QUARANTINE_DIR)
            .join("Rapor")
            .join("eski.pdf")
            .exists()
    );
    assert!(result.skipped.is_empty());

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn keep_policy_shortest_path_picks_shallower_file() {
    let root = temp_root("shortest");
    let sub = root.join("alt");
    fs::create_dir_all(&sub).unwrap();
    fs::write(root.join("a.pdf"), b"A").unwrap();
    fs::write(sub.join("a.pdf"), b"A").unwrap();

    let files = vec![
        item(&root, "alt\\a.pdf", 500, 1),
        item(&root, "a.pdf", 500, 1),
    ];
    let groups = exact_group(files);

    mover::apply_move(&root, &groups, "Kopya", KeepPolicy::ShortestPath, false).unwrap();

    // En kisa yol (root\a.pdf) etikette kalmali; derin kopya karantinada.
    assert!(root.join("Kopya").join("a.pdf").exists());
    assert!(
        root.join(QUARANTINE_DIR)
            .join("Kopya")
            .join("a.pdf")
            .exists()
    );

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn full_hash_verification_skips_mismatched_member() {
    let root = temp_root("verify");
    fs::write(root.join("a.pdf"), b"AYNI-ICERIK").unwrap();
    fs::write(root.join("b.pdf"), b"FARKLI-ICERIK").unwrap();

    // Boyutlari 500 MB esiginin uzerinde gosterir (F3 arama yolu calinsin).
    let files = vec![
        item(&root, "a.pdf", 100, 600 * 1024 * 1024),
        item(&root, "b.pdf", 100, 600 * 1024 * 1024),
    ];
    let groups = exact_group(files);

    let result = mover::apply_move(&root, &groups, "Dogrula", KeepPolicy::First, true).unwrap();

    // b.pdf hash uymadigi icin yerinde kalmali; a.pdf etikete gitmeli.
    assert_eq!(result.skipped.len(), 1);
    assert!(root.join("b.pdf").exists());
    assert!(root.join("Dogrula").join("a.pdf").exists());
    assert_eq!(result.moved_to_quarantine.len(), 0);

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn full_hash_verification_moves_identical_members() {
    let root = temp_root("verify_ok");
    fs::write(root.join("a.pdf"), b"AYNI-ICERIK").unwrap();
    fs::write(root.join("b.pdf"), b"AYNI-ICERIK").unwrap();

    let files = vec![
        item(&root, "a.pdf", 100, 600 * 1024 * 1024),
        item(&root, "b.pdf", 100, 600 * 1024 * 1024),
    ];
    let groups = exact_group(files);

    let result = mover::apply_move(&root, &groups, "Dogrula", KeepPolicy::First, true).unwrap();

    assert!(result.skipped.is_empty());
    assert!(root.join("Dogrula").join("a.pdf").exists());
    assert!(
        root.join(QUARANTINE_DIR)
            .join("Dogrula")
            .join("b.pdf")
            .exists()
    );

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn quarantine_count_and_restore_roundtrip() {
    let root = temp_root("restore");
    fs::write(root.join("c1.pdf"), b"K1").unwrap();
    fs::write(root.join("c2.pdf"), b"K2").unwrap();

    let files = vec![
        item(&root, "c1.pdf", 10, 2),
        item(&root, "c2.pdf", 10, 2),
    ];
    let groups = exact_group(files);
    mover::apply_move(&root, &groups, "Eski", KeepPolicy::First, false).unwrap();

    // 2 dosyadan 1'i etikette, 1'i karantinada.
    assert_eq!(mover::quarantine_count(&root).unwrap(), 1);

    let restored = mover::restore_from_quarantine(&root, "Eski").unwrap();
    assert_eq!(restored, 1);
    // Karantinadaki c2, etiket klasorune (catisma olmadigi icin ayni isimle) dondu.
    assert!(root.join("Eski").join("c1.pdf").exists());
    assert!(root.join("Eski").join("c2.pdf").exists());
    assert_eq!(mover::quarantine_count(&root).unwrap(), 0);

    fs::remove_dir_all(&root).unwrap();
}

//! Klasor dosya listesi (list_files) dogruluk testleri.

use smart_file_organizer_lib::scanner;
use std::fs;
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("sfo_fl_{}_{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

fn names(files: &[smart_file_organizer_lib::models::FolderFile]) -> Vec<String> {
    files.iter().map(|f| f.name.clone()).collect()
}

#[test]
fn list_files_returns_categorized_only_by_default() {
    let root = temp_dir("cat_only");
    fs::write(root.join("rapor.pdf"), b"x").unwrap(); // Dokuman
    fs::write(root.join("foto.jpg"), b"x").unwrap(); // Resim
    fs::write(root.join("notlar.log"), b"x").unwrap(); // kategoride degil
    fs::write(root.join("script.bat"), b"x").unwrap(); // yasakli (varsayilan degil ama burda banned bos)

    let files = scanner::list_files(&root, true, &[], &[], false).unwrap();
    let n = names(&files);

    assert!(n.contains(&"rapor.pdf".to_string()));
    assert!(n.contains(&"foto.jpg".to_string()));
    assert!(!n.contains(&"notlar.log".to_string()), "kategoride degil: {n:?}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn list_files_returns_all_when_uncategorized_included() {
    let root = temp_dir("all");
    fs::write(root.join("rapor.pdf"), b"x").unwrap();
    fs::write(root.join("notlar.log"), b"x").unwrap(); // kategoride degil

    let cat = scanner::list_files(&root, true, &[], &[], false).unwrap();
    let all = scanner::list_files(&root, true, &[], &[], true).unwrap();

    assert_eq!(cat.len(), 1, "sadece kategori: {:?}", names(&cat));
    assert_eq!(all.len(), 2, "tum dosyalar: {:?}", names(&all));
    assert!(names(&all).contains(&"notlar.log".to_string()));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn list_files_excludes_banned_and_quarantine() {
    let root = temp_dir("exclude");
    fs::write(root.join("a.exe"), b"x").unwrap(); // yasakli
    fs::write(root.join("b.pdf"), b"x").unwrap();
    fs::create_dir_all(root.join("_Kopyalar_Karantina")).unwrap();
    fs::write(root.join("_Kopyalar_Karantina").join("c.pdf"), b"x").unwrap();

    let files = scanner::list_files(&root, true, &[], &["exe".to_string()], true).unwrap();
    let n = names(&files);

    assert!(!n.contains(&"a.exe".to_string()), "yasakli: {n:?}");
    assert!(!n.contains(&"c.pdf".to_string()), "karantina: {n:?}");
    assert!(n.contains(&"b.pdf".to_string()));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn list_files_excludes_subdirs_when_disabled() {
    let root = temp_dir("depth");
    fs::write(root.join("top.pdf"), b"x").unwrap();
    fs::create_dir_all(root.join("alt")).unwrap();
    fs::write(root.join("alt").join("inner.pdf"), b"x").unwrap();

    let files = scanner::list_files(&root, false, &[], &[], true).unwrap();
    let n = names(&files);

    assert_eq!(files.len(), 1, "sadece kok: {n:?}");
    assert!(n.contains(&"top.pdf".to_string()));
    assert!(!n.contains(&"inner.pdf".to_string()));

    fs::remove_dir_all(&root).unwrap();
}

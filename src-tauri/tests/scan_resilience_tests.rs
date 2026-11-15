//! Tarama dayanikliligi: gelen kutusu taramaya girmez; okunamayan (kilitli) dosya
//! taramayi durdurmaz, gruplara girmez ve uyari olarak bildirilir.

use smart_file_organizer_lib::models::GroupType;
use smart_file_organizer_lib::{grouper, hash_cache::HashCache, scanner};
use std::fs;
use std::path::PathBuf;

fn temp_root(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("sfo_resilience_{}_{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn scan_skips_inbox_folder() {
    let root = temp_root("inbox_skip");
    let inbox = root.join("_GelenKutusu").join("2026-11-14_Bakilacaklar").join("Dokümanlar");
    fs::create_dir_all(&inbox).unwrap();
    fs::write(inbox.join("bekleyen.pdf"), b"x").unwrap();
    fs::write(root.join("normal.pdf"), b"y").unwrap();
    let names: Vec<String> = scanner::scan(&root, true, &[], &[], false, &|_| {}, &|| false).unwrap().into_iter().map(|f| f.name).collect();
    assert_eq!(names, vec!["normal.pdf".to_string()]);
    fs::remove_dir_all(&root).unwrap();
}

/// Baska bir programin kilitledigi dosya taramayi durdurmaz; gruplara girmez, uyari olur.
#[cfg(windows)]
#[test]
fn unreadable_file_is_reported_and_does_not_abort_grouping() {
    use std::os::windows::fs::OpenOptionsExt;
    let root = temp_root("locked");
    fs::write(root.join("a.pdf"), b"AYNI-ICERIK").unwrap();
    fs::write(root.join("b.pdf"), b"AYNI-ICERIK").unwrap();
    fs::write(root.join("kilitli.pdf"), b"AYNI-ICERIK").unwrap();
    // share_mode(0): dosya acikken baska hicbir okuma acilamaz.
    let lock = fs::OpenOptions::new().read(true).share_mode(0).open(root.join("kilitli.pdf")).unwrap();

    let out = scanner::scan_reporting(&root, true, &[], &[], false, &|_| {}, &|| false).unwrap();
    assert_eq!(out.files.len(), 3);
    let (groups, warnings) = grouper::group_reporting(out.files, 0.82, &HashCache::new(), &|_, _| {}, &|| false).unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("kilitli.pdf"));
    let exact: Vec<_> = groups.iter().filter(|g| g.group_type == GroupType::Exact).collect();
    assert_eq!(exact.len(), 1);
    assert_eq!(exact[0].files.len(), 2, "kilitli dosya gruba girmez");
    drop(lock);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn hash_cache_survives_restart_and_history_exports_to_csv() {
    use smart_file_organizer_lib::hash_cache::HashCache;
    use smart_file_organizer_lib::models::{MoveRecord, UndoRecord};
    use smart_file_organizer_lib::reporter::write_history_csv;
    let dir = std::env::temp_dir().join(format!("sfo_cache_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    let cache = HashCache::new();
    cache.set_full("C:/x/a.pdf", 10, 5, "abc".into());
    cache.save(&dir).unwrap();
    let reloaded = HashCache::load(&dir);
    assert_eq!(reloaded.get_full("C:/x/a.pdf", 10, 5).as_deref(), Some("abc"));
    assert_eq!(reloaded.get_full("C:/x/a.pdf", 11, 5), None, "boyut degisince gecersiz");

    let record = UndoRecord { id: "MV-1".into(), timestamp: 1_700_000_000_000, tag: "=Fatura".into(), moves: vec![MoveRecord { from: "C:/a;b.pdf".into(), to: "C:/t/a.pdf".into() }] };
    let csv = dir.join("gecmis.csv");
    assert_eq!(write_history_csv(&csv, &[record]).unwrap(), 1);
    let text = std::fs::read_to_string(&csv).unwrap();
    assert!(text.contains("MV-1;") && text.contains(";'=Fatura;") && text.contains("\"C:/a;b.pdf\""), "{text}");
    std::fs::remove_dir_all(&dir).unwrap();
}

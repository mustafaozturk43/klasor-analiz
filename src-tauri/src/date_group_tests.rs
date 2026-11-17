use super::*;
use crate::hash_cache::HashCache;
use crate::AppState;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

fn scratch(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("sfo_dgroup_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(base.join("Alt")).unwrap();
    base
}

/// 2026-01-15 12:00 UTC (her saat diliminde Ocak).
const JAN: u64 = 1_768_478_400;

fn no_progress(_: u64, _: u64) {}

#[test]
fn folder_uses_year_and_month_or_undated() {
    assert_eq!(folder_for(Some(JAN)), "2026\\01");
    assert_eq!(folder_for(None), UNDATED);
}

#[test]
fn plan_prefers_date_taken_filters_extensions_and_numbers_clashes() {
    let root = scratch("plan");
    for n in ["a.jpg", "b.png", "c.pdf"] {
        fs::write(root.join(n), b"x").unwrap();
    }
    fs::write(root.join("Alt").join("d.jpg"), b"x").unwrap();
    fs::create_dir_all(root.join("2026").join("01")).unwrap();
    fs::write(root.join("2026").join("01").join("a.jpg"), b"eski").unwrap();
    let taken = |p: &Path| (p.extension().is_some_and(|e| e == "jpg")).then_some(JAN);
    let opts = DateGroupOptions { extensions: vec!["JPG".into(), ".png".into()], recursive: false };
    let (plan, stamps) = plan_date_group(&root, &opts, &taken, &no_progress).unwrap();
    assert_eq!(plan.entries.len(), 2, "pdf ve alt klasor disarida");
    assert_eq!(stamps.len(), 2);
    assert_eq!(plan.from_taken, 1);
    let a = plan.entries.iter().find(|e| e.from.ends_with("a.jpg")).unwrap();
    assert!(a.to.ends_with("2026\\01\\a (1).jpg"), "{}", a.to);
    assert_eq!(plan.renamed, 1);
    let b = plan.entries.iter().find(|e| e.from.ends_with("b.png")).unwrap();
    assert!(!b.to.contains(UNDATED), "olusturma tarihi kullanilir: {}", b.to);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn recursive_plan_skips_files_already_in_place() {
    let root = scratch("rec");
    fs::create_dir_all(root.join("2026").join("01")).unwrap();
    fs::write(root.join("2026").join("01").join("yerinde.jpg"), b"x").unwrap();
    fs::write(root.join("Alt").join("tasin.jpg"), b"x").unwrap();
    let opts = DateGroupOptions { extensions: Vec::new(), recursive: true };
    let (plan, _) = plan_date_group(&root, &opts, &|_| Some(JAN), &no_progress).unwrap();
    assert_eq!(plan.in_place, 1);
    assert_eq!(plan.entries.len(), 1);
    assert_eq!(plan.buckets[0].folder, "2026\\01");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn apply_moves_files_and_undo_restores_and_removes_new_folders() {
    let root = scratch("apply");
    fs::write(root.join("foto.jpg"), b"x").unwrap();
    let app = AppState { history: Mutex::new(Vec::new()), dir: root.join("h"), cancel_scan: AtomicBool::new(false), hash_cache: HashCache::new() };
    let (plan, stamps) = plan_date_group(&root, &DateGroupOptions::default(), &|_| Some(JAN), &no_progress).unwrap();
    let out = crate::date_group::apply_core(&app, &plan, &stamps).unwrap();
    assert_eq!(out.moved.len(), 1);
    assert!(root.join("2026").join("01").join("foto.jpg").exists());
    assert!(crate::date_group::apply_core(&app, &plan, &stamps).is_err(), "eski plan ikinci kez uygulanmaz");

    let id = app.history.lock().unwrap()[0].id.clone();
    crate::history_cmds::undo_entry_core(&app, &id).unwrap();
    assert!(root.join("foto.jpg").exists());
    assert!(!root.join("2026").exists(), "bosalan Yıl\\Ay klasorleri kalkar");
    let _ = fs::remove_dir_all(&root);
}

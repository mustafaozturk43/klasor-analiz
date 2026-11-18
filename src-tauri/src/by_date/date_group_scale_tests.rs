//! Tarihe Gore gruplama: buyuk klasor (5.000 dosya) ve kismi basarisizlik senaryolari.

use super::*;
use crate::by_date::date_group_plan::{plan_date_group, DateGroupOptions};
use crate::hash_cache::HashCache;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// 2026-01-15 12:00 UTC.
const JAN: u64 = 1_768_478_400;
const DAY: u64 = 86_400;

fn no_progress(_: u64, _: u64) {}

fn scratch(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("sfo_dgscale_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

fn app(dir: PathBuf) -> AppState {
    AppState { history: Mutex::new(Vec::new()), dir, cancel_scan: AtomicBool::new(false), hash_cache: HashCache::new() }
}

/// Dosya adindaki sayidan 24 aylik bir tarih araligi uretir (cekim tarihi yerine).
fn spread(p: &Path) -> Option<u64> {
    let n: u64 = p.file_stem()?.to_string_lossy().trim_start_matches("f_").parse().ok()?;
    Some(JAN + (n % 24) * 30 * DAY)
}

#[test]
#[ignore = "yavas (~40 sn): npm run test:slow"]
fn five_thousand_files_plan_apply_and_undo_complete_in_reasonable_time() {
    let root = scratch("big");
    for i in 0..5000 {
        fs::write(root.join(format!("f_{i}.jpg")), format!("x{i}")).unwrap();
    }
    let app = app(root.join("h"));
    let t = Instant::now();
    let (plan, stamps) = plan_date_group(&root, &DateGroupOptions::default(), &spread, &no_progress).unwrap();
    let planned = t.elapsed();
    assert_eq!(plan.entries.len(), 5000);
    assert!(plan.buckets.len() >= 24, "24 aylik dagilim: {}", plan.buckets.len());
    assert_eq!(plan.buckets.iter().map(|b| b.count).sum::<usize>(), 5000);

    let t = Instant::now();
    let out = apply_core(&app, &plan, &stamps, &no_progress).unwrap();
    let applied = t.elapsed();
    assert_eq!((out.moved.len(), out.failed.len()), (5000, 0));
    assert_eq!(app.history.lock().unwrap()[0].moves.len(), 5000);
    assert!(!root.join("f_0.jpg").exists());
    eprintln!("5000 dosya: plan {planned:?}, uygulama {applied:?}");
    assert!(planned < Duration::from_secs(60) && applied < Duration::from_secs(60), "cok yavas: plan {planned:?}, uygulama {applied:?}");

    let id = app.history.lock().unwrap()[0].id.clone();
    crate::history_cmds::undo_entry_core(&app, &id).unwrap();
    assert_eq!(fs::read_dir(&root).unwrap().flatten().filter(|e| e.path().extension().is_some_and(|x| x == "jpg")).count(), 5000);
    assert!(!root.join("2026").exists(), "bosalan Yıl klasorleri kalkar");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn partial_failure_records_only_moved_files_and_they_can_be_undone() {
    let root = scratch("fail");
    fs::write(root.join("a.jpg"), b"a").unwrap();
    fs::write(root.join("b.jpg"), b"b").unwrap();
    // `a.jpg` 2026'ya gider; kokte "2026" adli bir DOSYA oldugu icin klasoru olusturulamaz.
    fs::write(root.join("2026"), b"engel").unwrap();
    let taken = |p: &Path| Some(if p.ends_with("a.jpg") { JAN } else { JAN - 400 * DAY });
    let opts = DateGroupOptions { extensions: vec!["jpg".into()], ..Default::default() };
    let (plan, stamps) = plan_date_group(&root, &opts, &taken, &no_progress).unwrap();
    let app = app(root.join("h"));
    let out = apply_core(&app, &plan, &stamps, &no_progress).unwrap();
    assert_eq!((out.moved.len(), out.failed.len()), (1, 1));
    assert!(root.join("a.jpg").exists(), "basarisiz dosya yerinde kalir");
    assert!(!root.join("b.jpg").exists());
    let hist = app.history.lock().unwrap();
    assert_eq!(hist[0].moves.len(), 1, "gecmiste yalniz tasinan dosya var");
    let id = hist[0].id.clone();
    drop(hist);
    crate::history_cmds::undo_entry_core(&app, &id).unwrap();
    assert!(root.join("b.jpg").exists());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn history_is_written_before_files_move() {
    let root = scratch("order");
    fs::write(root.join("a.jpg"), b"a").unwrap();
    let app = app(root.join("h"));
    let (plan, stamps) = plan_date_group(&root, &DateGroupOptions::default(), &|_| Some(JAN), &no_progress).unwrap();
    let seen = Mutex::new(Vec::new());
    // Ilerleme geri cagrisi parti sonunda cagrilir; o ana kadar gecmis dosyasi diskte olmali.
    let progress = |done: u64, _: u64| {
        let on_disk = fs::read_to_string(root.join("h").join("move_history.json")).unwrap_or_default();
        seen.lock().unwrap().push((done, on_disk.contains("a.jpg")));
    };
    apply_core(&app, &plan, &stamps, &progress).unwrap();
    assert!(seen.lock().unwrap().iter().any(|(done, recorded)| *done == 1 && *recorded), "{:?}", seen.lock().unwrap());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn rebuild_index_reflects_disk_and_drops_stale_sections() {
    let root = scratch("rebuild");
    fs::write(root.join("a.jpg"), b"a").unwrap();
    // Gecmis klasoru kokun disinda olmali; yoksa "alt klasor" sayilir.
    let app = app(std::env::temp_dir().join(format!("sfo_dgscale_rebuild_h_{}", std::process::id())));
    let (plan, stamps) = plan_date_group(&root, &DateGroupOptions::default(), &|_| Some(JAN), &no_progress).unwrap();
    apply_core(&app, &plan, &stamps, &no_progress).unwrap();
    let id = app.history.lock().unwrap()[0].id.clone();
    crate::history_cmds::undo_entry_core(&app, &id).unwrap();
    let stale = fs::read_to_string(root.join(crate::date_group_index::INDEX_FILE)).unwrap();
    assert!(stale.contains(">a.jpg<"), "geri alma dizini guncellemez");

    assert!(rebuild_date_index(root.to_string_lossy().into_owned()).is_err(), "alt klasorde dosya yok");
    fs::create_dir_all(root.join("2024").join("05")).unwrap();
    fs::write(root.join("2024").join("05").join("b.jpg"), b"b").unwrap();
    fs::write(root.join("kok.jpg"), b"k").unwrap();
    assert_eq!(rebuild_date_index(root.to_string_lossy().into_owned()).unwrap(), 1, "kok dosyalari dizine girmez");
    let fresh = fs::read_to_string(root.join(crate::date_group_index::INDEX_FILE)).unwrap();
    assert!(fresh.contains(">b.jpg<") && !fresh.contains(">a.jpg<") && !fresh.contains(">kok.jpg<"), "{fresh}");
    assert_eq!(fresh.matches("<!doctype").count(), 1);
    let _ = fs::remove_dir_all(&root);
}

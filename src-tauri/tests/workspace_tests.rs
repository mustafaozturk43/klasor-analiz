use smart_file_organizer_lib::{compare::run_compare, file_ops, folder_stats, hash_cache::HashCache, models::*, mover, rename::plan_date_rename};
use std::{fs, path::PathBuf, sync::atomic::{AtomicUsize, Ordering}};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("sfo_workspace_{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir_all(&path).unwrap(); Self(path)
    }
    fn s(&self) -> String { self.0.to_string_lossy().into_owned() }
}
impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
fn compare(a: &Temp, b: &Temp) -> Result<ScanResult, String> {
    run_compare(&a.0, &b.0, true, 0.82, &[], &[], true, &HashCache::new(), &|_| {}, &|_, _| {}, &|| false)
}

#[test]
fn comparison_retains_unmatched_inventory_and_only_cross_root_groups() {
    let a = Temp::new(); let b = Temp::new();
    fs::write(a.0.join("alpha.pdf"), b"SAME-A").unwrap();
    fs::write(a.0.join("omega.pdf"), b"SAME-A").unwrap();
    fs::write(a.0.join("cross.txt"), b"CROSS").unwrap();
    fs::write(b.0.join("copy.txt"), b"CROSS").unwrap();
    fs::write(b.0.join("unique.log"), b"INDEPENDENT").unwrap();
    let result = compare(&a, &b).unwrap();
    assert_eq!(result.groups.len(), 1);
    let details = result.comparison.unwrap();
    assert_eq!(details.files.len(), 5);
    assert_eq!(details.files.iter().filter(|f| f.group_id.is_some()).count(), 2);
    assert!(details.files.iter().any(|f| f.source == "B" && f.file.name == "unique.log" && f.group_id.is_none()));
}

#[test]
fn same_or_nested_comparison_roots_are_rejected() {
    let a = Temp::new();
    assert!(compare(&a, &a).is_err());
    fs::create_dir(a.0.join("nested")).unwrap();
    assert!(run_compare(&a.0, &a.0.join("nested"), true, 0.82, &[], &[], false, &HashCache::new(), &|_| {}, &|_, _| {}, &|| false).is_err());
}

#[test]
fn analysis_category_totals_include_root_files() {
    let a = Temp::new();
    fs::write(a.0.join("root.pdf"), b"123").unwrap();
    fs::create_dir(a.0.join("images")).unwrap();
    fs::write(a.0.join("images/pic.jpg"), b"12345").unwrap();
    fs::write(a.0.join("data.log"), b"12").unwrap();
    let result = folder_stats::collect(&a.0).unwrap();
    assert_eq!(result.category_bytes, [3, 5, 0, 2]);
    assert_eq!(result.root_total_bytes, 10);
}

#[test]
fn transfer_never_overwrites_existing_destination() {
    let a = Temp::new();
    let from = a.0.join("from.txt"); let to = a.0.join("to.txt");
    fs::write(&from, b"SOURCE").unwrap(); fs::write(&to, b"KEEP").unwrap();
    assert!(file_ops::transfer(&from, &to).is_err());
    assert_eq!(fs::read(to).unwrap(), b"KEEP"); assert!(from.exists());
}

#[test]
fn transfer_and_undo_round_trip_and_refuse_collision() {
    let a = Temp::new();
    let from = a.0.join("from.txt"); let to = a.0.join("dated.txt");
    fs::write(&from, b"DATA").unwrap();
    file_ops::transfer(&from, &to).unwrap();
    let record = UndoRecord { id: "RN-TEST".into(), timestamp: 0, tag: "Rename".into(), moves: vec![MoveRecord { from: from.to_string_lossy().into_owned(), to: to.to_string_lossy().into_owned() }] };
    fs::write(&from, b"OTHER").unwrap();
    assert!(mover::undo_move(&record).is_err());
    assert_eq!(fs::read(&from).unwrap(), b"OTHER");
    fs::remove_file(&from).unwrap();
    mover::undo_move(&record).unwrap();
    assert_eq!(fs::read(from).unwrap(), b"DATA"); assert!(!to.exists());
}

#[test]
fn copy_fallback_keeps_source_timestamps_and_never_overwrites() {
    let a = Temp::new();
    let from = a.0.join("eski.txt"); let to = a.0.join("kopya.txt");
    fs::write(&from, b"DATA").unwrap();
    let old = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000_000);
    fs::File::options().write(true).open(&from).unwrap().set_modified(old).unwrap();
    file_ops::copy_preserving(&from, &to).unwrap();
    assert_eq!(fs::read(&to).unwrap(), b"DATA");
    assert_eq!(fs::metadata(&to).unwrap().modified().unwrap(), old, "degisiklik tarihi korunur");
    #[cfg(windows)]
    assert_eq!(fs::metadata(&to).unwrap().created().unwrap(), fs::metadata(&from).unwrap().created().unwrap(), "olusturulma tarihi korunur");
    assert!(file_ops::copy_preserving(&from, &to).is_err(), "var olan hedefin uzerine yazilmaz");
}

#[test]
fn date_suffix_handles_epoch_and_leap_day() {
    assert_eq!(file_ops::date_suffix(0), "01-01-1970");
    assert_eq!(file_ops::date_suffix(1709164800), "29-02-2024");
}

#[test]
fn rename_plan_is_read_only_and_preserves_extension() {
    let a = Temp::new(); let path = a.0.join("report.pdf");
    fs::write(&path, b"DATA").unwrap();
    let (plan, _) = plan_date_rename(vec![a.s()], vec![path.to_string_lossy().into_owned()]).unwrap();
    assert_eq!(plan.entries.len(), 1);
    if let Some(to) = &plan.entries[0].to {
        assert!(to.ends_with(".pdf")); assert!(!PathBuf::from(to).exists());
    } else { assert!(plan.entries[0].warning.is_some()); }
    assert!(path.exists());
}

#[test]
fn rename_plan_rejects_outside_and_duplicate_sources() {
    let a = Temp::new(); let b = Temp::new();
    let path = b.0.join("data.txt"); fs::write(&path, b"DATA").unwrap();
    let raw = path.to_string_lossy().into_owned();
    assert!(plan_date_rename(vec![a.s()], vec![raw.clone()]).is_err());
    assert!(plan_date_rename(vec![b.s()], vec![raw.clone(), raw]).is_err());
}

#[test]
fn rename_plan_reserves_existing_targets_without_clobbering() {
    let a = Temp::new(); let path = a.0.join("data.txt");
    fs::write(&path, b"DATA").unwrap();
    let raw = path.to_string_lossy().into_owned();
    let (first, _) = plan_date_rename(vec![a.s()], vec![raw.clone()]).unwrap();
    if let Some(to) = &first.entries[0].to {
        fs::write(to, b"KEEP").unwrap();
        let (second, _) = plan_date_rename(vec![a.s()], vec![raw]).unwrap();
        assert_ne!(first.entries[0].to, second.entries[0].to);
        assert_eq!(fs::read(to).unwrap(), b"KEEP");
    }
}

#[test]
fn local_date_suffix_follows_the_machine_time_zone() {
    use chrono::{Local, TimeZone};
    // 31-12-2023 22:30 UTC: UTC+2 ve otesinde yerel gun 01-01-2024'tur.
    let secs = 1_704_061_800u64;
    let expected = Local.timestamp_opt(secs as i64, 0).unwrap().format("%d-%m-%Y").to_string();
    assert_eq!(file_ops::local_date_suffix(secs), expected);
    assert_eq!(file_ops::date_suffix(secs), "31-12-2023");
}

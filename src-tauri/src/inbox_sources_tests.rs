use super::*;

fn scratch(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("sfo_sources_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(base.join("Desktop").join("Alt")).unwrap();
    base
}

#[test]
fn lists_only_todays_eligible_top_level_files() {
    let base = scratch("list");
    let src = base.join("Desktop");
    fs::write(src.join("teklif.pdf"), b"x").unwrap();
    fs::write(src.join("indiriliyor.crdownload"), b"x").unwrap();
    fs::write(src.join("Alt").join("derin.pdf"), b"x").unwrap();
    let today = Local::now().date_naive();
    let files = source_files(std::slice::from_ref(&src), today);
    assert_eq!(files.len(), 1);
    assert!(files[0].pending);
    assert_eq!(files[0].group, "İzlenen · Desktop");
    assert!(source_files(std::slice::from_ref(&src), today.pred_opt().unwrap()).is_empty());
    let days = source_days(std::slice::from_ref(&src), today);
    assert_eq!(days.len(), 1);
    assert_eq!(days[0].files, 1);
}

#[test]
fn pending_source_check_accepts_only_direct_children() {
    let base = scratch("check");
    let src = base.join("Desktop");
    fs::write(src.join("a.pdf"), b"x").unwrap();
    fs::write(src.join("Alt").join("b.pdf"), b"x").unwrap();
    fs::write(base.join("c.pdf"), b"x").unwrap();
    let sources = vec![src.clone()];
    assert!(is_pending_source(&sources, &src.join("a.pdf")));
    assert!(!is_pending_source(&sources, &src.join("Alt").join("b.pdf")));
    assert!(!is_pending_source(&sources, &base.join("c.pdf")));
    assert!(!is_pending_source(&[], &src.join("a.pdf")));
}

#[test]
fn merging_days_sums_counts_and_sorts_newest_first() {
    let d = |date: &str, files| InboxDay { date: date.into(), files };
    let merged = merge_days(vec![d("2026-11-15", 2)], vec![d("2026-11-15", 1), d("2026-11-17", 4)]);
    assert_eq!(merged.iter().map(|x| (x.date.as_str(), x.files)).collect::<Vec<_>>(), vec![("2026-11-17", 4), ("2026-11-15", 3)]);
}

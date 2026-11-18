use super::*;
use crate::engine::rule_model::RuleAction;
use crate::hash_cache::HashCache;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

fn scratch(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("sfo_rules_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

fn app(tag: &str) -> AppState {
    let dir = std::env::temp_dir().join(format!("sfo_rules_hist_{tag}_{}", std::process::id()));
    AppState { history: Mutex::new(Vec::new()), dir, cancel_scan: AtomicBool::new(false), hash_cache: HashCache::new() }
}

fn rule(name: &str, action: RuleAction) -> Rule {
    Rule { name: name.into(), enabled: true, exts: Vec::new(), name_contains: String::new(), min_mb: None, max_mb: None, min_age_days: None, action, target: "Hedef".into() }
}

fn none(_: u64, _: u64) {}
const NOW: u64 = 4_000_000_000;

#[test]
fn first_matching_rule_wins_and_unmatched_files_stay() {
    let root = scratch("first");
    for n in ["ocak_fatura.pdf", "rapor.pdf", "foto.jpg"] {
        fs::write(root.join(n), b"x").unwrap();
    }
    let mut fatura = rule("Fatura", RuleAction::Move);
    (fatura.exts, fatura.name_contains, fatura.target) = (vec!["pdf".into()], "fatura".into(), "Faturalar".into());
    let mut pdf = rule("PDF", RuleAction::Move);
    (pdf.exts, pdf.target) = (vec!["pdf".into()], "PDF".into());
    let (plan, stamps) = plan_rules(&root, &[fatura, pdf], false, NOW).unwrap();
    assert_eq!((plan.entries.len(), plan.unmatched, stamps.len()), (3 - 1, 1, 2));
    let by = |n: &str| plan.entries.iter().find(|e| e.from.ends_with(n)).unwrap();
    assert_eq!(by("ocak_fatura.pdf").rule, "Fatura", "ilk eslesen kural kazanir");
    assert_eq!(by("rapor.pdf").rule, "PDF");
    assert!(by("rapor.pdf").to.ends_with("PDF\\rapor.pdf"));
    assert_eq!(plan.counts.iter().map(|c| c.count).collect::<Vec<_>>(), [1, 1]);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn disabled_rules_are_ignored_and_no_enabled_rule_is_an_error() {
    let root = scratch("off");
    fs::write(root.join("a.txt"), b"x").unwrap();
    let mut r = rule("Kapali", RuleAction::Move);
    (r.exts, r.enabled) = (vec!["txt".into()], false);
    assert!(plan_rules(&root, &[r], false, NOW).unwrap_err().contains("Etkin kural yok"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn year_month_action_age_condition_and_in_place_skip() {
    let root = scratch("ym");
    fs::write(root.join("eski.log"), b"x").unwrap();
    // Dosyalarin degisiklik tarihi simdi; "zaten.log" hedef Yil\Ay klasorunde, yani yerinde.
    let (y, m) = crate::by_date::date_group_layout::year_month(SystemTime::now().duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs())).unwrap();
    let here = root.join("Hedef").join(y.to_string()).join(format!("{m:02}"));
    fs::create_dir_all(&here).unwrap();
    fs::write(here.join("zaten.log"), b"x").unwrap();
    let mut r = rule("Eski loglar", RuleAction::MoveYearMonth);
    (r.exts, r.min_age_days) = (vec!["log".into()], Some(30));
    let (plan, _) = plan_rules(&root, &[r.clone()], true, NOW).unwrap();
    assert_eq!((plan.entries.len(), plan.in_place), (1, 1));
    assert!(plan.entries[0].to.contains("Hedef\\") && plan.entries[0].to.ends_with("\\eski.log"), "{}", plan.entries[0].to);
    assert_eq!(plan.entries[0].to.matches('\\').count(), root.to_string_lossy().matches('\\').count() + 4, "Hedef\\Yil\\Ay\\dosya");
    // Yeni dosya (simdi olusturuldu) yas kosulunu saglamaz.
    let now_real = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let (fresh, _) = plan_rules(&root, &[r], true, now_real).unwrap();
    assert_eq!(fresh.entries.len(), 0);
    assert!(fresh.in_place == 0 && fresh.unmatched == 2, "{:?}", (fresh.in_place, fresh.unmatched));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn apply_moves_with_history_and_undo_restores() {
    let root = scratch("apply");
    fs::write(root.join("a.pdf"), b"a").unwrap();
    let mut r = rule("PDF", RuleAction::Move);
    (r.exts, r.target) = (vec!["pdf".into()], "Belgeler\\PDF".into());
    let (plan, stamps) = plan_rules(&root, &[r], false, NOW).unwrap();
    let app = app("apply");
    let out = apply_core(&app, &plan, &stamps, false, &|_| Ok(()), &none).unwrap();
    assert_eq!((out.moved.len(), out.trashed, out.failed.len()), (1, 0, 0));
    assert!(root.join("Belgeler").join("PDF").join("a.pdf").exists());
    let id = app.history.lock().unwrap()[0].id.clone();
    crate::history_cmds::undo_entry_core(&app, &id).unwrap();
    assert!(root.join("a.pdf").exists() && !root.join("Belgeler").exists());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn trash_needs_explicit_permission_and_never_touches_history() {
    let root = scratch("trash");
    fs::write(root.join("gecici.bak"), b"x").unwrap();
    let mut r = rule("Gecici", RuleAction::Trash);
    r.exts = vec!["bak".into()];
    let (plan, stamps) = plan_rules(&root, &[r], false, NOW).unwrap();
    assert_eq!((plan.trash_count, plan.entries[0].to.as_str()), (1, ""));
    let app = app("trash");
    let sent = Mutex::new(Vec::new());
    let fake = |p: &Path| {
        sent.lock().unwrap().push(p.to_path_buf());
        Ok(())
    };
    assert!(apply_core(&app, &plan, &stamps, false, &fake, &none).unwrap_err().contains("onaylanmadı"));
    assert!(sent.lock().unwrap().is_empty(), "onaysiz hicbir dosya gonderilmez");
    let out = apply_core(&app, &plan, &stamps, true, &fake, &none).unwrap();
    assert_eq!((out.trashed, out.moved.len()), (1, 0));
    assert_eq!(sent.lock().unwrap().len(), 1);
    assert!(app.history.lock().unwrap().is_empty(), "Geri Donusum Kutusu Gecmis'e yazilmaz");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn changed_files_abort_before_anything_moves() {
    let root = scratch("changed");
    fs::write(root.join("a.pdf"), b"a").unwrap();
    fs::write(root.join("b.bak"), b"b").unwrap();
    let mut mv = rule("PDF", RuleAction::Move);
    mv.exts = vec!["pdf".into()];
    let mut tr = rule("Gecici", RuleAction::Trash);
    tr.exts = vec!["bak".into()];
    let (plan, stamps) = plan_rules(&root, &[mv, tr], false, NOW).unwrap();
    fs::write(root.join("b.bak"), b"degisti, daha uzun").unwrap();
    let app = app("changed");
    assert!(apply_core(&app, &plan, &stamps, true, &|_| Ok(()), &none).is_err());
    assert!(root.join("a.pdf").exists(), "hicbir tasima yapilmadi");
    assert!(app.history.lock().unwrap().is_empty());
    let _ = fs::remove_dir_all(&root);
}

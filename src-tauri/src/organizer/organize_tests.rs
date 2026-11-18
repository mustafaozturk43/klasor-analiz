use super::*;
use crate::hash_cache::HashCache;
use crate::models::{KeepPolicy, QUARANTINE_DIR};
use crate::organize_models::RuleDef;
use std::fs;
use crate::mover;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::time::{Duration, UNIX_EPOCH};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fx { root: PathBuf, app: AppState, org: OrganizeState }
impl Fx {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!("sfo_org_{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        let root = base.join("root");
        fs::create_dir_all(&root).unwrap();
        let app = AppState { history: Mutex::new(Vec::new()), dir: base.join("history"), cancel_scan: AtomicBool::new(false), hash_cache: HashCache::new() };
        Self { root, app, org: OrganizeState::default() }
    }
    /// `rel` yoluna icerik yazar; `age` gun once degistirilmis gibi damgalar.
    fn file(&self, rel: &str, body: &[u8], age_days: u64) -> PathBuf {
        let path = self.root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, body).unwrap();
        let when = UNIX_EPOCH + Duration::from_secs(1_700_000_000 - age_days * 86_400);
        fs::File::options().write(true).open(&path).unwrap().set_modified(when).unwrap();
        path
    }
    fn preview(&self, opts: &OrganizeOptions) -> OrganizePlan {
        preview_core(&self.app, &self.org, &self.root.to_string_lossy(), opts, &|_, _, _| {}).unwrap()
    }
    fn entry<'a>(plan: &'a OrganizePlan, name: &str, dir: &str) -> &'a OrganizeEntry {
        plan.entries.iter().find(|e| e.from.ends_with(name) && e.from.contains(dir)).unwrap()
    }
}
impl Drop for Fx { fn drop(&mut self) { let _ = fs::remove_dir_all(self.root.parent().unwrap()); } }

fn opts() -> OrganizeOptions {
    OrganizeOptions {
        include_subdirs: true, quarantine_duplicates: true, protect_projects: true, include_other: false,
        flat: false, keep_policy: KeepPolicy::First, exclude_folders: Vec::new(), banned_extensions: vec!["exe".into()],
        similarity_threshold: 0.82, rules: Vec::new(), by_year: false, remove_empty_dirs: false,
    }
}

#[test]
fn year_folders_custom_rules_and_bad_rules() {
    let fx = Fx::new();
    fx.file("x/a.pdf", b"a", 0);
    fx.file("x/klip.mp4", b"m", 0);
    let mut o = opts();
    o.by_year = true;
    o.rules = vec![
        RuleDef { category: "Belgeler".into(), kind: "PDF".into(), extensions: vec!["pdf".into()] },
        RuleDef { category: "Videolar".into(), kind: "MP4".into(), extensions: vec!["mp4".into()] },
    ];
    let plan = fx.preview(&o);
    let pdf = Fx::entry(&plan, "a.pdf", "x");
    assert_eq!(pdf.to.as_deref(), Some(fx.root.join("Belgeler").join("PDF").join("2023").join("a.pdf").to_string_lossy().as_ref()));
    assert_eq!(Fx::entry(&plan, "klip.mp4", "x").action, OrganizeAction::Move);
    o.rules.push(RuleDef { category: "X".into(), kind: "Y".into(), extensions: vec!["PDF".into()] });
    assert!(preview_core(&fx.app, &fx.org, &fx.root.to_string_lossy(), &o, &|_, _, _| {}).is_err());
}

#[test]
fn emptied_folders_are_removed_and_undo_recreates_them() {
    let fx = Fx::new();
    let a = fx.file("Eski/Arşiv/2019/a.pdf", b"a", 1);
    fx.file("Dolu/not.mp4", b"m", 1);
    fx.file("Dolu/b.pdf", b"b", 1);
    let mut o = opts();
    o.remove_empty_dirs = true;
    let plan = fx.preview(&o);
    let out = apply_core(&fx.app, &fx.org, &plan.id).unwrap();
    assert_eq!(out.removed_dirs, 3, "Eski, Eski/Arşiv, Eski/Arşiv/2019");
    assert!(!fx.root.join("Eski").exists());
    assert!(fx.root.join("Dolu").exists(), "mp4 kaldigi icin klasor durur");
    let record = fx.app.history.lock().unwrap()[0].clone();
    mover::undo_move(&record).unwrap();
    assert_eq!(fs::read(&a).unwrap(), b"a");
}

#[test]
fn same_name_different_content_gets_numbered_oldest_keeps_name() {
    let fx = Fx::new();
    fx.file("Teklifler/Rapor.pdf", b"AAAA", 10);
    fx.file("Muhasebe/Eski/Rapor.pdf", b"BBBBB", 30);
    let plan = fx.preview(&opts());
    let pdf = fx.root.join("Dokümanlar").join("PDF");
    let old = Fx::entry(&plan, "Rapor.pdf", "Muhasebe");
    let new = Fx::entry(&plan, "Rapor.pdf", "Teklifler");
    assert_eq!(old.to.as_deref(), Some(pdf.join("Rapor.pdf").to_string_lossy().as_ref()));
    assert_eq!(new.to.as_deref(), Some(pdf.join("Rapor (1).pdf").to_string_lossy().as_ref()));
    assert!(new.renamed && !old.renamed);
}

#[test]
fn exact_copies_go_to_quarantine_and_empty_files_are_not_copies() {
    let fx = Fx::new();
    fx.file("A/Fiyat.xlsx", b"SAME", 5);
    fx.file("Yedek/Fiyat.xlsx", b"SAME", 1);
    fx.file("a.txt", b"", 1);
    fx.file("b.txt", b"", 1);
    let plan = fx.preview(&opts());
    let q: Vec<_> = plan.entries.iter().filter(|e| e.action == OrganizeAction::Quarantine).collect();
    assert_eq!(q.len(), 1);
    let excel = plan.kinds.iter().find(|k| k.kind == "Excel").unwrap();
    assert_eq!(excel.count, 1, "karantinaya giden kopya tur sayimina girmez");
    assert!(q[0].to.as_deref().unwrap().contains(QUARANTINE_DIR));
    assert!(q[0].to.as_deref().unwrap().contains("Excel"));
}

#[test]
fn skips_temp_banned_unknown_and_protects_projects() {
    let fx = Fx::new();
    fx.file("~$taslak.docx", b"x", 1);
    fx.file("setup.exe", b"x", 1);
    fx.file("video.mp4", b"x", 1);
    fx.file("web/package.json", b"{}", 1);
    fx.file("web/img/logo.png", b"png", 1);
    let plan = fx.preview(&opts());
    assert!(plan.entries.iter().all(|e| e.action == OrganizeAction::Skip));
    assert!(!plan.entries.iter().any(|e| e.from.contains("logo.png")));
    assert_eq!(plan.protected_dirs.len(), 1);
}

#[test]
fn root_project_is_refused_and_subdirs_switch_is_respected() {
    let fx = Fx::new();
    fx.file("deep/a.pdf", b"x", 1);
    fx.file("b.pdf", b"y", 1);
    let mut o = opts();
    o.include_subdirs = false;
    let plan = fx.preview(&o);
    assert_eq!(plan.entries.len(), 1);
    fx.file("Cargo.toml", b"", 1);
    assert!(preview_core(&fx.app, &fx.org, &fx.root.to_string_lossy(), &o, &|_, _, _| {}).is_err());
}

#[test]
fn apply_is_idempotent_and_undo_restores_everything() {
    let fx = Fx::new();
    let a = fx.file("x/Rapor.pdf", b"one", 3);
    let b = fx.file("y/Rapor.pdf", b"two", 2);
    let c = fx.file("y/copy.pdf", b"one", 1);
    let plan = fx.preview(&opts());
    let out = apply_core(&fx.app, &fx.org, &plan.id).unwrap();
    assert_eq!((out.moved, out.quarantined, out.failed.len()), (2, 1, 0));
    assert!(!a.exists() && !b.exists() && !c.exists());
    assert!(apply_core(&fx.app, &fx.org, &plan.id).is_err(), "plan tek kullanimlik");

    let again = fx.preview(&opts());
    assert!(again.entries.iter().all(|e| e.action == OrganizeAction::InPlace), "ikinci calistirma dosya oynatmaz");

    let record = fx.app.history.lock().unwrap()[0].clone();
    assert_eq!(record.moves.len(), 3);
    mover::undo_move(&record).unwrap();
    assert_eq!(fs::read(&a).unwrap(), b"one");
    assert_eq!(fs::read(&b).unwrap(), b"two");
    assert_eq!(fs::read(&c).unwrap(), b"one");
}

#[test]
fn changed_file_invalidates_plan_without_moving_anything() {
    let fx = Fx::new();
    let a = fx.file("x/a.pdf", b"one", 3);
    let plan = fx.preview(&opts());
    fs::write(&a, b"changed!").unwrap();
    assert!(apply_core(&fx.app, &fx.org, &plan.id).is_err());
    assert!(a.exists());
    assert!(fx.app.history.lock().unwrap().is_empty());
}

#[test]
fn flat_layout_and_similar_names() {
    let fx = Fx::new();
    fx.file("p/Butce_2026_v1.xlsx", b"1", 2);
    fx.file("q/Butce_2026_v2.xlsx", b"22", 1);
    let mut o = opts();
    o.flat = true;
    let plan = fx.preview(&o);
    let e = Fx::entry(&plan, "Butce_2026_v2.xlsx", "q");
    assert_eq!(e.to.as_deref(), Some(fx.root.join("Excel").join("Butce_2026_v2.xlsx").to_string_lossy().as_ref()));
    assert_eq!(e.similar_to.as_deref(), Some("Butce_2026_v1.xlsx"));
}

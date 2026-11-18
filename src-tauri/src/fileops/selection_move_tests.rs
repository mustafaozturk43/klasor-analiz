use super::*;
use crate::hash_cache::HashCache;
use crate::models::QUARANTINE_DIR;
use crate::selection_paths::split_group;
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fx { base: PathBuf, a: PathBuf, b: PathBuf, app: AppState }

impl Fx {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!("sfo_sel_{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::SeqCst)));
        let _ = fs::remove_dir_all(&base);
        let (a, b) = (base.join("A"), base.join("B"));
        fs::create_dir_all(a.join("Faturalar")).unwrap();
        fs::create_dir_all(a.join(QUARANTINE_DIR)).unwrap();
        fs::create_dir_all(&b).unwrap();
        let app = AppState { history: Mutex::new(Vec::new()), dir: base.join("h"), cancel_scan: AtomicBool::new(false), hash_cache: HashCache::new() };
        Self { base, a, b, app }
    }
    fn file(&self, dir: &Path, name: &str, body: &[u8]) -> String {
        let p = dir.join(name);
        fs::write(&p, body).unwrap();
        p.to_string_lossy().into_owned()
    }
    fn req(&self, paths: Vec<String>, base: &Path, group: &str) -> SelectionMoveRequest {
        SelectionMoveRequest {
            roots: vec![self.a.to_string_lossy().into_owned(), self.b.to_string_lossy().into_owned()],
            paths, target_base: base.to_string_lossy().into_owned(), group: group.into(),
        }
    }
}

impl Drop for Fx {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.base); }
}

fn names(plan: &SelectionMovePlan) -> Vec<String> {
    plan.entries.iter().map(|e| Path::new(&e.to).file_name().unwrap().to_string_lossy().into_owned()).collect()
}

#[test]
fn group_segments_split_on_both_slashes() {
    assert_eq!(split_group(" Faturalar \\ Mayıs/ Su ").unwrap(), vec!["Faturalar", "Mayıs", "Su"]);
    assert!(split_group("  ").unwrap().is_empty());
}

#[test]
fn group_rejects_traversal_reserved_and_empty_segments() {
    for g in ["..\\Windows", "a\\\\b", "CON", "lpt1.txt", "a:b", "x/./y", "a\\"] {
        assert!(split_group(g).is_err(), "{g} reddedilmeli");
    }
}

#[test]
fn moves_into_nested_new_group_under_base() {
    let fx = Fx::new();
    let f = fx.file(&fx.a, "a.pdf", b"a");
    let (plan, _) = plan_selection_move(&fx.req(vec![f], &fx.a.join("Faturalar"), "Mayıs")).unwrap();
    let target = fx.a.join("Faturalar").join("Mayıs");
    assert_eq!(PathBuf::from(&plan.target_dir), target);
    assert_eq!(plan.created_dirs, vec![target.to_string_lossy().into_owned()]);
    assert_eq!(PathBuf::from(&plan.entries[0].to), target.join("a.pdf"));
    assert_eq!(plan.renamed, 0);
}

#[test]
fn empty_group_moves_directly_into_base() {
    let fx = Fx::new();
    let f = fx.file(&fx.a, "a.pdf", b"a");
    let (plan, _) = plan_selection_move(&fx.req(vec![f], &fx.a.join("Faturalar"), "")).unwrap();
    assert_eq!(PathBuf::from(&plan.target_dir), fx.a.join("Faturalar"));
    assert!(plan.created_dirs.is_empty());
}

#[test]
fn same_name_from_two_roots_gets_numbered() {
    let fx = Fx::new();
    fx.file(&fx.a.join("Faturalar"), "x.jpg", b"eski");
    let (p1, p2) = (fx.file(&fx.a, "x.jpg", b"1"), fx.file(&fx.b, "x.jpg", b"2"));
    let (plan, _) = plan_selection_move(&fx.req(vec![p1, p2], &fx.a.join("Faturalar"), "")).unwrap();
    assert_eq!(names(&plan), vec!["x (1).jpg", "x (2).jpg"]);
    assert_eq!(plan.renamed, 2);
}

#[test]
fn rejects_outside_roots_quarantine_and_same_folder() {
    let fx = Fx::new();
    let inside = fx.file(&fx.a, "a.pdf", b"a");
    let outside = fx.file(&fx.base, "dis.pdf", b"d");
    assert!(plan_selection_move(&fx.req(vec![outside], &fx.a.join("Faturalar"), "")).is_err(), "kok disi");
    assert!(plan_selection_move(&fx.req(vec![inside.clone()], &fx.a.join(QUARANTINE_DIR), "")).is_err(), "karantina hedefi");
    assert!(plan_selection_move(&fx.req(vec![inside.clone()], &fx.a, "Yeni\\..\\..")).is_err(), "kok disina cikis");
    let err = plan_selection_move(&fx.req(vec![inside], &fx.a, "")).unwrap_err();
    assert_eq!(err, "Dosyalar zaten bu klasörde.");
}

#[test]
fn apply_moves_and_writes_history_before_transfer() {
    let fx = Fx::new();
    let (p1, p2) = (fx.file(&fx.a, "a.pdf", b"a"), fx.file(&fx.b, "b.pdf", b"b"));
    let (plan, stamps) = plan_selection_move(&fx.req(vec![p1.clone(), p2.clone()], &fx.a.join("Faturalar"), "Mayıs")).unwrap();
    let out = apply_selection_core(&fx.app, &plan, &stamps).unwrap();
    assert_eq!(out.moved.len(), 2);
    assert!(out.failed.is_empty());
    assert!(fx.a.join("Faturalar/Mayıs/a.pdf").exists() && fx.a.join("Faturalar/Mayıs/b.pdf").exists());
    assert!(!Path::new(&p1).exists() && !Path::new(&p2).exists());
    let record = fx.app.history.lock().unwrap()[0].clone();
    assert_eq!(record.tag, "Seçilenleri taşı · Mayıs");
    assert_eq!(record.moves.len(), 2);
    crate::mover::undo_move(&record).unwrap();
    assert!(Path::new(&p1).exists() && Path::new(&p2).exists());
}

#[test]
fn apply_refuses_changed_file() {
    let fx = Fx::new();
    let p = fx.file(&fx.a, "a.pdf", b"a");
    let (plan, stamps) = plan_selection_move(&fx.req(vec![p.clone()], &fx.a.join("Faturalar"), "Mayıs")).unwrap();
    fs::write(&p, b"degisti, daha uzun").unwrap();
    let err = apply_selection_core(&fx.app, &plan, &stamps).unwrap_err();
    assert_eq!(err, "Dosya veya hedef değişti; yeniden ön izleyin.");
    assert!(Path::new(&p).exists());
    assert!(!fx.a.join("Faturalar/Mayıs").exists());
    assert!(fx.app.history.lock().unwrap().is_empty());
}

#[test]
fn rejects_quarantine_named_group_segment() {
    let fx = Fx::new();
    let inside = fx.file(&fx.a, "q.pdf", b"q");
    let group = format!("{}/Alt", QUARANTINE_DIR.to_lowercase());
    let err = plan_selection_move(&fx.req(vec![inside], &fx.a, &group)).err().expect("karantina adli grup reddedilmeli");
    assert!(err.contains(QUARANTINE_DIR), "{err}");
}

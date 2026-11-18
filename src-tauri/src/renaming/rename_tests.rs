use super::*;
use crate::{hash_cache::HashCache, mover};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture { root: PathBuf, app: AppState, state: RenameState }
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("sfo_rename_{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir_all(&root).unwrap();
        Self { app: AppState { history: Mutex::new(Vec::new()), dir: root.join("history"), cancel_scan: AtomicBool::new(false), hash_cache: HashCache::new() }, root, state: RenameState::default() }
    }
    fn preview(&self) -> RenamePlan {
        let path = self.root.join("report.txt");
        fs::write(&path, b"DATA").unwrap();
        let (plan, stamps) = plan_date_rename(vec![self.root.to_string_lossy().into_owned()], vec![path.to_string_lossy().into_owned()]).unwrap();
        *self.state.pending.lock().unwrap() = Some(Pending { plan: plan.clone(), stamps, label: "Tarihle isimlendirme" });
        plan
    }
}
impl Drop for Fixture { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.root); } }

#[test]
fn applying_date_rename_can_be_undone_and_is_one_shot() {
    let fixture = Fixture::new(); let plan = fixture.preview();
    let Some(to) = &plan.entries[0].to else { return; };
    assert_eq!(apply_pending(&fixture.state, &fixture.app, plan.id.clone()).unwrap(), 1);
    assert!(PathBuf::from(to).exists());
    assert!(!PathBuf::from(&plan.entries[0].from).exists());
    assert!(apply_pending(&fixture.state, &fixture.app, plan.id).is_err());
    let record = fixture.app.history.lock().unwrap()[0].clone();
    mover::undo_move(&record).unwrap();
    assert_eq!(fs::read(&plan.entries[0].from).unwrap(), b"DATA");
    assert_eq!(history::load(&fixture.app.dir)[0].moves.len(), 1);
}

#[test]
fn changed_source_invalidates_plan_without_mutation() {
    let fixture = Fixture::new(); let plan = fixture.preview();
    fs::write(&plan.entries[0].from, b"CHANGED-LENGTH").unwrap();
    assert!(apply_pending(&fixture.state, &fixture.app, plan.id).is_err());
    assert_eq!(fs::read(&plan.entries[0].from).unwrap(), b"CHANGED-LENGTH");
    assert!(fixture.app.history.lock().unwrap().is_empty());
}

#[test]
fn newly_occupied_target_invalidates_plan_without_overwrite() {
    let fixture = Fixture::new(); let plan = fixture.preview();
    let Some(to) = &plan.entries[0].to else { return; };
    fs::write(to, b"KEEP").unwrap();
    assert!(apply_pending(&fixture.state, &fixture.app, plan.id.clone()).is_err());
    assert_eq!(fs::read(to).unwrap(), b"KEEP");
    assert!(PathBuf::from(&plan.entries[0].from).exists());
}

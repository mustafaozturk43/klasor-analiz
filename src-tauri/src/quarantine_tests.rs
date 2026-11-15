use super::*;
use crate::hash_cache::HashCache;
use crate::history_cmds::undo_entry_core;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fx { root: PathBuf, app: AppState }
impl Fx {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!("sfo_q_{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        let root = base.join("root");
        fs::create_dir_all(&root).unwrap();
        Self { app: AppState { history: Mutex::new(Vec::new()), dir: base.join("h"), cancel_scan: AtomicBool::new(false), hash_cache: HashCache::new() }, root }
    }
    fn file(&self, rel: &str, body: &[u8]) -> PathBuf {
        let p = self.root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, body).unwrap();
        p
    }
    fn s(p: &Path) -> String { p.to_string_lossy().into_owned() }
}
impl Drop for Fx { fn drop(&mut self) { let _ = fs::remove_dir_all(self.root.parent().unwrap()); } }

#[test]
fn quarantine_then_list_shows_original_and_restore_returns_it() {
    let fx = Fx::new();
    let a = fx.file("Eski/bos.txt", b"");
    let out = quarantine_core(&fx.app, &fx.root, &[Fx::s(&a)], "Boş dosyalar").unwrap();
    assert_eq!((out.done, out.failed.len()), (1, 0));
    assert!(!a.exists());

    let list = list_core(&fx.app, &fx.root).unwrap();
    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].group, "Boş dosyalar");
    assert_eq!(list.items[0].original.as_deref(), Some(Fx::s(&a).as_str()));

    let back = restore_core(&fx.app, &fx.root, &[list.items[0].path.clone()]).unwrap();
    assert_eq!(back.done, 1);
    assert!(a.exists());
    assert!(!fx.root.join(QUARANTINE_DIR).join("Boş dosyalar").exists(), "bosalan karantina klasoru temizlenir");
}

#[test]
fn restore_without_record_or_occupied_original_goes_to_group_folder() {
    let fx = Fx::new();
    let q = fx.file(&format!("{QUARANTINE_DIR}/Etiket/rapor.pdf"), b"q");
    fx.file("Etiket/rapor.pdf", b"busy");
    let out = restore_core(&fx.app, &fx.root, &[Fx::s(&q)]).unwrap();
    assert_eq!(out.done, 1);
    assert_eq!(fs::read(fx.root.join("Etiket/rapor (1).pdf")).unwrap(), b"q");
}

#[test]
fn delete_is_limited_to_quarantine() {
    let fx = Fx::new();
    let q = fx.file(&format!("{QUARANTINE_DIR}/X/a.pdf"), b"q");
    let outside = fx.file("b.pdf", b"b");
    // Testte Geri Donusum Kutusu yerine dogrudan kaldirma kullanilir.
    let send = |p: &Path| fs::remove_file(p).map_err(|e| e.to_string());
    assert!(delete_core(&fx.root, &[Fx::s(&outside)], &send).is_err());
    assert!(outside.exists());
    assert_eq!(delete_core(&fx.root, &[Fx::s(&q)], &send).unwrap(), 1);
    assert!(!q.exists());

    // "Tumunu temizle" yalnizca ana klasorun kendi karantina klasorunu gonderir.
    let q2 = fx.file(&format!("{QUARANTINE_DIR}/Y/c.pdf"), b"c");
    let sent = std::cell::RefCell::new(Vec::new());
    crate::quarantine_cmds::clear_core(&fx.root, &|p| { sent.borrow_mut().push(p.to_path_buf()); Ok(()) }).unwrap();
    assert_eq!(*sent.borrow(), vec![fx.root.join(QUARANTINE_DIR)]);
    assert!(q2.exists() && outside.exists());
    assert!(crate::quarantine_cmds::clear_core(&fx.root.join("yok"), &|_| Ok(())).is_err());
    assert!(quarantine_core(&fx.app, &fx.root, &[Fx::s(&std::env::temp_dir())], "x").is_err());
}

#[test]
fn any_history_entry_can_be_undone_unless_a_newer_one_touched_the_same_files() {
    let fx = Fx::new();
    let a = fx.file("a.txt", b"a");
    let b = fx.file("b.txt", b"b");
    quarantine_core(&fx.app, &fx.root, &[Fx::s(&a)], "Bir").unwrap();
    quarantine_core(&fx.app, &fx.root, &[Fx::s(&b)], "Iki").unwrap();
    let older = fx.app.history.lock().unwrap()[1].clone();
    undo_entry_core(&fx.app, &older.id).unwrap();
    assert!(a.exists() && !b.exists(), "eski islem tek basina geri alindi");
    assert_eq!(fx.app.history.lock().unwrap().len(), 1);

    // Ayni dosyayi once karantinaya al, sonra geri yukle: eski kaydi geri almak engellenir.
    quarantine_core(&fx.app, &fx.root, &[Fx::s(&a)], "Uc").unwrap();
    let qa = list_core(&fx.app, &fx.root).unwrap().items.into_iter().find(|i| i.group == "Uc").unwrap();
    restore_core(&fx.app, &fx.root, &[qa.path]).unwrap();
    let blocked = fx.app.history.lock().unwrap()[1].clone();
    let err = undo_entry_core(&fx.app, &blocked.id).unwrap_err();
    assert!(err.contains("Karantinadan geri yükleme"), "{err}");
}

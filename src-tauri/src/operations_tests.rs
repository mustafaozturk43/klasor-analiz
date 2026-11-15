use super::*;
use crate::hash_cache::HashCache;
use crate::inbox_rename::{plan_list_rename, ListRenameOptions, HISTORY_LABEL};
use crate::rename::{apply_pending, Pending, RenameState};
use crate::trash_ops::trash_core;
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fx {
    base: PathBuf,
    backup: PathBuf,
    day: PathBuf,
    app: AppState,
}

impl Fx {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!("sfo_ops_{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::SeqCst)));
        let _ = fs::remove_dir_all(&base);
        let backup = base.join("Yedek");
        let day = backup.join(INBOX_DIR).join("2026-11-12_Bakilacaklar");
        fs::create_dir_all(day.join("Dokümanlar")).unwrap();
        fs::create_dir_all(day.join("Görseller")).unwrap();
        fs::create_dir_all(backup.join("Faturalar")).unwrap();
        fs::create_dir_all(backup.join(QUARANTINE_DIR)).unwrap();
        let app = AppState { history: Mutex::new(Vec::new()), dir: base.join("h"), cancel_scan: AtomicBool::new(false), hash_cache: HashCache::new() };
        Self { base, backup, day, app }
    }
    fn file(&self, rel: &str, body: &[u8]) -> String {
        let p = self.day.join(rel);
        fs::write(&p, body).unwrap();
        p.to_string_lossy().into_owned()
    }
}

impl Drop for Fx {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

#[test]
fn target_folders_hide_inbox_and_quarantine() {
    let fx = Fx::new();
    fs::create_dir_all(fx.backup.join("projeler")).unwrap();
    let names: Vec<String> = target_folders(&fx.backup).into_iter().map(|t| t.name).collect();
    assert_eq!(names, vec!["Faturalar", "projeler"]);
}

#[test]
fn moves_selected_files_records_history_and_cleans_emptied_dirs() {
    let fx = Fx::new();
    fs::write(fx.backup.join("Faturalar").join("su.pdf"), b"eski").unwrap();
    let a = fx.file("Dokümanlar/su.pdf", b"yeni");
    let b = fx.file("Görseller/foto.jpg", b"img");
    let out = move_core(&fx.app, &fx.backup, &[a.clone(), b], &fx.backup.join("Faturalar"), "").unwrap();
    assert_eq!(out.moved.len(), 2);
    assert!(out.failed.is_empty());
    assert_eq!(fs::read(fx.backup.join("Faturalar").join("su_1.pdf")).unwrap(), b"yeni", "hedefte ayni ad: _1");
    assert!(!fx.day.exists(), "bosalan grup ve gun klasorleri silinir");
    assert!(fx.backup.join(INBOX_DIR).exists(), "gelen kutusu kokune dokunulmaz");
    let h = fx.app.history.lock().unwrap();
    assert_eq!(h[0].tag, "Gelen kutusundan taşıma · Faturalar");
    assert_eq!(h[0].moves.len(), 2);
}

#[test]
fn group_creates_subfolder_inside_target_and_rejects_bad_names() {
    let fx = Fx::new();
    let a = fx.file("Dokümanlar/a.pdf", b"x");
    let target = fx.backup.join("Faturalar");
    for bad in ["..\\dışarı", "a\\\\b", INBOX_DIR, "con"] {
        assert!(move_core(&fx.app, &fx.backup, std::slice::from_ref(&a), &target, bad).is_err(), "{bad}");
    }
    assert!(Path::new(&a).exists(), "reddedilen grup adinda dosya yerinde kalir");
    let out = move_core(&fx.app, &fx.backup, std::slice::from_ref(&a), &target, " 2026 / Kasım ").unwrap();
    assert_eq!(out.moved.len(), 1);
    assert!(target.join("2026").join("Kasım").join("a.pdf").exists());
}

#[test]
fn rejects_sources_outside_inbox_and_reserved_targets() {
    let fx = Fx::new();
    let a = fx.file("Dokümanlar/a.pdf", b"x");
    let outside = fx.backup.join("Faturalar").join("b.pdf");
    fs::write(&outside, b"x").unwrap();
    let target = fx.backup.join("Faturalar");
    assert!(move_core(&fx.app, &fx.backup, &[outside.to_string_lossy().into()], &target, "").is_err());
    assert!(move_core(&fx.app, &fx.backup, std::slice::from_ref(&a), &fx.backup.join(QUARANTINE_DIR), "").is_err());
    assert!(move_core(&fx.app, &fx.backup, std::slice::from_ref(&a), &fx.backup.join(INBOX_DIR), "").is_err());
    assert!(move_core(&fx.app, &fx.backup, std::slice::from_ref(&a), &fx.base, "").is_err(), "yedek disi hedef");
    assert!(Path::new(&a).exists());
    assert!(fx.app.history.lock().unwrap().is_empty(), "reddedilen islem gecmise yazilmaz");
}

#[test]
fn trash_accepts_only_inbox_files_and_reports_each_result() {
    let fx = Fx::new();
    let a = fx.file("Dokümanlar/a.pdf", b"x");
    let b = fx.file("Dokümanlar/b.pdf", b"x");
    let outside = fx.backup.join("Faturalar").join("c.pdf");
    fs::write(&outside, b"x").unwrap();
    let never = |_: &Path| -> Result<(), String> { panic!("gecersiz istekte hicbir dosyaya dokunulmamali") };
    assert!(trash_core(&fx.backup, &[a.clone(), outside.to_string_lossy().into()], &never).is_err());

    // Gercek Geri Donusum Kutusu yerine sahte gonderici: b basarisiz olur.
    let fake = |p: &Path| if p.ends_with("b.pdf") { Err("Dosya kullanımda".to_string()) } else { fs::remove_file(p).map_err(|e| e.to_string()) };
    let out = trash_core(&fx.backup, &[a.clone(), b.clone()], &fake).unwrap();
    assert_eq!(out.trashed, vec![a]);
    assert_eq!(out.failed.len(), 1);
    assert!(Path::new(&b).exists());
}

#[test]
fn list_rename_uses_group_custom_text_and_order_and_can_be_undone() {
    let fx = Fx::new();
    let a = fx.file("Dokümanlar/Elektrik.pdf", b"1");
    let b = fx.file("Görseller/IMG_1.jpg", b"2");
    let c = fx.file("serbest.txt", b"3");
    let opts = ListRenameOptions { template: "{OzelMetin}_{Grup}_{sıra}".into(), custom_text: " Kasim ".into(), start: 1, pad: 2, ..Default::default() };
    let inbox = fx.backup.join(INBOX_DIR);
    let (plan, stamps) = plan_list_rename(&inbox, &[b.clone(), a.clone(), c.clone()], &opts).unwrap();
    let names: Vec<String> = plan.entries.iter().map(|e| Path::new(e.to.as_deref().unwrap()).file_name().unwrap().to_string_lossy().into_owned()).collect();
    assert_eq!(names, vec!["Kasim_Görseller_01.jpg", "Kasim_Dokümanlar_02.pdf", "Kasim__03.txt"]);

    let state = RenameState::default();
    *state.pending.lock().unwrap() = Some(Pending { plan: plan.clone(), stamps, label: HISTORY_LABEL });
    assert_eq!(apply_pending(&state, &fx.app, plan.id.clone()).unwrap(), 3);
    assert!(fx.day.join("Dokümanlar").join("Kasim_Dokümanlar_02.pdf").exists());
    assert_eq!(fx.app.history.lock().unwrap()[0].tag, HISTORY_LABEL);

    let id = fx.app.history.lock().unwrap()[0].id.clone();
    crate::history_cmds::undo_entry_core(&fx.app, &id).unwrap();
    assert!(Path::new(&a).exists() && Path::new(&b).exists() && Path::new(&c).exists(), "geri alinir");

    assert!(plan_list_rename(&inbox, &[a.clone(), a.clone()], &opts).is_err(), "ayni dosya iki kez");
    let bad = ListRenameOptions { template: "{bilinmeyen}".into(), ..opts };
    assert!(plan_list_rename(&inbox, &[a], &bad).is_err());
}

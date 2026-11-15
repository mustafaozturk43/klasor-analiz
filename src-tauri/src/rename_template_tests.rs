use super::*;
use crate::hash_cache::HashCache;
use crate::rename::apply_pending;
use crate::{mover, AppState};
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn fields(stem: &str, n: u64) -> Fields<'_> {
    // 14-11-2023 12:13 UTC: tarih yerel saatle yazildigi icin gun ortasi secildi (saat dilimine dayaniklidir).
    Fields { stem, n, pad: 3, mtime: 1_699_964_000, folder: "Muhasebe", group: "Dokümanlar", custom: "Kasım" }
}

#[test]
fn renders_tokens_and_cleans_bad_characters() {
    assert_eq!(render("Fatura_{yıl}-{ay}_{sıra}", &fields("x", 7)).unwrap(), "Fatura_2023-11_007");
    assert_eq!(render("{klasör} {ad} {tarih}", &fields("rapor", 1)).unwrap(), "Muhasebe rapor 2023-11-14");
    assert_eq!(render("{AD}/{gun}.", &fields("a", 1)).unwrap(), "a_14");
    assert!(render("{bilinmeyen}", &fields("a", 1)).unwrap_err().contains("{bilinmeyen}"));
    assert!(render("Fatura_{yıl", &fields("a", 1)).is_err());
}

fn opts(template: &str) -> TemplateOptions {
    TemplateOptions { template: template.into(), start: 1, pad: 2, sort: "date".into(), extensions: vec!["pdf".into()], include_subdirs: true, exclude_folders: Vec::new() }
}

#[test]
fn plans_by_date_numbers_conflicts_skips_unchanged_and_applies_with_undo() {
    let root = std::env::temp_dir().join(format!("sfo_tpl_{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("alt")).unwrap();
    let mk = |rel: &str, secs: u64| {
        let p = root.join(rel);
        fs::write(&p, rel.as_bytes()).unwrap();
        fs::File::options().write(true).open(&p).unwrap().set_modified(UNIX_EPOCH + Duration::from_secs(secs)).unwrap();
        p
    };
    let b = mk("b.pdf", 2_000_000);
    let a = mk("a.pdf", 1_000_000);
    mk("alt/c.pdf", 3_000_000);
    mk("not.txt", 1);
    mk("Belge_03.pdf", 4_000_000); // b'nin hedefini dolu tutar
    mk("Belge_01 (x).pdf", 5); // en eski: sira 01

    let (plan, stamps) = plan_template_rename(&root, &opts("Belge_{sıra}")).unwrap();
    let to = |p: &PathBuf| plan.entries.iter().find(|e| e.from == p.to_string_lossy()).unwrap().to.clone();
    assert_eq!(plan.entries.len(), 5, "txt filtrelenir");
    assert_eq!(to(&root.join("Belge_01 (x).pdf")).as_deref(), Some(root.join("Belge_01.pdf").to_string_lossy().as_ref()), "en eski dosya 01 alir");
    assert_eq!(to(&a).as_deref(), Some(root.join("Belge_02.pdf").to_string_lossy().as_ref()));
    assert_eq!(to(&b).as_deref(), Some(root.join("Belge_03 (1).pdf").to_string_lossy().as_ref()), "Belge_03.pdf dolu");
    let b_entry = plan.entries.iter().find(|e| e.from == b.to_string_lossy()).unwrap();
    assert_eq!(b_entry.warning.as_deref(), Some("Bu ad dolu; numara eklendi."));

    let app = AppState { history: Mutex::new(Vec::new()), dir: root.join("..").join(format!("sfo_tpl_h_{}", std::process::id())), cancel_scan: AtomicBool::new(false), hash_cache: HashCache::new() };
    let state = RenameState::default();
    *state.pending.lock().unwrap() = Some(Pending { plan: plan.clone(), stamps, label: "Şablonla adlandırma" });
    let renamed = apply_pending(&state, &app, plan.id.clone()).unwrap();
    assert_eq!(renamed, 5);
    assert!(root.join("Belge_02.pdf").exists() && !a.exists());
    let record = app.history.lock().unwrap()[0].clone();
    assert_eq!(record.tag, "Şablonla adlandırma");
    mover::undo_move(&record).unwrap();
    assert!(a.exists() && b.exists());
    let _ = fs::remove_dir_all(&app.dir);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn unchanged_and_case_only_names_are_skipped() {
    let root = std::env::temp_dir().join(format!("sfo_tpl_u_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("rapor.pdf"), b"r").unwrap();
    let (same, _) = plan_template_rename(&root, &opts("{ad}")).unwrap();
    assert!(same.entries[0].to.is_none() && same.entries[0].warning.as_deref().unwrap().contains("değişmiyor"));
    let (case, _) = plan_template_rename(&root, &opts("RAPOR")).unwrap();
    assert!(case.entries[0].to.is_none() && case.entries[0].warning.as_deref().unwrap().contains("büyük/küçük"));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn empty_template_and_no_matching_files_are_reported() {
    let root = std::env::temp_dir().join(format!("sfo_tpl_e_{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    assert!(plan_template_rename(&root, &opts("  ")).is_err());
    assert!(plan_template_rename(&root, &opts("x_{sıra}")).err().unwrap().contains("bulunamadı"));
    fs::remove_dir_all(&root).unwrap();
}

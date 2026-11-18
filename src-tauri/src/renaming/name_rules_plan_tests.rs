use super::*;
use crate::hash_cache::HashCache;
use crate::name_rules_apply::{apply_plan, undo, HISTORY_LABEL};
use crate::AppState;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn opts() -> NameRuleOptions {
    NameRuleOptions {
        scope: "both".into(), include_subdirs: true, case: "title".into(), space: "dash".into(), ascii: false, clean: false,
        seq_dirs: false, seq_files: false, start: 1, pad: 2, date: "none".into(), max_len: 0, exclude_folders: Vec::new(), custom_names: Default::default(),
    }
}

/// Bos bir gecici kok; `files` icindeki goreli yollar olusturulur (`/` ile biten klasordur).
fn tree(files: &[&str]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("sfo_nr_{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    for rel in files {
        let path = root.join(rel);
        if rel.ends_with('/') {
            fs::create_dir_all(&path).unwrap();
        } else {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, rel.as_bytes()).unwrap();
        }
    }
    root
}

fn app(root: &Path) -> AppState {
    let dir = root.with_file_name(format!("{}_h", root.file_name().unwrap().to_string_lossy()));
    AppState { history: Mutex::new(Vec::new()), dir, cancel_scan: AtomicBool::new(false), hash_cache: HashCache::new() }
}

/// Goreli `from` yolu icin planlanan yeni ad (yalnizca son parca).
fn new_name(plan: &NamePlan, root: &Path, rel: &str) -> Option<String> {
    // Path karsilastirmasi ayiricidan bagimsizdir (`/` ile yazilan goreli yol, diskten gelen `\` ile eslesir).
    let entry = plan.entries.iter().find(|e| Path::new(&e.from) == root.join(rel)).unwrap_or_else(|| panic!("{rel} planda yok"));
    entry.to.as_ref().map(|to| Path::new(to).file_name().unwrap().to_string_lossy().into_owned())
}

fn names(dir: &Path) -> Vec<String> {
    let mut list: Vec<String> = fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    list.sort();
    list
}

#[test]
fn plans_folders_and_files_keeps_extensions_and_numbers_folders() {
    let root = tree(&["müşteri sözleşmeleri/acme son.docx", "FATURALAR 2026/fatura ocak.PDF", "yeni klasör/", "okuma listesi.txt", "Hazır-Ad.txt"]);
    let o = NameRuleOptions { seq_dirs: true, ..opts() };
    let plan = plan_name_rules(&root, &o, &[]).unwrap();
    assert_eq!(new_name(&plan, &root, "FATURALAR 2026").as_deref(), Some("01-Faturalar-2026"));
    assert_eq!(new_name(&plan, &root, "müşteri sözleşmeleri").as_deref(), Some("02-Müşteri-Sözleşmeleri"));
    assert_eq!(new_name(&plan, &root, "yeni klasör").as_deref(), Some("03-Yeni-Klasör"));
    assert_eq!(new_name(&plan, &root, "FATURALAR 2026/fatura ocak.PDF").as_deref(), Some("Fatura-Ocak.PDF"), "uzanti korunur");
    assert_eq!(new_name(&plan, &root, "okuma listesi.txt").as_deref(), Some("Okuma-Listesi.txt"));
    assert_eq!(new_name(&plan, &root, "Hazır-Ad.txt"), None, "ad degismiyor");
    assert!(plan.entries.iter().filter(|e| e.is_dir).count() == 3 && plan.entries.len() == 7);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn scope_and_subdir_switches_limit_the_plan() {
    let root = tree(&["alt klasör/iç dosya.txt", "üst dosya.txt"]);
    let dirs = plan_name_rules(&root, &NameRuleOptions { scope: "dirs".into(), ..opts() }, &[]).unwrap();
    assert_eq!(new_name(&dirs, &root, "alt klasör").as_deref(), Some("Alt-Klasör"));
    assert_eq!(new_name(&dirs, &root, "üst dosya.txt"), None);
    let flat = plan_name_rules(&root, &NameRuleOptions { scope: "files".into(), include_subdirs: false, ..opts() }, &[]).unwrap();
    assert_eq!(flat.entries.len(), 2, "alt klasorun icine girilmez");
    assert_eq!(new_name(&flat, &root, "alt klasör"), None);
    assert_eq!(new_name(&flat, &root, "üst dosya.txt").as_deref(), Some("Üst-Dosya.txt"));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn conflicts_are_numbered_and_nothing_takes_an_existing_name() {
    let root = tree(&["rapor son.txt", "Rapor-Son.txt", "RAPOR  SON.txt"]);
    let plan = plan_name_rules(&root, &opts(), &[]).unwrap();
    let mut targets: Vec<String> = ["rapor son.txt", "RAPOR  SON.txt"].iter().map(|r| new_name(&plan, &root, r).unwrap()).collect();
    targets.sort();
    assert_eq!(targets, ["Rapor-Son (1).txt", "Rapor-Son (2).txt"], "mevcut Rapor-Son.txt yerinde kalir");
    assert!(plan.entries.iter().filter(|e| e.to.is_some()).all(|e| e.warning.as_deref() == Some("Bu ad dolu; numara eklendi.")));
    apply_plan(plan, &app(&root)).unwrap();
    assert_eq!(names(&root), ["Rapor-Son (1).txt", "Rapor-Son (2).txt", "Rapor-Son.txt"]);
    assert_eq!(fs::read_to_string(root.join("Rapor-Son.txt")).unwrap(), "Rapor-Son.txt", "ustune yazilmadi");
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn applies_nested_renames_and_undo_restores_every_name() {
    let root = tree(&["eski klasör/iç klasör/derin dosya.txt", "eski klasör/not.txt", "rapor.TXT", "boş klasör/"]);
    let before = (names(&root), names(&root.join("eski klasör")));
    let plan = plan_name_rules(&root, &NameRuleOptions { seq_dirs: true, ..opts() }, &[]).unwrap();
    let state = app(&root);
    assert_eq!(apply_plan(plan, &state).unwrap(), 6);
    assert_eq!(names(&root), ["01-Boş-Klasör", "02-Eski-Klasör", "Rapor.TXT"]);
    assert_eq!(names(&root.join("02-Eski-Klasör")), ["01-İç-Klasör", "Not.txt"]);
    assert!(root.join("02-Eski-Klasör/01-İç-Klasör/Derin-Dosya.txt").is_file());

    let record = state.history.lock().unwrap()[0].clone();
    assert_eq!((record.tag.as_str(), record.moves.len()), (HISTORY_LABEL, 6));
    undo(&record).unwrap();
    assert_eq!((names(&root), names(&root.join("eski klasör"))), before, "bos klasor dahil hepsi geri geldi");
    assert!(root.join("eski klasör/iç klasör/derin dosya.txt").is_file());
    let _ = fs::remove_dir_all(&state.dir);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn case_only_change_is_applied_and_undone() {
    let root = tree(&["rapor.txt", "klasör/"]);
    let plan = plan_name_rules(&root, &opts(), &[]).unwrap();
    assert_eq!(new_name(&plan, &root, "rapor.txt").as_deref(), Some("Rapor.txt"));
    assert!(plan.entries.iter().all(|e| e.warning.is_none()), "kendi adiyla cakisma sayilmaz");
    let state = app(&root);
    apply_plan(plan, &state).unwrap();
    assert_eq!(names(&root), ["Klasör", "Rapor.txt"]);
    undo(&state.history.lock().unwrap()[0]).unwrap();
    assert_eq!(names(&root), ["klasör", "rapor.txt"]);
    let _ = fs::remove_dir_all(&state.dir);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn protected_project_quarantine_and_hidden_items_are_left_alone() {
    let root = tree(&[
        "izlenen kaynak/yeni dosya.txt", "proje klasörü/package.json", "proje klasörü/src dosya.ts", "_Kopyalar_Karantina/etiket/kopya dosya.txt",
        "_GelenKutusu/bekleyen dosya.txt", ".gizli klasör/iç dosya.txt", "özel/önbellek/paket dosya.js", "~$geçici belge.docx", "serbest dosya.txt",
    ]);
    let o = NameRuleOptions { exclude_folders: vec!["Önbellek".into()], ..opts() };
    let watched = root.join("izlenen kaynak/alt").to_string_lossy().into_owned();
    let plan = plan_name_rules(&root, &o, &[watched]).unwrap();
    let changed: Vec<String> = plan.entries.iter().filter_map(|e| e.to.clone()).collect();
    assert_eq!(changed, [root.join("Özel").to_string_lossy().into_owned(), root.join("Serbest-Dosya.txt").to_string_lossy().into_owned()]);
    let warning = |rel: &str| plan.entries.iter().find(|e| Path::new(&e.from) == root.join(rel)).unwrap().warning.clone().unwrap();
    assert!(warning("izlenen kaynak").contains("kayıtlı"), "korunan yolun ust klasoru de korunur");
    assert!(warning("proje klasörü").contains("Proje"));
    let is_guarded = |rel: &str| plan.entries.iter().find(|e| Path::new(&e.from) == root.join(rel)).unwrap().guarded;
    assert!(is_guarded("proje klasörü") && is_guarded("izlenen kaynak") && !is_guarded("serbest dosya.txt"));
    assert_eq!(plan.entries.len(), 4, "karantina, gelen kutusu, gizli, haric ve gecici ogeler listelenmez; korunanlarin icine girilmez");

    let inside = plan_name_rules(&root.join("izlenen kaynak"), &opts(), &[root.to_string_lossy().into_owned()]);
    assert!(inside.unwrap_err().contains("otomasyon"), "izlenen klasorun icinde plan yapilmaz");
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn stale_plan_and_reserved_or_too_long_names_are_refused() {
    let root = tree(&["con.txt", "nul/", "eski ad.txt", &format!("{}.txt", "uzun ad ".repeat(30))]);
    let plan = plan_name_rules(&root, &NameRuleOptions { case: "upper".into(), ..opts() }, &[]).unwrap();
    assert!(plan.entries.iter().find(|e| e.from.ends_with("con.txt")).unwrap().warning.as_ref().unwrap().contains("ayırdığı"));
    assert!(plan.entries.iter().find(|e| e.is_dir).unwrap().to.is_none());
    assert!(plan.entries.iter().any(|e| e.warning.as_deref().is_some_and(|w| w.contains("260"))) || root.to_string_lossy().len() < 20);

    fs::write(root.join("ESKİ-AD.txt"), b"sonradan geldi").unwrap();
    fs::remove_file(root.join("eski ad.txt")).unwrap();
    let state = app(&root);
    assert!(apply_plan(plan, &state).unwrap_err().contains("yeniden ön izleyin"));
    assert!(state.history.lock().unwrap().is_empty(), "hicbir seye dokunulmadi");
    assert!(root.join("con.txt").exists());
    fs::remove_dir_all(&root).unwrap();
}

fn with_custom(root: &Path, pairs: &[(&str, &str)], base: NameRuleOptions) -> NameRuleOptions {
    let custom_names = pairs.iter().map(|(rel, text)| (root.join(rel).to_string_lossy().into_owned(), text.to_string())).collect();
    NameRuleOptions { custom_names, ..base }
}

#[test]
fn custom_name_replaces_rule_result_keeps_extension_and_marks_entry() {
    let root = tree(&["Rapor Ocak.pdf", "diger dosya.pdf"]);
    let o = with_custom(&root, &[("Rapor Ocak.pdf", "  Yıllık   Özet: 2026? ")], opts());
    let plan = plan_name_rules(&root, &o, &[]).unwrap();
    assert_eq!(new_name(&plan, &root, "Rapor Ocak.pdf").as_deref(), Some("Yıllık Özet 2026.pdf"), "gecersiz karakterler atilir, uzanti korunur");
    assert_eq!(new_name(&plan, &root, "diger dosya.pdf").as_deref(), Some("Diger-Dosya.pdf"), "digerleri kurallarla");
    let by = |n: &str| plan.entries.iter().find(|e| e.from.ends_with(n)).unwrap();
    assert!(by("Rapor Ocak.pdf").custom && !by("diger dosya.pdf").custom);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn custom_name_does_not_consume_sequence_numbers_and_wins_over_scope() {
    let root = tree(&["a.txt", "b.txt", "c.txt"]);
    let base = NameRuleOptions { seq_files: true, ..opts() };
    let plan = plan_name_rules(&root, &with_custom(&root, &[("b.txt", "Ozel")], base.clone()), &[]).unwrap();
    assert_eq!(new_name(&plan, &root, "a.txt").as_deref(), Some("01-A.txt"));
    assert_eq!(new_name(&plan, &root, "b.txt").as_deref(), Some("Ozel.txt"));
    assert_eq!(new_name(&plan, &root, "c.txt").as_deref(), Some("02-C.txt"), "ozel ad numara harcamaz");
    // Kapsam "yalniz klasorler" olsa da acikca ozel ad yazilan dosya adlandirilir.
    let dirs_only = NameRuleOptions { scope: "dirs".into(), ..base };
    let plan = plan_name_rules(&root, &with_custom(&root, &[("c.txt", "Elle")], dirs_only), &[]).unwrap();
    assert_eq!(new_name(&plan, &root, "c.txt").as_deref(), Some("Elle.txt"));
    assert!(new_name(&plan, &root, "a.txt").is_none(), "ozel adsiz dosya kapsam disinda kalir");
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn custom_name_conflicts_are_numbered_and_unusable_text_is_skipped() {
    let root = tree(&["a.txt", "Hedef.txt"]);
    let plan = plan_name_rules(&root, &with_custom(&root, &[("a.txt", "hedef")], opts()), &[]).unwrap();
    assert_eq!(new_name(&plan, &root, "a.txt").as_deref(), Some("hedef (1).txt"), "dolu ad numaralanir, ustune yazilmaz");
    let plan = plan_name_rules(&root, &with_custom(&root, &[("a.txt", "???")], opts()), &[]).unwrap();
    let e = plan.entries.iter().find(|e| e.from.ends_with("a.txt")).unwrap();
    assert!(e.to.is_none() && e.warning.as_ref().unwrap().contains("Özel ad"), "{:?}", e.warning);
    let plan = plan_name_rules(&root, &with_custom(&root, &[("a.txt", "   ")], opts()), &[]).unwrap();
    assert!(!plan.entries.iter().find(|e| e.from.ends_with("a.txt")).unwrap().custom, "bos metin ozel ad sayilmaz");
    fs::remove_dir_all(&root).unwrap();
}

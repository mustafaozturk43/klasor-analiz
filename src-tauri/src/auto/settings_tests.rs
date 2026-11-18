use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn tmp() -> PathBuf {
    let d = std::env::temp_dir().join(format!("sfo_settings_{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::SeqCst)));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn s(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

#[test]
fn defaults_are_valid_and_map_extensions_case_insensitively() {
    let st = AppSettings::default().validated().unwrap();
    assert_eq!(st.ext_groups.len(), 5);
    assert_eq!(st.group_of("PDF"), Some("Dokümanlar"));
    assert_eq!(st.group_of(".HEIC"), Some("Görseller"));
    assert_eq!(st.group_of("mkv"), Some("Videolar"));
    assert_eq!(st.group_of("xyz"), None, "Diğer kapaliyken eslesmeyen dosya yerinde kalir");
    let other = AppSettings { include_other: true, ..AppSettings::default() };
    assert_eq!(other.group_of("xyz"), Some(OTHER_GROUP));
    assert!(!st.automation_enabled, "otomasyon kapali baslar");
}

#[test]
fn normalizes_extensions_and_rejects_shared_or_bad_ones() {
    let st = AppSettings {
        ext_groups: vec![ExtGroup { name: " Belgeler ".into(), extensions: vec![".PDF".into(), "pdf".into(), " Txt ".into(), "".into()] }],
        ..AppSettings::default()
    };
    let ok = st.validated().unwrap();
    assert_eq!(ok.ext_groups[0].name, "Belgeler");
    assert_eq!(ok.ext_groups[0].extensions, vec!["pdf", "txt"]);

    let mut dup = AppSettings::default();
    dup.ext_groups[4].extensions.push("PDF".into());
    let err = dup.validated().unwrap_err();
    assert!(err.contains(".pdf") && err.contains("Dokümanlar"), "{err}");

    let mut bad = AppSettings::default();
    bad.ext_groups[0].extensions.push("tar.gz".into());
    assert!(bad.validated().is_err());

    let mut empty = AppSettings::default();
    empty.ext_groups[0].extensions.clear();
    assert!(empty.validated().is_err());
}

#[test]
fn rejects_reserved_duplicate_and_invalid_group_names() {
    for name in ["_GelenKutusu", "_kopyalar_karantina", "Diğer", "A/B", ""] {
        let mut st = AppSettings::default();
        st.ext_groups[0].name = name.into();
        assert!(st.validated().is_err(), "{name} reddedilmeli");
    }
    let mut st = AppSettings::default();
    st.ext_groups[1].name = "dokümanlar".into();
    assert!(st.validated().is_err(), "ayni ad iki grupta olamaz");
}

#[test]
fn backup_and_source_cannot_be_nested_and_sources_are_deduplicated() {
    let base = tmp();
    let src = base.join("Downloads");
    let backup = base.join("Yedek");
    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&backup).unwrap();

    let mut st = AppSettings { source_folders: vec![s(&src), format!("{}\\", s(&src).to_uppercase()), "  ".into()], backup_root: s(&backup), ..AppSettings::default() };
    st = st.validated().unwrap();
    assert_eq!(st.source_folders, vec![s(&src)]);

    for (source, backup_root) in [(s(&src), s(&src.join("Yedek"))), (s(&backup.join("ic")), s(&backup)), (s(&src), s(&src))] {
        let st = AppSettings { source_folders: vec![source.clone()], backup_root: backup_root.clone(), ..AppSettings::default() };
        assert!(st.validated().is_err(), "{source} / {backup_root} ic ice");
    }
    fs::remove_dir_all(&base).unwrap();
}

#[test]
fn automation_needs_source_and_backup() {
    let st = AppSettings { automation_enabled: true, ..AppSettings::default() };
    assert!(st.validated().unwrap_err().contains("en az bir kaynak"));
}

#[test]
fn save_and_load_round_trip_and_tolerate_missing_or_corrupt_file() {
    let dir = tmp();
    assert_eq!(load(&dir), AppSettings::default());
    let saved = save(&dir, AppSettings { source_folders: vec![s(&dir.join("A"))], backup_root: s(&dir.join("B")), ..AppSettings::default() }).unwrap();
    assert_eq!(load(&dir), saved);
    assert!(!dir.join("settings.json.tmp").exists());

    fs::write(dir.join("settings.json"), "{bozuk").unwrap();
    assert_eq!(load(&dir), AppSettings::default());
    assert_eq!(fs::read_to_string(dir.join("settings.json.bozuk")).unwrap(), "{bozuk", "bozuk dosya yedeklenir");
    fs::write(dir.join("settings.json"), r#"{"backupRoot":"D:\\Yedek"}"#).unwrap();
    let partial = load(&dir);
    assert_eq!(partial.backup_root, r"D:\Yedek");
    assert_eq!(partial.ext_groups.len(), 5, "eksik alanlar varsayilanla dolar");

    assert!(save(&dir, AppSettings { automation_enabled: true, ..AppSettings::default() }).is_err());
    assert_eq!(load(&dir).backup_root, r"D:\Yedek", "gecersiz ayar eski kaydi bozmaz");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn ui_prefs_live_in_settings_json_and_survive_reload() {
    use crate::settings_cmds::set_pref;
    let dir = tmp();
    let mut current = load(&dir);
    set_pref(&mut current, &dir, "sfo-theme".into(), serde_json::json!("dark")).unwrap();
    set_pref(&mut current, &dir, "sfo-recent".into(), serde_json::json!([r"C:\A", r"D:\B"])).unwrap();
    assert_eq!(load(&dir).ui_prefs.len(), 2);
    assert_eq!(load(&dir).ui_prefs["sfo-theme"], "dark");

    set_pref(&mut current, &dir, "sfo-theme".into(), serde_json::Value::Null).unwrap();
    assert!(!load(&dir).ui_prefs.contains_key("sfo-theme"), "null deger anahtari siler");
    assert!(set_pref(&mut current, &dir, String::new(), serde_json::json!(1)).is_err());
    assert_eq!(load(&dir).ext_groups.len(), 5, "tercih yazmak diger ayarlari bozmaz");
    fs::remove_dir_all(&dir).unwrap();
}

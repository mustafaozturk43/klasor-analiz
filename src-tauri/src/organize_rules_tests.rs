use super::*;

fn rule(category: &str, kind: &str, exts: &[&str]) -> RuleDef {
    RuleDef { category: category.into(), kind: kind.into(), extensions: exts.iter().map(|e| e.to_string()).collect() }
}

#[test]
fn default_rules_map_extensions_to_two_level_folders() {
    let set = RuleSet::new(&[]).unwrap();
    assert_eq!(set.target_of("xlsx", false), Some((DOCS.into(), "Excel".into())));
    assert_eq!(set.target_of("jpeg", false), Some((IMAGES.into(), "JPG".into())));
    assert_eq!(set.target_of("gz", false), Some((ARCHIVES.into(), "TAR".into())));
    assert_eq!(set.target_of("mp4", false), None);
    assert_eq!(set.target_of("mp4", true), Some((OTHERS.into(), "MP4".into())));
    assert_eq!(set.target_of("", true), Some((OTHERS.into(), "Uzantısız".into())));
}

#[test]
fn custom_rules_replace_defaults_and_normalize_extensions() {
    let set = RuleSet::new(&[rule("Videolar", "MP4", &[".MP4", " mov "])]).unwrap();
    assert_eq!(set.target_of("mov", false), Some(("Videolar".into(), "MP4".into())));
    assert_eq!(set.target_of("pdf", false), None);
}

#[test]
fn invalid_rules_are_rejected_with_reason() {
    let dup = RuleSet::new(&[rule("A", "Excel", &["xlsx"]), rule("A", "Tablo", &["XLSX"])]);
    assert!(dup.err().unwrap().contains(".xlsx"));
    assert!(RuleSet::new(&[rule("", "PDF", &["pdf"])]).is_err());
    assert!(RuleSet::new(&[rule("A", "P/DF", &["pdf"])]).is_err());
    assert!(RuleSet::new(&[rule("A", "PDF", &[" "])]).is_err());
    assert!(RuleSet::new(&[rule("_kopyalar_karantina", "PDF", &["pdf"])]).is_err());
}

#[test]
fn year_folder_uses_local_year_or_undated() {
    assert_eq!(year_folder(1_700_000_000), "2023");
    assert_eq!(year_folder(0), "Tarihsiz");
}

#[test]
fn detects_temp_files_and_project_markers() {
    assert!(is_temp_or_system("~$sözleşme.docx"));
    assert!(is_temp_or_system("Thumbs.db"));
    assert!(!is_temp_or_system("rapor.pdf"));
    assert!(has_project_marker(["src", "Package.json"]));
    assert!(has_project_marker(["App.sln"]));
    assert!(!has_project_marker(["Rapor.pdf", "foto.jpg"]));
}

use super::*;

// 14-11-2023 12:13 UTC: tarih yerel saatle yazildigi icin gun ortasi secildi.
const MTIME: u64 = 1_699_964_000;

pub(crate) fn opts() -> NameRuleOptions {
    NameRuleOptions {
        scope: "both".into(), include_subdirs: true, case: "title".into(), space: "dash".into(), ascii: false, clean: false,
        seq_dirs: false, seq_files: false, start: 1, pad: 2, date: "none".into(), max_len: 0, exclude_folders: Vec::new(),
    }
}

fn run(stem: &str, o: &NameRuleOptions, seq: Option<u64>) -> String {
    transform(stem, o, seq, MTIME).unwrap()
}

#[test]
fn title_case_follows_turkish_letter_rules() {
    let o = opts();
    assert_eq!(run("IŞIK ölçüm sonuçları", &o, None), "Işık-Ölçüm-Sonuçları");
    assert_eq!(run("istanbul ığdır", &o, None), "İstanbul-Iğdır");
    assert_eq!(run("  çift   boşluk  ", &o, None), "Çift-Boşluk");
}

#[test]
fn ascii_mode_drops_turkish_letters_and_uses_plain_latin_case() {
    let o = NameRuleOptions { ascii: true, case: "lower".into(), ..opts() };
    assert_eq!(run("IŞIK Ölçüm İstanbul", &o, None), "isik-olcum-istanbul");
    let o = NameRuleOptions { ascii: true, case: "upper".into(), space: "underscore".into(), ..opts() };
    assert_eq!(run("çalışma ığdır", &o, None), "CALISMA_IGDIR");
}

#[test]
fn case_modes_and_space_modes() {
    let o = NameRuleOptions { case: "sentence".into(), space: "keep".into(), ..opts() };
    assert_eq!(run("FATURA ocak", &o, None), "Fatura ocak");
    let o = NameRuleOptions { case: "keep".into(), space: "none".into(), ..opts() };
    assert_eq!(run("Fatura OCAK son", &o, None), "FaturaOCAKson");
    let o = NameRuleOptions { case: "upper".into(), ..opts() };
    assert_eq!(run("fatura ilk", &o, None), "FATURA-İLK");
}

#[test]
fn sequence_prefix_and_date_suffix_use_the_chosen_separator() {
    let o = NameRuleOptions { space: "underscore".into(), date: "my".into(), ..opts() };
    assert_eq!(run("müşteri sözleşmeleri", &o, Some(2)), "02_Müşteri_Sözleşmeleri_11-2023");
    let o = NameRuleOptions { date: "ym".into(), pad: 3, ..opts() };
    assert_eq!(run("rapor", &o, Some(7)), "007-Rapor-2023-11");
    let o = NameRuleOptions { space: "keep".into(), date: "my".into(), ..opts() };
    assert_eq!(run("rapor", &o, Some(1)), "01 Rapor 11-2023");
}

#[test]
fn applying_twice_does_not_stack_numbers_or_dates() {
    let o = NameRuleOptions { date: "my".into(), ..opts() };
    let once = run("toplantı notları", &o, Some(3));
    assert_eq!(once, "03-Toplantı-Notları-11-2023");
    assert_eq!(run(&once, &o, Some(3)), once);
    // Yil gibi dort basamak sira numarasi sayilmaz.
    assert_eq!(run("2026 faturalar", &opts(), Some(1)), "01-2026-Faturalar");
    // Numaralandirma kapaliyken bastaki sayi korunur.
    assert_eq!(run("01 giriş", &opts(), None), "01-Giriş");
}

#[test]
fn length_limit_shortens_the_body_but_keeps_number_and_date() {
    let o = NameRuleOptions { space: "underscore".into(), date: "my".into(), max_len: 20, ..opts() };
    let out = run("çalışma planı ığdır şubesi uzun başlık", &o, Some(1));
    assert_eq!(out, "01_Çalışma_P_11-2023");
    assert!(out.chars().count() <= 20);
    let o = NameRuleOptions { max_len: 5, ..opts() };
    assert_eq!(run("abcdefghij", &o, None), "Abcde");
}

#[test]
fn cleans_punctuation_and_forbidden_characters() {
    let o = NameRuleOptions { clean: true, ..opts() };
    assert_eq!(run("acme sözleşme son (2)", &o, None), "Acme-Sözleşme-Son-2");
    assert_eq!(run("rapor: son? v2", &opts(), None), "Rapor-Son-V2");
    assert_eq!(run("nokta sonda.", &opts(), None), "Nokta-Sonda");
}

#[test]
fn empty_result_is_none_and_reserved_names_are_detected() {
    let o = NameRuleOptions { clean: true, ..opts() };
    assert_eq!(transform("(!)", &o, None, MTIME), None);
    assert_eq!(transform("???", &opts(), Some(1), MTIME), None);
    for name in ["CON", "con", "Nul.txt", "COM1", "lpt9.log"] {
        assert!(is_reserved(name), "{name}");
    }
    for name in ["Console", "COM0", "COM10", "Rapor"] {
        assert!(!is_reserved(name), "{name}");
    }
}

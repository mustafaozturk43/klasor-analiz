//! O1 similarity optimizasyonu dogruluk testleri: optimize = naif,
//! uzunluk eleme yanlis-negatif uretmez.

use smart_file_organizer_lib::models::{FileCategory, FileGroup, FileItem};
use smart_file_organizer_lib::similarity;

#[path = "support/naive.rs"]
mod naive;
use naive::cluster_naive;

fn make_item(name: &str, stem: &str) -> FileItem {
    FileItem {
        path: format!("C:\\x\\{name}"),
        name: name.to_string(),
        stem: stem.to_string(),
        extension: "txt".to_string(),
        size: 10,
        mtime: 0,
        category: FileCategory::Document,
        hash_id: String::new(),
    }
}

/// Gruplari karsilastirilabilir imzaya cevirir (sirali isim kumeleri).
fn group_signature(groups: &[FileGroup]) -> Vec<Vec<String>> {
    let mut sig: Vec<Vec<String>> = groups
        .iter()
        .map(|g| {
            let mut names: Vec<String> = g.files.iter().map(|f| f.name.clone()).collect();
            names.sort();
            names
        })
        .collect();
    sig.sort();
    sig
}

#[test]
fn optimized_cluster_matches_naive_on_realistic_names() {
    let items = vec![
        make_item("rapor_v1.txt", "rapor_v1"),
        make_item("rapor_v2.txt", "rapor_v2"),
        make_item("rapor_v3.txt", "rapor_v3"),
        // Ayni stem (tekillestirme testi): iki farkli dosya, ayni isim koku.
        make_item("rapor_v1_kopya.txt", "rapor_v1"),
        make_item("fatura_2024.txt", "fatura_2024"),
        make_item("fatura_2025.txt", "fatura_2025"),
        make_item("unique_a.txt", "zzzz"),
        make_item("unique_b.txt", "qqqq"),
    ];

    let naive = cluster_naive(items.clone(), 0.82);
    let optimized = similarity::cluster_by_similarity(items, 0.82, &|| false);

    assert_eq!(
        group_signature(&naive),
        group_signature(&optimized),
        "Optimize kumeleme naif ile ayni gruplari uretmeli"
    );
}

#[test]
fn counters_and_timestamps_are_not_versions_but_short_numbers_and_years_are() {
    assert!(similarity::is_sequence_pair("img_0001", "img_0002"));
    assert!(similarity::is_sequence_pair("scan001", "scan002"));
    assert!(similarity::is_sequence_pair("screenshot_20240101-101500", "screenshot_20240102-093011"));
    assert!(!similarity::is_sequence_pair("rapor_v1", "rapor_v2"));
    assert!(!similarity::is_sequence_pair("fatura_2024", "fatura_2025"));
    assert!(!similarity::is_sequence_pair("img_0001", "img_0001_duzenlenmis"));

    let mut items: Vec<FileItem> = (1..=40).map(|i| make_item(&format!("IMG_{i:04}.jpg"), &format!("IMG_{i:04}"))).collect();
    items.push(make_item("IMG_0007 (1).jpg", "IMG_0007 (1)"));
    items.push(make_item("rapor_v1.txt", "rapor_v1"));
    items.push(make_item("rapor_v2.txt", "rapor_v2"));
    let groups = group_signature(&similarity::cluster_by_similarity(items.clone(), 0.82, &|| false));
    assert_eq!(groups, group_signature(&cluster_naive(items, 0.82)));
    assert_eq!(groups.len(), 2, "40 fotograf tek gruba zincirlenmez: {groups:?}");
    assert!(groups.iter().all(|g| g.len() == 2));
}

#[test]
fn length_prune_never_rejects_true_matches() {
    let mut samples: Vec<String> = vec![
        "rapor", "rapor_v1", "rapor_v2_uzun", "fatura", "fatura_2024", "a", "ab", "abc",
        "abcd", "abcde", "abcdef", "resim_tatil", "resim", "dokuman_onemli_final",
        "dokuman_onemli", "rapor_v1", "fatura_2024",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    samples.push("x".repeat(50));
    samples.push("x".repeat(51));
    samples.push("rapor_cok_uzun_bir_dosya_adi_v2".to_string());

    for a in &samples {
        for b in &samples {
            let la = a.chars().count();
            let lb = b.chars().count();
            let jw = strsim::jaro_winkler(a, b);
            for &t in &[0.82f64, 0.9f64] {
                if jw >= t {
                    assert!(
                        similarity::could_match(la, lb, t),
                        "Yanlis negatif (eleme gercek eslesmeyi reddetti): '{a}' vs '{b}' jw={jw}"
                    );
                }
            }
        }
    }
}

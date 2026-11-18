use super::*;
use crate::inspector::detail_of;
use std::path::PathBuf;

fn put(dir: &Path, name: &str) -> PathBuf {
    fs::create_dir_all(dir).unwrap();
    let p = dir.join(name);
    fs::write(&p, name.as_bytes()).unwrap();
    p
}

#[test]
fn suggests_bin_for_copies_move_for_known_names_together_for_similar_and_keep_otherwise() {
    let base = std::env::temp_dir().join(format!("sfo_suggest_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    put(&base.join("Faturalar"), "Elektrik_Faturasi_Ekim.pdf");
    put(&base.join("_GelenKutusu"), "gizli.pdf"); // ayrilmis klasor hedef degildir
    let inbox = base.join("_GelenKutusu").join("2026-11-14_Bakilacaklar").join("Dokümanlar");
    let mut files: Vec<FileDetail> = ["Elektrik_Faturasi_Kasim.pdf", "notlar.txt", "Fatura (1).pdf", "Elektrik_Faturasi_Kasim_ek.pdf"]
        .iter()
        .map(|n| detail_of(&put(&inbox, n), "Dokümanlar", None).unwrap())
        .collect();
    files[2].duplicate_of = Some("D:/Yedek/Arşiv/Fatura.pdf".into());
    files[2].duplicate_in = Some("Arşiv".into());
    files[1].similar_to = Some("Elektrik_Faturasi_Kasim.pdf".into());
    files[3].similar_to = Some("Elektrik_Faturasi_Kasim.pdf".into());

    suggest(&mut files, &base);
    let s = |i: usize| files[i].suggestion.clone().unwrap();
    assert_eq!((s(0).verdict.as_str(), s(0).target.as_deref()), ("move", Some("Faturalar")));
    assert!(s(0).reason.contains("Elektrik_Faturasi_Ekim.pdf"));
    assert_eq!(s(2).verdict, "bin");
    assert_eq!(s(2).reason, "İçeriği Arşiv klasöründeki Fatura.pdf ile birebir aynı.");
    assert_eq!((s(1).verdict.as_str(), s(1).target.as_deref()), ("move", Some("Faturalar")), "benzer dosya birlikte gider");
    assert!(s(1).reason.contains("ikisi birlikte"));
    assert_eq!(s(3).verdict, "move");
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn unknown_files_are_kept() {
    let base = std::env::temp_dir().join(format!("sfo_suggest_keep_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    put(&base.join("Faturalar"), "Elektrik_Faturasi_Ekim.pdf");
    let mut files = vec![detail_of(&put(&base.join("_GelenKutusu").join("g"), "tatil.jpg"), "Görseller", None).unwrap()];
    suggest(&mut files, &base);
    let s = files[0].suggestion.clone().unwrap();
    assert_eq!(s.verdict, "keep");
    assert!(s.target.is_none());
    let _ = fs::remove_dir_all(&base);
}

use super::*;
use crate::inspector::detail_of;

fn put(dir: &Path, name: &str, body: &[u8]) -> PathBuf {
    fs::create_dir_all(dir).unwrap();
    let p = dir.join(name);
    fs::write(&p, body).unwrap();
    p
}

fn detail(p: &Path) -> FileDetail {
    detail_of(p, "G", None).unwrap()
}

fn scratch(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("sfo_dupes_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    base
}

#[test]
fn inbox_files_match_a_copy_in_the_backup_folder_but_not_a_same_size_stranger() {
    let base = scratch("folder");
    let inbox = base.join(INBOX_DIR).join("2026-11-14_Bakilacaklar").join("Arşivler");
    let same_file = put(&inbox, "yedek_2026.zip", b"PK-ayni-icerik");
    let other = put(&inbox, "baska.zip", b"PK-baska-icerik"); // ayni boyut degil, ama
    let stranger = put(&inbox, "yabanci.zip", b"PK-ayni-icerik".iter().map(|b| b ^ 1).collect::<Vec<u8>>().as_slice()); // ayni boyut, farkli icerik
    put(&base.join("Arşiv"), "eski.zip", b"PK-ayni-icerik");
    put(&base.join(QUARANTINE_DIR).join("Etiket"), "karantina.zip", b"PK-baska-icerik");

    let mut files = vec![detail(&same_file), detail(&other), detail(&stranger)];
    mark_exact(&mut files, &base);
    assert_eq!(files[0].duplicate_in.as_deref(), Some("Arşiv"));
    assert!(files[0].duplicate_of.as_deref().unwrap().ends_with("eski.zip"));
    assert!(files[1].duplicate_of.is_none(), "karantinadaki kopya sayilmaz");
    assert!(files[2].duplicate_of.is_none(), "ayni boyut ama farkli icerik");
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn identical_inbox_files_point_to_the_shortest_named_original_and_empty_files_are_ignored() {
    let base = scratch("inbox");
    let dir = base.join(INBOX_DIR).join("2026-11-14_Bakilacaklar").join("Dokümanlar");
    let orig = put(&dir, "Fatura.pdf", b"%PDF fatura");
    let copy = put(&dir, "Fatura (1).pdf", b"%PDF fatura");
    let e1 = put(&dir, "bos1.txt", b"");
    let e2 = put(&dir, "bos2.txt", b"");

    let mut files = vec![detail(&copy), detail(&orig), detail(&e1), detail(&e2)];
    mark_exact(&mut files, &base);
    assert_eq!(files[0].duplicate_of.as_deref(), Some(orig.to_string_lossy().as_ref()));
    assert!(files[0].duplicate_in.is_none(), "gelen kutusu ici kopya");
    assert!(files[1].duplicate_of.is_none(), "asil dosya isaretlenmez");
    assert!(files[2].duplicate_of.is_none() && files[3].duplicate_of.is_none(), "bos dosyalar kopya sayilmaz");
    let _ = fs::remove_dir_all(&base);
}

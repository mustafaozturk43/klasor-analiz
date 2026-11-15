use super::*;
use crate::inbox_list::{list_core, list_days, LOOSE_GROUP};
use crate::models::{MoveRecord, UndoRecord};
use crate::watcher_core::HISTORY_TAG;
use chrono::NaiveDate;
use std::path::PathBuf;

#[test]
fn preview_counts_characters_not_bytes_and_never_splits_a_letter() {
    let text = "ğüşİöç ".repeat(80); // her harf 2 bayt
    let p = text_preview(text.as_bytes(), PREVIEW_CHARS).unwrap();
    assert_eq!(p.chars().count(), PREVIEW_CHARS + 1, "300 karakter + …");
    assert!(p.ends_with('…'));

    // 1536 baytlik okuma bir harfin ortasinda bitebilir: yarim harf atilir.
    let bytes = "aaaş".as_bytes();
    let cut = &bytes[..bytes.len() - 1];
    assert_eq!(text_preview(cut, 300).unwrap(), "aaa");

    assert_eq!(text_preview(b"\xEF\xBB\xBFsatir 1\r\n\r\nsatir   2", 300).unwrap(), "satir 1 satir 2", "BOM ve fazla bosluk");
    assert_eq!(text_preview(b"ab\xFFcd", 300).unwrap(), "ab\u{FFFD}cd", "ortadaki bozuk bayt");
    assert!(text_preview(b"PK\x03\x04\x00\x00", 300).is_none(), "ikili icerik");
    assert!(text_preview(b"   ", 300).is_none());
}

fn detail(name: &str) -> FileDetail {
    FileDetail {
        path: name.into(), name: name.into(), ext: String::new(), size: 1, group: "G".into(), arrived_ms: None, created_ms: None,
        modified_ms: None, accessed_ms: None, content_preview: None, thumbnail_path: None, similar_to: None, similarity: None,
        duplicate_of: None, duplicate_in: None, suggestion: None,
    }
}

#[test]
fn similar_names_point_to_their_closest_match() {
    let mut files = vec![detail("Teklif_Q4.xlsx"), detail("Teklif_Q4_son.xlsx"), detail("foto.jpg"), detail("Kira.pdf"), detail("Kira_1.pdf")];
    mark_similar(&mut files, 0.82);
    assert_eq!(files[0].similar_to.as_deref(), Some("Teklif_Q4_son.xlsx"));
    assert!(files[0].similarity.unwrap() >= 82);
    assert_eq!(files[3].similar_to.as_deref(), Some("Kira_1.pdf"));
    assert!(files[2].similar_to.is_none());
}

#[test]
fn lists_a_day_by_group_with_arrival_times_previews_and_days() {
    let base = std::env::temp_dir().join(format!("sfo_inbox_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let day = NaiveDate::from_ymd_opt(2026, 11, 12).unwrap();
    let dir = crate::inbox_path::day_dir(&base, day);
    std::fs::create_dir_all(dir.join("Dokümanlar")).unwrap();
    std::fs::create_dir_all(dir.join("Görseller")).unwrap();
    std::fs::create_dir_all(crate::inbox_path::day_dir(&base, day.pred_opt().unwrap())).unwrap();
    std::fs::write(dir.join("Dokümanlar").join("notlar.txt"), "Toplantı notları: bütçe").unwrap();
    std::fs::write(dir.join("Dokümanlar").join("rapor.pdf"), b"%PDF").unwrap();
    std::fs::write(dir.join("serbest.bin"), b"x").unwrap();
    // 2x2 gercek PNG: kucuk gorsel oldugu gibi gosterilir.
    image::RgbImage::new(2, 2).save(dir.join("Görseller").join("a.png")).unwrap();

    let note: PathBuf = dir.join("Dokümanlar").join("notlar.txt");
    let history = vec![UndoRecord { id: "IN-1".into(), timestamp: 1_000, tag: HISTORY_TAG.into(), moves: vec![MoveRecord { from: "x".into(), to: note.to_string_lossy().into() }] }];
    let l = list_core(&base, day, &history, &base.join("cache"));

    assert!(l.exists);
    assert_eq!(l.total_files, 4);
    let names: Vec<&str> = l.groups.iter().map(|g| g.name.as_str()).collect();
    assert_eq!(names, vec!["Dokümanlar", "Görseller", LOOSE_GROUP]);
    let docs = &l.groups[0].files;
    let n = docs.iter().find(|f| f.name == "notlar.txt").unwrap();
    assert_eq!(n.arrived_ms, Some(1_000));
    assert_eq!(n.content_preview.as_deref(), Some("Toplantı notları: bütçe"));
    assert!(docs.iter().find(|f| f.name == "rapor.pdf").unwrap().content_preview.is_none(), "PDF icin ozet yok");
    let img = &l.groups[1].files[0];
    assert_eq!(img.thumbnail_path.as_deref(), Some(img.path.as_str()));

    let days = list_days(&base);
    assert_eq!(days.iter().map(|d| (d.date.as_str(), d.files)).collect::<Vec<_>>(), vec![("2026-11-12", 4), ("2026-11-11", 0)]);
    std::fs::remove_dir_all(&base).unwrap();
}

#[test]
fn large_images_get_a_cached_96px_thumbnail() {
    let base = std::env::temp_dir().join(format!("sfo_thumb_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let big = base.join("buyuk.png");
    image::RgbImage::from_fn(1200, 900, |x, y| image::Rgb([(x % 251) as u8, (y % 241) as u8, ((x * y) % 239) as u8])).save(&big).unwrap();
    let mut d = crate::inspector::detail_of(&big, "Görseller", None).unwrap();
    d.size = crate::thumbnail::DIRECT_LIMIT + 1; // buyuk dosya gibi davran
    let cache = base.join("cache");
    let out = crate::thumbnail::cached_thumbnail(&cache, &d).unwrap();
    let t = image::open(&out).unwrap();
    assert!(t.width() <= 96 && t.height() <= 96 && t.width() == 96, "en-boy orani korunur");
    let again = crate::thumbnail::cached_thumbnail(&cache, &d).unwrap();
    assert_eq!(out, again, "onbellekten doner");
    std::fs::remove_dir_all(&base).unwrap();
}

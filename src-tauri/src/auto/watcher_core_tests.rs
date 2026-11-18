use super::*;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fx {
    base: PathBuf,
    src: PathBuf,
    settings: AppSettings,
}

impl Fx {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!("sfo_watch_{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::SeqCst)));
        let _ = fs::remove_dir_all(&base);
        let src = base.join("Downloads");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(base.join("Yedek")).unwrap();
        let settings = AppSettings { source_folders: vec![src.to_string_lossy().into()], backup_root: base.join("Yedek").to_string_lossy().into(), automation_enabled: true, ..AppSettings::default() };
        Self { base, src, settings }
    }
    fn file(&self, name: &str, body: &[u8]) -> PathBuf {
        let p = self.src.join(name);
        fs::write(&p, body).unwrap();
        p
    }
}

impl Drop for Fx {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 12).unwrap()
}

#[test]
fn eligible_skips_dirs_temp_and_system_files() {
    let fx = Fx::new();
    assert!(eligible(&fx.file("fatura.pdf", b"x")));
    for name in ["film.mp4.crdownload", "kurulum.part", "~$taslak.docx", "desktop.ini", ".gizli"] {
        assert!(!eligible(&fx.file(name, b"x")), "{name}");
    }
    assert!(!eligible(&fx.src), "klasor alinmaz");
    assert!(!eligible(&fx.src.join("yok.pdf")));
}

#[test]
fn moves_to_group_folder_with_underscore_numbering_and_leaves_unmatched() {
    let fx = Fx::new();
    let a = fx.file("Kira.pdf", b"1");
    let m = move_to_inbox(&fx.settings, day(), &a).unwrap().unwrap();
    assert_eq!(m.group, "Dokümanlar");
    let inbox = fx.base.join("Yedek").join("_GelenKutusu").join("2026-11-12_Bakilacaklar").join("Dokümanlar");
    assert_eq!(PathBuf::from(&m.to), inbox.join("Kira.pdf"));
    assert!(!a.exists());

    let b = fx.file("Kira.pdf", b"2");
    let m2 = move_to_inbox(&fx.settings, day(), &b).unwrap().unwrap();
    assert_eq!(PathBuf::from(&m2.to), inbox.join("Kira_1.pdf"));
    assert_eq!(fs::read(inbox.join("Kira.pdf")).unwrap(), b"1", "uzerine yazilmaz");

    let odd = fx.file("model.stl", b"x");
    assert!(move_to_inbox(&fx.settings, day(), &odd).unwrap().is_none());
    assert!(odd.exists(), "grubu olmayan dosya yerinde kalir");

    let mut other = fx.settings.clone();
    other.include_other = true;
    let m3 = move_to_inbox(&other, day(), &odd).unwrap().unwrap();
    assert_eq!(m3.group, "Diğer");
}

#[test]
fn uppercase_extension_matches_and_extensionless_file_stays() {
    let fx = Fx::new();
    let up = fx.file("RAPOR.PDF", b"x");
    assert_eq!(move_to_inbox(&fx.settings, day(), &up).unwrap().unwrap().group, "Dokümanlar");
    let bare = fx.file("README", b"x");
    assert!(move_to_inbox(&fx.settings, day(), &bare).unwrap().is_none());
    assert!(bare.exists());
}

#[test]
fn moved_files_land_outside_watched_sources_so_they_are_not_caught_again() {
    let fx = Fx::new();
    let m = move_to_inbox(&fx.settings, day(), &fx.file("a.zip", b"x")).unwrap().unwrap();
    let to = PathBuf::from(&m.to);
    assert!(!to.starts_with(&fx.src), "gelen kutusu kaynagin disinda");
    // Yedek klasoru kaynagin icinde secilemez; aksi halde watcher kendi tasidigini yakalardi.
    let mut nested = fx.settings.clone();
    nested.backup_root = fx.src.join("Yedek").to_string_lossy().into();
    assert!(nested.validated().is_err());
}

#[test]
fn pending_waits_until_size_and_mtime_are_stable() {
    let fx = Fx::new();
    let p = fx.file("buyuk.zip", b"aaa");
    let t0 = Instant::now();
    let mut pending = Pending::default();
    pending.observe(p.clone(), t0);
    assert!(pending.take_ready(t0, STABLE_FOR).is_empty(), "ilk gozlem sayaci baslatir");
    assert!(pending.take_ready(t0 + Duration::from_secs(1), STABLE_FOR).is_empty());

    fs::write(&p, b"aaaaaa").unwrap(); // hala yaziliyor
    assert!(pending.take_ready(t0 + Duration::from_secs(3), STABLE_FOR).is_empty(), "degisince sayac sifirlanir");
    assert!(pending.take_ready(t0 + Duration::from_secs(4), STABLE_FOR).is_empty());
    assert_eq!(pending.take_ready(t0 + Duration::from_secs(5), STABLE_FOR), vec![p.clone()]);
    assert!(pending.is_empty());

    pending.observe(fx.src.join("silindi.pdf"), t0);
    assert!(pending.take_ready(t0 + Duration::from_secs(9), STABLE_FOR).is_empty());
    assert_eq!(pending.len(), 0, "kaybolan dosya unutulur");
}

#[test]
fn history_groups_moves_within_the_same_minute() {
    let mv = |n: &str| InboxMove { from: format!("C:\\D\\{n}"), to: format!("D:\\Y\\{n}"), group: "Dokümanlar".into() };
    let mut h = Vec::new();
    record_moves(&mut h, &[], 1_000);
    assert!(h.is_empty(), "bos adim yazilmaz");
    let minute = 29_000_000 * 60_000;
    record_moves(&mut h, &[mv("a.pdf")], minute + 1_000);
    record_moves(&mut h, &[mv("b.pdf"), mv("c.pdf")], minute + 59_000);
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].moves.len(), 3);
    assert_eq!(h[0].tag, HISTORY_TAG);
    record_moves(&mut h, &[mv("d.pdf")], minute + 61_000);
    assert_eq!(h.len(), 2, "yeni dakika yeni kayit");

    h.insert(0, UndoRecord { id: "X".into(), timestamp: minute + 62_000, tag: "Türüne göre düzenleme".into(), moves: Vec::new() });
    record_moves(&mut h, &[mv("e.pdf")], minute + 63_000);
    assert_eq!(h.len(), 4, "arada baska islem varsa birlestirilmez");
}

#[test]
fn same_dir_ignores_case_separators_and_trailing_slash() {
    let a = Path::new(r"C:\Users\Ali\Downloads");
    assert!(same_dir(a, Path::new(r"c:\users\ali\downloads\")));
    assert!(same_dir(a, Path::new("C:/Users/Ali/Downloads/")));
    assert!(same_dir(a, Path::new(r"\\?\C:\Users\Ali\Downloads")));
    assert!(!same_dir(a, Path::new(r"C:\Users\Ali\Downloads\Alt")));
}

#[test]
fn plan_does_not_move_and_failed_move_is_unrecorded() {
    let fx = Fx::new();
    let a = fx.file("plan.pdf", b"x");
    let m = plan_inbox_move(&fx.settings, day(), &a).unwrap().unwrap();
    assert!(a.exists() && !Path::new(&m.to).exists(), "plan dosyaya dokunmaz");

    let mut h = Vec::new();
    let now = 29_000_000 * 60_000;
    let first = InboxMove { from: r"C:\D\ilk.pdf".into(), to: r"D:\Y\ilk.pdf".into(), group: "Dokümanlar".into() };
    record_moves(&mut h, std::slice::from_ref(&first), now);
    record_moves(&mut h, std::slice::from_ref(&m), now + 1);
    unrecord(&mut h, &m);
    assert_eq!(h[0].moves.len(), 1, "yalnizca basarisiz tasima cikarilir");
    unrecord(&mut h, &first);
    assert!(h.is_empty(), "bos kalan kayit silinir");
}

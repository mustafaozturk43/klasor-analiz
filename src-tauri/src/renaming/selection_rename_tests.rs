use super::plan_selection_rename;
use crate::inbox_rename::ListRenameOptions;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn root() -> PathBuf {
    let r = std::env::temp_dir().join(format!("sfo_selrn_{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::SeqCst)));
    let _ = fs::remove_dir_all(&r);
    fs::create_dir_all(&r).unwrap();
    r
}

fn opts(template: &str, group: &str, dates: Vec<Option<u64>>, tidy: bool) -> ListRenameOptions {
    ListRenameOptions { template: template.into(), custom_text: String::new(), start: 1, pad: 2, group: Some(group.into()), dates, tidy }
}

fn new_name(to: &Option<String>) -> String {
    Path::new(to.as_ref().expect("yeni ad olmali")).file_name().unwrap().to_string_lossy().into_owned()
}

#[test]
fn selection_rename_uses_given_dates_and_group() {
    let r = root();
    let f = r.join("IMG_1.jpg");
    fs::write(&f, b"x").unwrap();
    let roots = vec![r.to_string_lossy().into_owned()];
    let o = opts("{Grup}_{tarih}_{sıra}", "Mayıs", vec![Some(1_718_366_400)], false);
    let (plan, _) = plan_selection_rename(&roots, &[f.to_string_lossy().into_owned()], &o).unwrap();
    assert_eq!(new_name(&plan.entries[0].to), "Mayıs_2024-06-14_01.jpg");
    let _ = fs::remove_dir_all(&r);
}

#[test]
fn selection_rename_tidies_empty_fields() {
    let r = root();
    let f = r.join("IMG_1.jpg");
    fs::write(&f, b"x").unwrap();
    let roots = vec![r.to_string_lossy().into_owned()];
    let o = opts("{OzelMetin}_{Grup}__{sıra}_", "Mayıs", vec![], true);
    let (plan, _) = plan_selection_rename(&roots, &[f.to_string_lossy().into_owned()], &o).unwrap();
    assert_eq!(new_name(&plan.entries[0].to), "Mayıs_01.jpg");
    let _ = fs::remove_dir_all(&r);
}

#[test]
fn selection_rename_rejects_paths_outside_roots() {
    let (r, other) = (root(), root());
    let f = other.join("a.jpg");
    fs::write(&f, b"x").unwrap();
    let roots = vec![r.to_string_lossy().into_owned()];
    let err = plan_selection_rename(&roots, &[f.to_string_lossy().into_owned()], &opts("{ad}_x", "", vec![], false)).err().expect("kok disi reddedilmeli");
    assert!(err.starts_with("Yalnızca açık klasördeki dosyalar adlandırılabilir"), "{err}");
    let _ = fs::remove_dir_all(&r);
    let _ = fs::remove_dir_all(&other);
}

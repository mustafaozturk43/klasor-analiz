use super::list_files_core as list_files;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn list_files_reports_modified_time() {
    let root = std::env::temp_dir().join(format!("sfo_list_mtime_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("a.txt"), b"x").unwrap();
    let files = list_files(root.to_string_lossy().into_owned(), Some(false), None, None, Some(true)).unwrap();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    assert_eq!(files.len(), 1);
    let modified = files[0].modified_at.expect("modified_at dolu olmali");
    assert!(now.saturating_sub(modified) <= 60, "modified_at yeni olmali: {modified} / {now}");
    let _ = std::fs::remove_dir_all(&root);
}

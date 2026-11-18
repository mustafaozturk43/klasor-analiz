//! Tarihe Gore gruplamada kok klasore yazilan dizin dosyasi: hangi dosya hangi
//! klasore gitti. Her gruplama dosyanin sonuna yeni bir bolum olarak eklenir;
//! tasimadan once yazilir (yazilamazsa gruplama baslamaz).

use crate::models::MoveRecord;
use chrono::Local;
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub const INDEX_FILE: &str = "Tarih_Dizini.txt";

fn rel(root: &Path, p: &str) -> String {
    Path::new(p).strip_prefix(root).map(|r| r.to_string_lossy().into_owned()).unwrap_or_else(|_| p.to_string())
}

pub fn render(root: &Path, entries: &[MoveRecord], when: &str) -> String {
    let mut by_dir: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for e in entries {
        let to = Path::new(&e.to);
        let dir = to.parent().map(|d| rel(root, &d.to_string_lossy())).unwrap_or_default();
        let name = to.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        by_dir.entry(dir).or_default().push((name, rel(root, &e.from)));
    }
    let mut out = format!("=== {when} · {} dosya · {} klasör ===\n", entries.len(), by_dir.len());
    for (dir, mut files) in by_dir {
        files.sort();
        out.push_str(&format!("\n[{dir}] ({} dosya)\n", files.len()));
        for (name, from) in files {
            out.push_str(&format!("  {name}   <- {from}\n"));
        }
    }
    out.push('\n');
    out
}

pub fn write_index(root: &Path, entries: &[MoveRecord]) -> Result<(), String> {
    let path = root.join(INDEX_FILE);
    let new = !path.exists();
    let text = render(root, entries, &Local::now().format("%Y-%m-%d %H:%M").to_string());
    let mut f = OpenOptions::new().create(true).append(true).open(&path).map_err(|e| format!("Dizin dosyası yazılamadı: {e}"))?;
    let bom: &[u8] = if new { b"\xEF\xBB\xBF" } else { b"" };
    f.write_all(bom).and_then(|_| f.write_all(text.as_bytes())).map_err(|e| format!("Dizin dosyası yazılamadı: {e}"))
}

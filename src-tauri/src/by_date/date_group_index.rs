//! Tarihe Gore gruplamada kok klasore yazilan HTML dizin: klasor -> dosya (link)
//!. Her gruplama en uste yeni bir bolum olarak eklenir; tasimadan
//! once yazilir (yazilamazsa gruplama baslamaz). Linkler kok klasore gorelidir.

use crate::models::MoveRecord;
use chrono::Local;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const INDEX_FILE: &str = "Tarih_Dizini.html";
/// Eski surumun duz metin dizini; gruplamaya girmesin diye atlanir.
pub const LEGACY_INDEX_FILE: &str = "Tarih_Dizini.txt";
const MARKER: &str = "<!--YENI-->";

const HEAD: &str = "<!doctype html><html lang=\"tr\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
<title>Tarih Dizini</title><style>\
body{font:15px/1.5 system-ui,sans-serif;margin:0 auto;max-width:960px;padding:24px;color:#1d2a24;background:#f6f8f5}\
h1{margin:0 0 12px}h2{font-size:16px;margin:28px 0 8px;color:#4a5a50}\
#q{width:100%;box-sizing:border-box;padding:10px 12px;border:1px solid #b8c4bb;border-radius:8px;font:inherit}\
details{background:#fff;border:1px solid #d6ded8;border-radius:8px;margin:8px 0;padding:6px 12px}\
summary{cursor:pointer;font-weight:600}table{width:100%;border-collapse:collapse;margin:6px 0}\
td{padding:3px 8px 3px 0;vertical-align:top;border-top:1px solid #eef1ee}\
a{color:#1a6b4a;text-decoration:none}a:hover{text-decoration:underline}\
@media(prefers-color-scheme:dark){body{background:#121a16;color:#dfe8e2}details{background:#1a251f;border-color:#2c3a32}\
#q{background:#1a251f;color:inherit;border-color:#2c3a32}td{border-color:#26332b}a{color:#6fd0a0}h2{color:#9fb2a6}}\
</style></head><body><h1>Tarih Dizini</h1>\
<input id=\"q\" type=\"search\" placeholder=\"Dosya veya klasör ara…\" autocomplete=\"off\"><main>";

const TAIL: &str = "</main><script>\
const q=document.getElementById('q');q.addEventListener('input',()=>{const t=q.value.trim().toLowerCase();\
document.querySelectorAll('details').forEach(d=>{let n=0;d.querySelectorAll('tr').forEach(r=>{const ok=!t||r.textContent.toLowerCase().includes(t);\
r.hidden=!ok;n+=ok;});d.hidden=!n;if(t&&n)d.open=true;});});\
</script></body></html>\n";

fn rel(root: &Path, p: &str) -> String {
    Path::new(p).strip_prefix(root).map(|r| r.to_string_lossy().into_owned()).unwrap_or_else(|_| p.to_string())
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Goreli yol -> `/` ayiracli, yuzde-kodlanmis URL.
fn href(rel_path: &str) -> String {
    let mut out = String::new();
    for b in rel_path.replace('\\', "/").bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

pub fn render_section(root: &Path, entries: &[MoveRecord], when: &str) -> String {
    let mut by_dir: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for e in entries {
        let to = rel(root, &e.to);
        let p = Path::new(&to);
        let dir = p.parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
        by_dir.entry(dir).or_default().push(p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default());
    }
    let mut out = format!("<section><h2>{} · {} dosya · {} klasör</h2>\n", esc(when), entries.len(), by_dir.len());
    for (dir, mut names) in by_dir {
        names.sort();
        // Dosya adi, dosyanin bulundugu klasoru acar.
        let link = if dir.is_empty() { "./".to_string() } else { format!("{}/", href(&dir)) };
        out.push_str(&format!("<details open><summary><a href=\"{link}\">{}</a> ({} dosya)</summary><table>\n", esc(&dir), names.len()));
        for name in names {
            out.push_str(&format!("<tr><td><a href=\"{link}\" title=\"Klasörü aç\">{}</a></td></tr>\n", esc(&name)));
        }
        out.push_str("</table></details>\n");
    }
    out.push_str("</section>\n");
    out
}

pub fn write_index(root: &Path, entries: &[MoveRecord]) -> Result<(), String> {
    let path = root.join(INDEX_FILE);
    let section = render_section(root, entries, &Local::now().format("%Y-%m-%d %H:%M").to_string());
    let page = match fs::read_to_string(&path) {
        Ok(old) if old.contains(MARKER) => old.replacen(MARKER, &format!("{MARKER}\n{section}"), 1),
        _ => format!("{HEAD}\n{MARKER}\n{section}{TAIL}"),
    };
    let tmp = root.join(format!("{INDEX_FILE}.tmp"));
    let fail = |e: std::io::Error| format!("Dizin dosyası yazılamadı: {e}");
    fs::write(&tmp, page).map_err(fail)?;
    fs::rename(&tmp, &path).map_err(fail)
}

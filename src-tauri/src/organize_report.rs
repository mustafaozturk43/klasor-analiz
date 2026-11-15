//! Duzenleme onizlemesini CSV olarak disa aktarir (Excel uyumlu, `;` ayiricli).

use crate::organize_models::{OrganizeAction, OrganizeEntry};
use std::fs;
use std::path::Path;

fn action_label(a: OrganizeAction) -> &'static str {
    match a {
        OrganizeAction::Move => "Taşınacak",
        OrganizeAction::InPlace => "Zaten yerinde",
        OrganizeAction::Quarantine => "Karantinaya",
        OrganizeAction::Skip => "Atlanacak",
    }
}

/// Excel'in formul olarak calistirabilecegi baslangiclar (=, +, -, @) metne cevrilir;
/// noktali virgul, tirnak ve satir sonu iceren degerler tirnaklanir.
pub(crate) fn cell(value: &str) -> String {
    let guarded = if value.starts_with(['=', '+', '-', '@']) { format!("'{value}") } else { value.to_string() };
    if guarded.contains([';', '"', '\n', '\r']) {
        format!("\"{}\"", guarded.replace('"', "\"\""))
    } else {
        guarded
    }
}

pub fn write_csv(target: &Path, entries: &[OrganizeEntry]) -> Result<usize, String> {
    let mut out = String::from("\u{FEFF}");
    out.push_str("Durum;Kaynak;Hedef;Kategori;Tür;Boyut (bayt);Numaralandı;Benzer ad;Not\n");
    for e in entries {
        let row = [
            action_label(e.action).to_string(),
            cell(&e.from),
            cell(e.to.as_deref().unwrap_or("")),
            cell(e.category.as_deref().unwrap_or("")),
            cell(e.kind.as_deref().unwrap_or("")),
            e.size.to_string(),
            if e.renamed { "Evet" } else { "" }.to_string(),
            cell(e.similar_to.as_deref().unwrap_or("")),
            cell(e.note.as_deref().unwrap_or("")),
        ];
        out.push_str(&row.join(";"));
        out.push('\n');
    }
    fs::write(target, out).map_err(|e| e.to_string())?;
    Ok(entries.len())
}

#[tauri::command(async)]
pub fn export_organize_csv(file_path: String, entries: Vec<OrganizeEntry>) -> Result<usize, String> {
    write_csv(Path::new(&file_path), &entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guards_formulas_and_quotes_separators() {
        assert_eq!(cell("=HYPERLINK(1)"), "'=HYPERLINK(1)");
        assert_eq!(cell("a;b"), "\"a;b\"");
        assert_eq!(cell("say \"x\""), "\"say \"\"x\"\"\"");
        assert_eq!(cell("C:\\Rapor.pdf"), "C:\\Rapor.pdf");
    }
}

//! Genel tablo raporu: arayuz basliklari ve satirlari hazirlar (Klasor sagligi, Benzer gorseller,
//! Kural motoru plani); burasi hucreleri guvenle CSV'ye yazar. Excel uyumlu: UTF-8 BOM, ayirici
//! noktali virgul, formul gibi baslayan hucreler `cell` ile metne cevrilir.

use crate::organize_report::cell;
use std::fs;
use std::path::Path;

/// Rapor satiri ust siniri (dev klasorlerde bellek ve Excel sinirini korur).
pub const MAX_ROWS: usize = 1_000_000;

pub fn table_csv(header: &[String], rows: &[Vec<String>]) -> String {
    let line = |cells: &[String]| cells.iter().map(|c| cell(c)).collect::<Vec<_>>().join(";");
    let mut out = String::from("\u{FEFF}");
    out.push_str(&line(header));
    out.push('\n');
    for r in rows {
        out.push_str(&line(r));
        out.push('\n');
    }
    out
}

/// Tabloyu `.csv` dosyasina yazir; yazilan veri satiri sayisini dondurur. Var olan dosyanin
/// uzerine yalnizca kullanicinin kaydet diyalogunda sectigi hedef yazilir.
pub fn write_table(target: &Path, header: &[String], rows: &[Vec<String>]) -> Result<usize, String> {
    if rows.len() > MAX_ROWS {
        return Err(format!("Rapor çok büyük ({} satır); en fazla {MAX_ROWS} satır yazılabilir.", rows.len()));
    }
    if !target.extension().is_some_and(|e| e.eq_ignore_ascii_case("csv")) {
        return Err("Rapor .csv uzantılı bir dosyaya yazılmalı.".into());
    }
    fs::write(target, table_csv(header, rows)).map_err(|e| e.to_string())?;
    Ok(rows.len())
}

#[tauri::command(async)]
pub(crate) fn export_table_csv(file_path: String, header: Vec<String>, rows: Vec<Vec<String>>) -> Result<usize, String> {
    write_table(Path::new(&file_path), &header, &rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn csv_has_bom_semicolons_and_guards_formulas_and_quotes() {
        let csv = table_csv(&s(&["Ad", "Not"]), &[s(&["=1+1", "a;b"]), s(&["söz \"x\"", "satır\nsonu"])]);
        assert_eq!(csv, "\u{FEFF}Ad;Not\n'=1+1;\"a;b\"\n\"söz \"\"x\"\"\";\"satır\nsonu\"\n");
    }

    #[test]
    fn write_returns_data_row_count_and_requires_csv_extension() {
        let dir = std::env::temp_dir().join(format!("sfo_table_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let ok = dir.join("rapor.CSV");
        assert_eq!(write_table(&ok, &s(&["A"]), &[s(&["1"]), s(&["2"])]).unwrap(), 2);
        assert!(fs::read_to_string(&ok).unwrap().starts_with('\u{FEFF}'));
        assert!(write_table(&dir.join("rapor.exe"), &s(&["A"]), &[]).unwrap_err().contains(".csv"));
        assert!(!dir.join("rapor.exe").exists(), "yanlis uzantiya yazilmaz");
        let _ = fs::remove_dir_all(&dir);
    }
}

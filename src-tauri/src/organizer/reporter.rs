//! Secili gruplari CSV rapor olarak disa aktarir (Excel uyumlu). Hucreler `organize_report::cell`
//! ile yazilir: formul gibi baslayan (=, +, -, @) adlar metne cevrilir.

use crate::models::{FileCategory, FileGroup, GroupType, UndoRecord};
use crate::organize_report::cell;
use std::fs;
use std::path::Path;

/// Gruplari hedef dosyaya yazar; yazilan dosya satiri sayisini dondurur.
/// Ayirici olarak noktali virgul kullanilir (Turkce Excel yerel ayari uyumlu).
#[tauri::command(async)]
pub(crate) fn export_groups_csv(file_path: String, groups: Vec<FileGroup>) -> Result<usize, String> {
    export_csv(Path::new(&file_path), &groups)
}

pub fn export_csv(target: &Path, groups: &[FileGroup]) -> Result<usize, String> {
    let mut rows = 0usize;
    let mut out = String::from("\u{FEFF}"); // Excel UTF-8 BOM
    out.push_str("GrupID;GrupTipi;Kategori;DosyaAdi;Boyut;TamYol;HashID\n");

    for group in groups {
        let type_label = match group.group_type {
            GroupType::Exact => "Birebir Kopya",
            GroupType::Similar => "Isim Benzeri",
        };
        let cat_label = category_label(&group.category);

        for file in &group.files {
            out.push_str(&format!(
                "{};{};{};{};{};{};{}\n",
                group.id,
                type_label,
                cat_label,
                cell(&file.name),
                file.size,
                cell(&file.path),
                file.hash_id,
            ));
            rows += 1;
        }
    }

    fs::write(target, out).map_err(|e| e.to_string())?;
    Ok(rows)
}

/// Islem gecmisini (her tasima bir satir) CSV olarak yazar; satir sayisini dondurur.
#[tauri::command(async)]
pub(crate) fn export_history_csv(state: tauri::State<'_, crate::AppState>, file_path: String) -> Result<usize, String> {
    let records = state.history.lock().map_err(|e| e.to_string())?.clone();
    write_history_csv(Path::new(&file_path), &records)
}

pub fn write_history_csv(target: &Path, records: &[UndoRecord]) -> Result<usize, String> {
    use chrono::{Local, TimeZone};
    let mut rows = 0usize;
    let mut out = String::from("\u{FEFF}IslemID;Zaman;Islem;Kaynak;Hedef\n");
    for r in records {
        let when = Local.timestamp_millis_opt(r.timestamp as i64).single().map(|d| d.format("%d.%m.%Y %H:%M").to_string()).unwrap_or_default();
        for m in &r.moves {
            out.push_str(&format!("{};{};{};{};{}\n", cell(&r.id), when, cell(&r.tag), cell(&m.from), cell(&m.to)));
            rows += 1;
        }
    }
    fs::write(target, out).map_err(|e| e.to_string())?;
    Ok(rows)
}

fn category_label(category: &FileCategory) -> &'static str {
    match category {
        FileCategory::Document => "Doküman",
        FileCategory::Image => "Resim",
        FileCategory::Archive => "Arşiv",
        FileCategory::Other => "Diğer",
    }
}

//! Ana klasor + alt klasorlerin boyut analizini paralel olarak hesaplar.
//! Sadece metadata okur; her jwalk yuruyusu bagimsiz, sinirli bir havuz kullanir.

use crate::models::{FileCategory, FolderFile, FolderStat, FolderStatsResult};
use crate::scanner;
use std::fs;
use std::path::Path;

/// Root'un toplam boyutunu ve dogrudan alt klasorlerinin ozetini dondurur.
#[tauri::command(async)]
pub(crate) fn get_folder_stats(root_path: String) -> Result<FolderStatsResult, String> {
    collect(Path::new(&root_path))
}

pub fn collect(root: &Path) -> Result<FolderStatsResult, String> {
    let entries = fs::read_dir(root)
        .map_err(|e| format!("Klasör açılamadı: {} — {e}", root.display()))?;
    let mut warnings = ReadWarnings::default();
    let mut dirs = Vec::new();
    let mut root_files = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => match entry.file_type() {
                Ok(kind) if kind.is_dir() => dirs.push(entry.path()),
                Ok(_) => root_files.push(entry.path()),
                Err(e) => warnings.push(&entry.path(), &e.to_string()),
            },
            Err(e) => warnings.push(root, &e.to_string()),
        }
    }

    // Ana klasordeki direkt dosyalar.
    let mut root_total = 0u64;
    let mut root_count = 0u64;
    let mut category_bytes = [0u64; 4];
    for path in &root_files {
        match fs::metadata(path) {
            Ok(meta) => {
            if meta.is_file() {
                root_total += meta.len();
                root_count += 1;
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                category_bytes[category_index(scanner::category_of(&ext))] += meta.len();
            }
            }
            Err(e) => warnings.push(path, &e.to_string()),
        }
    }

    // Dis katman sirali; klasorun kendi yuruyusu paraleldir. Rayon par_iter
    // icinde ayni havuza is gonderip beklemek havuz tukenmesine yol aciyordu.
    let results: Vec<_> = dirs
        .iter()
        .map(|dir| folder_stat(dir.as_path()))
        .collect();
    let mut folders = Vec::new();
    for (stat, problems) in results {
        folders.push(stat);
        warnings.merge(problems);
    }
    folders.sort_by_key(|f| std::cmp::Reverse(f.total_bytes));

    let sub_total: u64 = folders.iter().map(|f| f.total_bytes).sum();
    let sub_count: u64 = folders.iter().map(|f| f.file_count).sum();
    for f in &folders {
        for (i, size) in [f.document_bytes, f.image_bytes, f.archive_bytes, f.other_bytes].iter().enumerate() {
            category_bytes[i] += size;
        }
    }

    Ok(FolderStatsResult {
        root_path: root.to_string_lossy().into_owned(),
        root_total_bytes: root_total + sub_total,
        root_file_count: root_count + sub_count,
        folders,
        category_bytes,
        warnings: warnings.finish(),
    })
}

/// Tek bir klasorun (alt klasorleri dahil) boyut ozetini hesaplar.
/// Tek jwalk gecisi: hem toplamlar hem direkt dosya listesi ayni yuruyuste toplanir.
fn folder_stat(dir: &Path) -> (FolderStat, ReadWarnings) {
    let mut stat = FolderStat {
        name: dir.file_name().unwrap_or_default().to_string_lossy().into_owned(),
        path: dir.to_string_lossy().into_owned(),
        total_bytes: 0,
        file_count: 0,
        document_bytes: 0,
        image_bytes: 0,
        archive_bytes: 0,
        other_bytes: 0,
        files: Vec::new(),
    };
    let mut warnings = ReadWarnings::default();

    for entry in scanner::walker(dir).skip_hidden(false) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                warnings.push(e.path().unwrap_or(dir), &e.to_string());
                continue;
            }
        };
        if entry.file_type().is_dir() {
            continue;
        }

        let path = entry.path();
        // Cloud/sanal suruculerde jwalk metadata hatasinda standart API ile bir kez dene.
        let metadata = entry.metadata().map_err(|e| e.to_string())
            .or_else(|_| fs::metadata(&path).map_err(|e| e.to_string()));
        accumulate_file(&mut stat, &path, metadata, entry.depth() == 1, &mut warnings);
    }

    stat.files.sort_by_key(|f| std::cmp::Reverse(f.size));
    (stat, warnings)
}

/// Hata yalniz ilgili dosyayi etkiler; onceki toplamlar asla atilmaz.
fn accumulate_file(stat: &mut FolderStat, path: &Path, metadata: Result<fs::Metadata, String>, direct: bool, warnings: &mut ReadWarnings) {
        let meta = match metadata {
            Ok(meta) if meta.is_file() => meta,
            Ok(_) => return,
            Err(e) => { warnings.push(path, &e); return; }
        };
        let size = meta.len();
        stat.total_bytes += size;
        stat.file_count += 1;

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        match scanner::category_of(&ext) {
            Some(FileCategory::Document) => stat.document_bytes += size,
            Some(FileCategory::Image) => stat.image_bytes += size,
            Some(FileCategory::Archive) => stat.archive_bytes += size,
            Some(FileCategory::Other) => stat.other_bytes += size,
            None => stat.other_bytes += size,
        }

        // Jwalk kokunun ALTINDAKI dosyalar bu klasorun direkt dosyalaridir.
        if direct {
            stat.files.push(FolderFile {
                name: path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                path: path.to_string_lossy().into_owned(),
                size,
                category: scanner::category_of(&ext),
                created_at: meta.created().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()),
                modified_at: meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()),
            });
        }
}

#[derive(Default)]
struct ReadWarnings { messages: Vec<String>, count: usize }
impl ReadWarnings {
    const LIMIT: usize = 50;
    fn push(&mut self, path: &Path, error: &str) {
        self.count += 1;
        if self.messages.len() < Self::LIMIT {
            self.messages.push(format!("Öğe okunamadı; yalnız bu öğe atlandı: {} — {error}", path.display()));
        }
    }
    fn merge(&mut self, other: Self) {
        self.count += other.count;
        self.messages.extend(other.messages.into_iter().take(Self::LIMIT.saturating_sub(self.messages.len())));
    }
    fn finish(mut self) -> Vec<String> {
        if self.count > 0 {
            self.messages.insert(0, format!("Analiz kısmi: {} okuma hatası. Okunabilen dosyalar toplama katıldı; erişilemeyen klasörlerin içeriği eksik olabilir.", self.count));
            if self.count > Self::LIMIT { self.messages.push(format!("İlk {} hata gösteriliyor.", Self::LIMIT)); }
        }
        self.messages
    }
}

fn category_index(category: Option<FileCategory>) -> usize {
    match category {
        Some(FileCategory::Document) => 0,
        Some(FileCategory::Image) => 1,
        Some(FileCategory::Archive) => 2,
        _ => 3,
    }
}

#[cfg(test)]
#[path = "folder_stats_tests.rs"]
mod tests;

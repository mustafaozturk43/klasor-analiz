//! Tarihe Gore gruplamada klasor duzeni: `Yıl\Ay`, yalniz `Yıl-Ay`, ya da sira
//! numarali yil klasoru (`01_2026`). Sira numarasi mevcut `NN_YYYY` klasorlerini
//! yeniden kullanir; yeni yillar en buyuk numaranin ardindan gelir.

use chrono::{DateTime, Datelike, Local};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const UNDATED: &str = "Tarihsiz";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DateLayout {
    /// `2026\01`
    #[default]
    YearMonth,
    /// `2026-01` (tek duzey)
    Month,
    /// `01_2026` (yila gore, sira numarali)
    YearNumbered,
}

/// Unix saniyeden yerel (yil, ay); tarih yoksa None.
pub fn year_month(secs: Option<u64>) -> Option<(i32, u32)> {
    let d = DateTime::from_timestamp(secs? as i64, 0)?.with_timezone(&Local);
    Some((d.year(), d.month()))
}

pub struct FolderNamer {
    layout: DateLayout,
    years: BTreeMap<i32, String>,
    next: u32,
}

impl FolderNamer {
    pub fn new(layout: DateLayout, root: &Path) -> Self {
        let mut years = BTreeMap::new();
        let mut max = 0;
        if layout == DateLayout::YearNumbered {
            for e in fs::read_dir(root).into_iter().flatten().flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                let parsed = name.split_once('_').filter(|(_, y)| y.len() == 4).and_then(|(n, y)| Some((n.parse::<u32>().ok()?, y.parse::<i32>().ok()?)));
                if let Some((n, y)) = parsed.filter(|_| e.path().is_dir()) {
                    max = max.max(n);
                    years.entry(y).or_insert(name);
                }
            }
        }
        Self { layout, years, next: max + 1 }
    }

    /// Plan icin yillari kronolojik sirayla tanitir (yeni numaralar boyle atanir).
    pub fn register(&mut self, mut years: Vec<i32>) {
        if self.layout != DateLayout::YearNumbered {
            return;
        }
        years.sort_unstable();
        years.dedup();
        for y in years {
            if !self.years.contains_key(&y) {
                self.years.insert(y, format!("{:02}_{y}", self.next));
                self.next += 1;
            }
        }
    }

    /// Goreli klasor yolu (`\` ayiracli); tarih yoksa `Tarihsiz`.
    pub fn folder(&self, secs: Option<u64>) -> String {
        let Some((y, m)) = year_month(secs) else { return UNDATED.to_string() };
        match self.layout {
            DateLayout::YearMonth => format!("{y}\\{m:02}"),
            DateLayout::Month => format!("{y}-{m:02}"),
            DateLayout::YearNumbered => self.years.get(&y).cloned().unwrap_or_else(|| format!("{y}")),
        }
    }
}

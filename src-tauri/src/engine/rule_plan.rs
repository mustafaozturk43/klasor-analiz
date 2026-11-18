//! Kural motoru plani: secili klasordeki dosyalara kurallar sirayla uygulanir, ILK eslesen
//! kural kazanir. Plan diske dokunmaz; ayni ad cakisirsa `ad (1).uzanti`.

use super::rule_model::{Facts, Rule, RuleAction};
use crate::by_date::date_group_layout::year_month;
use crate::by_date::date_group_plan::collect;
use crate::rename::FileStamps;
use crate::selection_paths::free_dest;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleEntry {
    pub from: String,
    /// Bos: Geri Donusum Kutusu'na gidecek.
    pub to: String,
    pub rule: String,
    pub action: RuleAction,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleCount {
    pub rule: String,
    pub action: RuleAction,
    pub count: usize,
    pub samples: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RulePlan {
    pub id: String,
    pub root: String,
    pub entries: Vec<RuleEntry>,
    pub counts: Vec<RuleCount>,
    pub scanned: usize,
    /// Hicbir kurala uymayan dosyalar.
    pub unmatched: usize,
    /// Bir kurala uyan ama zaten hedef klasorde olan dosyalar.
    pub in_place: usize,
    pub trash_count: usize,
    pub created_dirs: Vec<String>,
}

const SAMPLES: usize = 3;

/// `now`: unix saniye (yas hesabi). Tum kurallar dogrulanir; en az bir etkin kural gerekir.
pub fn plan_rules(root: &Path, rules: &[Rule], recursive: bool, now: u64) -> Result<(RulePlan, FileStamps), String> {
    if !root.is_dir() {
        return Err("Önce var olan bir klasör seçin.".into());
    }
    let rules: Vec<Rule> = rules.iter().filter(|r| r.enabled).map(Rule::validated).collect::<Result<_, _>>()?;
    if rules.is_empty() {
        return Err("Etkin kural yok. Önce bir kural ekleyin.".into());
    }
    let mut files = Vec::new();
    collect(root, recursive, &[], &mut files);
    let mut counts: Vec<RuleCount> = rules.iter().map(|r| RuleCount { rule: r.name.clone(), action: r.action, count: 0, samples: Vec::new() }).collect();
    let (mut entries, mut stamps, mut reserved) = (Vec::new(), Vec::new(), HashSet::new());
    let (mut unmatched, mut in_place, mut trash_count) = (0, 0, 0);
    let mut created: Vec<String> = Vec::new();
    for p in &files {
        let Ok(meta) = fs::metadata(p) else { continue };
        let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        let mtime = meta.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs());
        let name_lower = name.to_lowercase();
        let facts = Facts { ext: &ext, name_lower: &name_lower, size: meta.len(), age_days: mtime.map_or(0, |m| now.saturating_sub(m) / 86_400) };
        let Some(i) = rules.iter().position(|r| r.matches(&facts)) else {
            unmatched += 1;
            continue;
        };
        let rule = &rules[i];
        let to = if rule.action == RuleAction::Trash {
            trash_count += 1;
            String::new()
        } else {
            let mut dir = rule.target.split('\\').fold(root.to_path_buf(), |d, seg| d.join(seg));
            if rule.action == RuleAction::MoveYearMonth {
                let (y, m) = year_month(mtime).map_or(("Tarihsiz".to_string(), None), |(y, m)| (y.to_string(), Some(m)));
                dir = dir.join(y);
                if let Some(m) = m {
                    dir = dir.join(format!("{m:02}"));
                }
            }
            if p.parent() == Some(dir.as_path()) {
                in_place += 1;
                continue;
            }
            for d in dir.ancestors().take_while(|d| *d != root) {
                let s = d.to_string_lossy().into_owned();
                if !d.exists() && !created.contains(&s) {
                    created.push(s);
                }
            }
            free_dest(&dir, &name, &mut reserved).to_string_lossy().into_owned()
        };
        counts[i].count += 1;
        if counts[i].samples.len() < SAMPLES {
            counts[i].samples.push(name);
        }
        stamps.push((meta.len(), meta.modified().ok()));
        entries.push(RuleEntry { from: p.to_string_lossy().into_owned(), to, rule: rule.name.clone(), action: rule.action });
    }
    let id = format!("RE-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos());
    let plan = RulePlan { id, root: root.to_string_lossy().into_owned(), entries, counts, scanned: files.len(), unmatched, in_place, trash_count, created_dirs: created };
    Ok((plan, stamps))
}

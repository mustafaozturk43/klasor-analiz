//! Otomatik tasima bildirimi: watcher dosya yerlestirdiginde, ayar aciksa ve pencere
//! o an kullanilmiyorsa bir Windows bildirimi gosterir. Bildirim yalnizca bilgi verir.

use crate::settings::SettingsState;
use crate::watcher_core::InboxMove;
use std::path::Path;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

/// Bildirim metni: tek dosyada "ad → Grup", coklu tasimada grup basina sayi.
pub fn summary(moved: &[InboxMove]) -> Option<String> {
    let first = moved.first()?;
    if moved.len() == 1 {
        let name = Path::new(&first.to).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        return Some(format!("{name} → {}", first.group));
    }
    let mut counts: Vec<(&str, usize)> = Vec::new();
    for m in moved {
        match counts.iter_mut().find(|(g, _)| *g == m.group) {
            Some((_, n)) => *n += 1,
            None => counts.push((&m.group, 1)),
        }
    }
    let groups: Vec<String> = counts.iter().map(|(g, n)| format!("{g} {n}")).collect();
    Some(format!("{} dosya gelen kutusuna taşındı ({})", moved.len(), groups.join(", ")))
}

fn window_in_use(app: &AppHandle) -> bool {
    app.webview_windows().values().any(|w| w.is_visible().unwrap_or(false) && w.is_focused().unwrap_or(false))
}

pub fn inbox_moved(app: &AppHandle, moved: &[InboxMove]) {
    let enabled = app.state::<SettingsState>().settings.lock().map(|s| s.notify_moves).unwrap_or(false);
    if !enabled || window_in_use(app) {
        return;
    }
    if let Some(body) = summary(moved) {
        let _ = app.notification().builder().title("Smart File Organizer").body(body).show();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mv(name: &str, group: &str) -> InboxMove {
        InboxMove { from: format!(r"C:\Indirilenler\{name}"), to: format!(r"D:\Yedek\_GelenKutusu\{group}\{name}"), group: group.into() }
    }

    #[test]
    fn summary_names_a_single_file_and_counts_groups_for_many() {
        assert_eq!(summary(&[]), None);
        assert_eq!(summary(&[mv("rapor.pdf", "Dokümanlar")]).as_deref(), Some("rapor.pdf → Dokümanlar"));
        let many = [mv("a.pdf", "Dokümanlar"), mv("b.jpg", "Görseller"), mv("c.pdf", "Dokümanlar")];
        assert_eq!(summary(&many).as_deref(), Some("3 dosya gelen kutusuna taşındı (Dokümanlar 2, Görseller 1)"));
    }
}

//! Tauri komutlarini disa aktaran ana baglanti noktasi.
//!
//! Agir komutlar `#[tauri::command(async)]` ile ayri bir thread'de calisir;
//! boylece tarama sirasinda WebView ana thread'i bloke olmaz (UI donmaz).

// Moduller konuya gore klasorlere ayrilir; `crate::<modul>` yollari asagidaki yeniden disa aktarimlarla korunur.
pub mod auto;
pub mod by_date;
pub mod engine;
pub mod fileops;
pub mod inbox;
pub mod models;
pub mod organizer;
pub mod renaming;
pub mod scan;
pub mod winshell;

pub use auto::{automation, notifier, settings, settings_cmds, startup, watcher, watcher_core};
use auto::{single_instance, tray};
pub use engine::{rule_apply, rule_model, rule_plan};
pub use by_date::{date_group, date_group_index, date_group_layout, date_group_plan};
pub use fileops::{batch_move, file_ops, group_moves, history, history_cmds, mover, path_guard, quarantine_cmds, quarantine_manage, selection_move, selection_paths, subfolders};
pub use inbox::{inbox_dupes, inbox_list, inbox_path, inbox_rename, inbox_sources, inbox_suggest, inspector, operations, trash_ops};
pub use organizer::{organize, organize_apply, organize_cleanup, organize_models, organize_names, organize_plan, organize_report, organize_rules, organize_walk, reporter, table_report};
pub use renaming::{name_rules, name_rules_apply, name_rules_plan, naming, rename, rename_template, selection_rename};
pub use scan::{cloud, compare, file_list, folder_stats, grouper, hash_cache, hasher, health, scanner, similarity, visual};
pub use winshell::{asset_scope, media_dates, shell, shell_cmds, shell_thumb, thumbnail};

use hash_cache::HashCache;
use models::{ScanProgress, ScanResult, UndoRecord};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

/// Uygulama genelinde paylasilan durum: gecmis + iptal bayragi + hash onbellegi.
pub(crate) struct AppState {
    pub(crate) history: Mutex<Vec<UndoRecord>>,
    pub(crate) dir: PathBuf,
    pub(crate) cancel_scan: AtomicBool,
    /// Yeniden taramalarda degismeyen dosyalari tekrar hash'lememek icin.
    pub(crate) hash_cache: HashCache,
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[tauri::command(async)]
fn scan_and_group(
    app: AppHandle, state: State<'_, AppState>,
    root_path: String,
    include_subdirs: Option<bool>,
    similarity_threshold: Option<f64>,
    exclude_folders: Option<Vec<String>>,
    banned_extensions: Option<Vec<String>>,
    include_other: Option<bool>,
    on_progress: Channel<ScanProgress>,
) -> Result<ScanResult, String> {
    // Yeni tarama baslarken iptal bayragini sifirla.
    state.cancel_scan.store(false, Ordering::SeqCst);
    let cancel_flag = &state.cancel_scan;
    let is_cancelled = move || cancel_flag.load(Ordering::Relaxed);

    let include = include_subdirs.unwrap_or(true);
    let threshold = similarity_threshold
        .unwrap_or(models::DEFAULT_SIMILARITY_THRESHOLD)
        .clamp(0.5, 0.99);
    let excludes = exclude_folders.unwrap_or_default();
    let banned = models::normalize_exts(&banned_extensions.unwrap_or_default());
    let include_other = include_other.unwrap_or(false);
    let root = PathBuf::from(&root_path);
    asset_scope::allow_dir(&app, &root);

    let emit = |phase: &str, done: u64, total: u64| {
        let _ = on_progress.send(ScanProgress {
            phase: phase.to_string(),
            files_done: done,
            total_files: total,
        });
    };

    emit("scanning", 0, 0);
    let scanned = scanner::scan_reporting(&root, include, &excludes, &banned, include_other, &|found| {
        emit("scanning", found, 0)
    }, &is_cancelled)?;
    let (files, mut warnings) = (scanned.files, scanned.warnings);
    let total_files = files.len();

    emit("hashing", 0, total_files as u64);
    let (groups, unreadable) = grouper::group_reporting(files, threshold, &state.hash_cache, &|done, total| {
        emit("hashing", done, total)
    }, &is_cancelled)?;
    warnings.extend(unreadable);
    let _ = state.hash_cache.save(&state.dir); // Sonraki acilista ayni dosyalar yeniden hash'lenmez.
    emit("done", 1, 1);

    Ok(ScanResult {
        root_path,
        total_files,
        groups,
        comparison: None,
        warnings,
    })
}

/// Devam eden taramayi durdurur (bir sonraki kontrol noktasinda kesilir).
#[tauri::command]
fn cancel_scan(state: State<'_, AppState>) -> Result<(), String> {
    state.cancel_scan.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(single_instance::plugin())
        .manage(rename::RenameState::default())
        .manage(name_rules_apply::NameRulesState::default())
        .manage(selection_move::SelectionMoveState::default())
        .manage(organize::OrganizeState::default())
        .manage(date_group::DateGroupState::default())
        .manage(rule_apply::RuleEngineState::default())
        .manage(visual::VisualState::default())
        .manage(automation::AutomationState::default())
        .manage(watcher::WatcherState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(startup::plugin())
        .plugin(tauri_plugin_notification::init())
        .on_window_event(tray::on_window_event)
        .setup(|app| {
            // Tasma gecmisini uygulama veri dizininden yukle.
            let dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir());
            let _ = std::fs::create_dir_all(&dir);
            let loaded = settings::SettingsState::load(&dir);
            let (autostart, automation_on) = loaded.settings.lock().map(|s| (s.start_with_windows, s.automation_enabled)).unwrap_or_default();
            app.manage(loaded);
            tray::setup(app.handle())?;
            asset_scope::allow_settings(app.handle());
            startup::on_launch(app.handle(), autostart, automation_on);
            let history = history::load(&dir);
            let hash_cache = HashCache::load(&dir);
            app.manage(AppState {
                history: Mutex::new(history),
                dir,
                cancel_scan: AtomicBool::new(false),
                hash_cache,
            });
            // Gecmis yuklendikten sonra: watcher tasimalari gecmise yazar.
            watcher::restart(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_and_group, cancel_scan,
            compare::compare_folders,
            file_list::list_files,
            rename::preview_date_rename, rename::apply_rename_plan,
            rename_template::preview_template_rename,
            name_rules_apply::preview_name_rules, name_rules_apply::apply_name_rules,
            group_moves::move_named_groups,
            selection_move::preview_selection_move, selection_move::apply_selection_move,
            subfolders::list_subfolders,
            shell_cmds::get_shell_thumbnails, shell_cmds::read_media_dates,
            folder_stats::get_folder_stats,
            history_cmds::undo_last_move, history_cmds::undo_history_entry, history_cmds::get_move_history, history_cmds::clear_move_history,
            health::folder_health, health::remove_empty_folders, visual::find_similar_images,
            quarantine_manage::list_quarantine, quarantine_manage::restore_quarantined, quarantine_manage::delete_quarantined, quarantine_manage::quarantine_files,
            organize::preview_organize, organize::apply_organize, organize::cancel_organize,
            organize_rules::default_organize_rules,
            organize_report::export_organize_csv,
            quarantine_cmds::clear_quarantine,
            settings_cmds::get_settings, settings_cmds::save_settings, settings_cmds::set_ui_pref, settings_cmds::default_ext_groups, settings_cmds::set_auto_move,
            automation::get_automation_status, automation::set_automation_paused,
            inbox_list::list_inbox,
            inbox_rename::preview_inbox_rename,
            selection_rename::preview_selection_rename,
            operations::get_target_subfolders, operations::create_target_folder, operations::move_selected_files,
            trash_ops::delete_files_to_trash,
            reporter::export_groups_csv, reporter::export_history_csv, table_report::export_table_csv,
            shell::reveal_in_explorer, shell::open_file_default,
            date_group::preview_date_group, date_group::apply_date_group, date_group::rebuild_date_index, rule_apply::preview_rules, rule_apply::apply_rules,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

//! Kaynak klasorleri (yalnizca ilk seviye) `notify` ile izler. Gelen dosyalar
//! yazimi bitince (`watcher_core::Pending`) gunun gelen kutusuna tasinir, gecmise
//! yazilir ve arayuze `inbox-updated` olayi gonderilir. Otomasyon duraklatildiginda
//! dosyalar bekletilir, devam edilince tasinir. Ayarlar kaydedilince yeniden kurulur.
//! "Otomatik tasi" kapaliysa dosyalara dokunulmaz; yalnizca arayuz yenilenir.

use crate::automation;
use crate::organize_models::FailedMove;
use crate::settings::{AppSettings, SettingsState};
use crate::watcher_core::{self, InboxMove, Pending, STABLE_FOR};
use crate::{history, AppState};
use notify::{EventKind, RecursiveMode, Watcher};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

struct Running {
    _watcher: notify::RecommendedWatcher,
    stop: Arc<AtomicBool>,
}

#[derive(Default)]
pub struct WatcherState {
    running: Mutex<Option<Running>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxUpdate {
    pub moved: Vec<InboxMove>,
    pub failed: Vec<FailedMove>,
}

/// Mevcut izlemeyi durdurur; otomasyon aciksa guncel ayarlarla yeniden baslatir.
pub fn restart(app: &AppHandle) {
    let state = app.state::<WatcherState>();
    let mut guard = state.running.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(old) = guard.take() {
        old.stop.store(true, Ordering::SeqCst);
    }
    let settings = app.state::<SettingsState>().settings.lock().map(|s| s.clone()).unwrap_or_default();
    if !settings.automation_enabled || settings.backup_root.is_empty() || settings.source_folders.is_empty() {
        return;
    }
    match start(app.clone(), &settings) {
        Ok(running) => *guard = Some(running),
        Err(e) => {
            let _ = app.emit("inbox-error", e);
        }
    }
}

fn top_level_files(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir).map(|rd| rd.flatten().map(|e| e.path()).collect()).unwrap_or_default()
}

fn start(app: AppHandle, settings: &AppSettings) -> Result<Running, String> {
    let (tx, rx) = mpsc::channel::<PathBuf>();
    let sources: Vec<PathBuf> = settings.source_folders.iter().map(PathBuf::from).collect();
    let watched = sources.clone();
    let event_tx = tx.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let Ok(event) = res else { return };
        if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
            return;
        }
        for p in event.paths {
            // Yalnizca kaynak klasorlerin dogrudan icindeki dosyalar.
            if p.parent().is_some_and(|parent| watched.iter().any(|w| watcher_core::same_dir(w, parent))) {
                let _ = event_tx.send(p);
            }
        }
    })
    .map_err(|e| format!("Klasör izleme başlatılamadı: {e}"))?;

    let mut missing = Vec::new();
    for src in &sources {
        if let Err(e) = watcher.watch(src, RecursiveMode::NonRecursive) {
            missing.push(format!("{}: {e}", src.display()));
        }
    }
    if missing.len() == sources.len() {
        return Err(format!("Hiçbir kaynak klasör izlenemedi. {}", missing.join("; ")));
    }
    if !missing.is_empty() {
        let _ = app.emit("inbox-error", format!("Bazı kaynak klasörler izlenemedi: {}", missing.join("; ")));
    }
    // Otomatik tasima kapaliyken bekleyen dosyalar yerinde kalir; kuyruga alinmaz.
    if settings.scan_pending_on_start && settings.auto_move {
        for p in sources.iter().flat_map(|s| top_level_files(s)) {
            let _ = tx.send(p);
        }
    }
    drop(tx);

    let stop = Arc::new(AtomicBool::new(false));
    let stop_flag = Arc::clone(&stop);
    std::thread::Builder::new()
        .name("inbox-watcher".into())
        .spawn(move || worker(app, rx, stop_flag))
        .map_err(|e| e.to_string())?;
    Ok(Running { _watcher: watcher, stop })
}

fn worker(app: AppHandle, rx: mpsc::Receiver<PathBuf>, stop: Arc<AtomicBool>) {
    let mut pending = Pending::default();
    loop {
        match rx.recv_timeout(Duration::from_millis(700)) {
            Ok(p) => {
                if watcher_core::eligible(&p) {
                    pending.observe(p, Instant::now());
                }
                while let Ok(p) = rx.try_recv() {
                    if watcher_core::eligible(&p) {
                        pending.observe(p, Instant::now());
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
        if stop.load(Ordering::SeqCst) {
            break;
        }
        if pending.is_empty() || !automation::current(&app).running {
            continue; // duraklatildiysa dosyalar bekler
        }
        let ready = pending.take_ready(Instant::now(), STABLE_FOR);
        if !ready.is_empty() {
            process(&app, &ready);
        }
    }
}

fn process(app: &AppHandle, ready: &[PathBuf]) {
    let settings = app.state::<SettingsState>().settings.lock().map(|s| s.clone()).unwrap_or_default();
    let today = chrono::Local::now().date_naive();
    let mut update = InboxUpdate { moved: Vec::new(), failed: Vec::new() };
    if !settings.auto_move {
        // Dosya izlenen klasorde kalir; Bugun sayfasi yalnizca yenilenir.
        let _ = app.emit("inbox-updated", &update);
        return;
    }
    let state = app.state::<AppState>();
    for path in ready {
        let mut fail = |error: String| update.failed.push(FailedMove { path: path.to_string_lossy().into_owned(), error });
        let planned = match watcher_core::plan_inbox_move(&settings, today, path) {
            Ok(Some(m)) => m,
            Ok(None) => continue,
            Err(error) => { fail(error); continue; }
        };
        // Once gecmise yaz, sonra tasi: uygulama arada kapansa da tasima geri alinabilir.
        let Ok(mut h) = state.history.lock() else { fail("İşlem geçmişi kilitlenemedi.".into()); continue; };
        watcher_core::record_moves(&mut h, std::slice::from_ref(&planned), crate::now_ms());
        let result = history::save(&state.dir, &h).and_then(|_| crate::file_ops::transfer(path, Path::new(&planned.to)));
        match result {
            Ok(()) => update.moved.push(planned),
            Err(error) => {
                watcher_core::unrecord(&mut h, &planned);
                let _ = history::save(&state.dir, &h);
                fail(error);
            }
        }
    }
    if !update.moved.is_empty() || !update.failed.is_empty() {
        let _ = app.emit("inbox-updated", &update);
        crate::notifier::inbox_moved(app, &update.moved);
    }
}

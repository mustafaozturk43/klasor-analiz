//! Yeniden taramalarda ayni dosyalari tekrar hash'lememek icin onbellek. Bellekte tutulur;
//! tarama sonunda `hash_cache.json` olarak diske yazilir ve acilista geri yuklenir.
//! Anahtar: dosya yolu; gecerlilik: boyut + mtime (degismediyse hash ayni kalir).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

const FILE_NAME: &str = "hash_cache.json";

/// Onbellekte tutulacak en fazla kayit (asimda temizlenir).
const MAX_ENTRIES: usize = 200_000;

#[derive(Default, Clone, Serialize, Deserialize)]
struct CacheEntry {
    size: u64,
    mtime: u64,
    preview: Option<String>,
    full: Option<String>,
}

/// Thread-safe hash onbellegi (rayon par_iter icinden erisilir).
#[derive(Default)]
pub struct HashCache {
    map: Mutex<HashMap<String, CacheEntry>>,
    /// Son kayittan beri degisti mi? (degismediyse diske yazilmaz)
    dirty: AtomicBool,
}

impl HashCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Diskteki onbellegi yukler; dosya yok veya bozuksa bos baslar.
    pub fn load(dir: &Path) -> Self {
        let map = fs::read_to_string(dir.join(FILE_NAME)).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        Self { map: Mutex::new(map), dirty: AtomicBool::new(false) }
    }

    /// Degisiklik varsa onbellegi diske yazar (gecici dosya + yeniden adlandirma).
    pub fn save(&self, dir: &Path) -> Result<(), String> {
        if !self.dirty.swap(false, Ordering::SeqCst) {
            return Ok(());
        }
        let json = serde_json::to_string(&*self.map.lock().map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let tmp = dir.join(format!("{FILE_NAME}.tmp"));
        fs::write(&tmp, json).map_err(|e| e.to_string())?;
        fs::rename(&tmp, dir.join(FILE_NAME)).map_err(|e| e.to_string())
    }

    fn valid(e: &CacheEntry, size: u64, mtime: u64) -> bool {
        e.size == size && e.mtime == mtime
    }

    pub fn get_preview(&self, path: &str, size: u64, mtime: u64) -> Option<String> {
        let map = self.map.lock().ok()?;
        let e = map.get(path)?;
        if Self::valid(e, size, mtime) {
            e.preview.clone()
        } else {
            None
        }
    }

    pub fn get_full(&self, path: &str, size: u64, mtime: u64) -> Option<String> {
        let map = self.map.lock().ok()?;
        let e = map.get(path)?;
        if Self::valid(e, size, mtime) {
            e.full.clone()
        } else {
            None
        }
    }

    pub fn set_preview(&self, path: &str, size: u64, mtime: u64, hash: String) {
        self.upsert(path, size, mtime, |e| e.preview = Some(hash));
    }

    pub fn set_full(&self, path: &str, size: u64, mtime: u64, hash: String) {
        self.upsert(path, size, mtime, |e| e.full = Some(hash));
    }

    fn upsert(&self, path: &str, size: u64, mtime: u64, apply: impl FnOnce(&mut CacheEntry)) {
        let Ok(mut map) = self.map.lock() else {
            return;
        };
        if map.len() >= MAX_ENTRIES {
            map.clear();
        }
        let entry = map.entry(path.to_string()).or_default();
        // Boyut/mtime degistiyse onceki hash'ler gecersizdir.
        if entry.size != size || entry.mtime != mtime {
            entry.preview = None;
            entry.full = None;
            entry.size = size;
            entry.mtime = mtime;
        }
        apply(entry);
        self.dirty.store(true, Ordering::Relaxed);
    }
}

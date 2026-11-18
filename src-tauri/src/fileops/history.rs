//! Tasma gecmisinin JSON olarak kalici saklanmasi (app data dizininde).
//! En yeni kayit listenin basinda tutulur (undo her zaman bastan pop eder).
//! Yazim atomiktir (gecici dosya + yeniden adlandirma): yazim sirasinda uygulama
//! kapansa bile onceki gecmis bozulmaz. Okunamayan dosya silinmez, `.bozuk` adiyla
//! saklanir; boylece geri alma kayitlari sessizce kaybolmaz.

use crate::models::UndoRecord;
use std::fs;
use std::path::Path;

/// Gecmiste tutulacak en fazla islem sayisi. Karantinadaki "asil yer" bilgisi ve gelen
/// kutusu gelis saatleri de bu kayitlardan okunur.
pub const MAX_HISTORY: usize = 200;
const FILE_NAME: &str = "move_history.json";

/// Kayitli gecmisi diskten yukler. Dosya yoksa bos liste; bozuksa dosya
/// `move_history.json.bozuk` olarak ayrilir ve bos liste doner.
pub fn load(dir: &Path) -> Vec<UndoRecord> {
    let file = dir.join(FILE_NAME);
    let Ok(text) = fs::read_to_string(&file) else { return Vec::new() };
    match serde_json::from_str(&text) {
        Ok(records) => records,
        Err(_) => {
            let _ = fs::rename(&file, dir.join(format!("{FILE_NAME}.bozuk")));
            Vec::new()
        }
    }
}

/// Gecmisi diske yazar (once gecici dosya, sonra yeniden adlandirma).
pub fn save(dir: &Path, history: &[UndoRecord]) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string(history).map_err(|e| e.to_string())?;
    let tmp = dir.join(format!("{FILE_NAME}.tmp"));
    fs::write(&tmp, json).map_err(|e| e.to_string())?;
    fs::rename(&tmp, dir.join(FILE_NAME)).map_err(|e| e.to_string())
}

/// En yeni kaydi listenin basina ekler ve siniri asanlari atar.
pub fn push(history: &mut Vec<UndoRecord>, record: UndoRecord) {
    history.insert(0, record);
    history.truncate(MAX_HISTORY);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_is_atomic_and_corrupt_file_is_kept_aside() {
        let dir = std::env::temp_dir().join(format!("sfo_history_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        assert!(load(&dir).is_empty(), "dosya yoksa bos");
        let rec = UndoRecord { id: "MV-1".into(), timestamp: 1, tag: "Test".into(), moves: Vec::new() };
        save(&dir, std::slice::from_ref(&rec)).unwrap();
        assert_eq!(load(&dir)[0].id, "MV-1");
        assert!(!dir.join("move_history.json.tmp").exists(), "gecici dosya kalmaz");

        fs::write(dir.join(FILE_NAME), b"{ bozuk").unwrap();
        assert!(load(&dir).is_empty());
        assert_eq!(fs::read(dir.join("move_history.json.bozuk")).unwrap(), b"{ bozuk", "bozuk dosya saklanir");
        fs::remove_dir_all(&dir).unwrap();
    }
}

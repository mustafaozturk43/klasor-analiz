//! Windows Shell komutlari: kucuk resimler ve cekim tarihleri. Isler en cok 4 is
//! parcacigina bolunur (her biri kendi COM baglamini acar); sonuc sirasi korunur.

use crate::media_dates::date_taken;
use crate::shell_thumb::shell_thumbnail;
use std::path::Path;
use std::sync::Once;
use tauri::{AppHandle, Manager};

fn par_map<T: Send>(paths: &[String], f: impl Fn(&Path) -> Option<T> + Sync) -> Vec<Option<T>> {
    let mut out: Vec<Option<T>> = (0..paths.len()).map(|_| None).collect();
    let chunk = paths.len().div_ceil(4).max(1);
    let f = &f;
    std::thread::scope(|s| {
        for (ps, os) in paths.chunks(chunk).zip(out.chunks_mut(chunk)) {
            s.spawn(move || ps.iter().zip(os).for_each(|(p, o)| *o = f(Path::new(p))));
        }
    });
    out
}

/// Her yol icin onbellekteki PNG yolu (`None`: Windows resim veremedi).
#[tauri::command(async)]
pub fn get_shell_thumbnails(app: AppHandle, paths: Vec<String>, size: u32) -> Vec<Option<String>> {
    crate::asset_scope::allow_thumbs(&app);
    let cache = app.path().app_cache_dir().unwrap_or_else(|_| std::env::temp_dir()).join("thumbs").join("shell");
    static PRUNE: Once = Once::new();
    PRUNE.call_once(|| { crate::thumbnail::prune_cache(&cache, crate::thumbnail::MAX_CACHE_AGE); });
    par_map(&paths, |p| shell_thumbnail(p, size, &cache).map(|x| x.to_string_lossy().into_owned()))
}

/// Her yol icin cekim tarihi (unix saniye, UTC) ya da `None`.
#[tauri::command(async)]
pub fn read_media_dates(paths: Vec<String>) -> Vec<Option<u64>> {
    par_map(&paths, date_taken)
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "elle: SFO_SAMPLE_PHOTO=<gercek fotograf yolu>"]
    fn prints_date_taken_of_sample_photo() {
        let p = std::env::var("SFO_SAMPLE_PHOTO").expect("SFO_SAMPLE_PHOTO");
        let got = super::par_map(&[p.clone()], crate::media_dates::date_taken);
        println!("DATE_TAKEN {p} => {:?}", got[0]);
        assert!(got[0].is_some());
    }
}

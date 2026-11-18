//! BLAKE3 tabanli hashing fonksiyonlari (huni yaklasimi + buyuk dosya orneklemesi).

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const MB: u64 = 1024 * 1024;
/// Ilk 4 KB on-hash boyutu.
const PREVIEW_BYTES: usize = 4096;
/// Buyuk dosyalarda orneklenen parca boyutu (1 MB).
const SAMPLE_BYTES: u64 = MB;

/// Dosyayi tamponla okuyarak tam BLAKE3 hash dondurur.
pub fn hash_full(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; 128 * 1024];

    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Dosyanin ilk 4 KB'lik kisminin hizli hash'ini dondurur.
pub fn hash_preview(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; PREVIEW_BYTES];
    let n = read_up_to(&mut file, &mut buf)?;
    Ok(blake3::hash(&buf[..n]).to_hex().to_string())
}

/// 500 MB ve uzeri dosyalar icin akilli ornekleme hash'i:
/// Ilk 1 MB + Tam Orta 1 MB + Son 1 MB + boyut => blake3 kimligi.
pub fn hash_large(path: &Path, size: u64) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; SAMPLE_BYTES as usize];

    // Ilk 1 MB
    let n = read_up_to(&mut file, &mut buf)?;
    hasher.update(&buf[..n]);

    // Tam orta 1 MB
    let mid_start = size / 2 - SAMPLE_BYTES / 2;
    file.seek(SeekFrom::Start(mid_start)).map_err(|e| e.to_string())?;
    let n = read_up_to(&mut file, &mut buf)?;
    hasher.update(&buf[..n]);

    // Son 1 MB
    let last_start = size - SAMPLE_BYTES;
    file.seek(SeekFrom::Start(last_start)).map_err(|e| e.to_string())?;
    let n = read_up_to(&mut file, &mut buf)?;
    hasher.update(&buf[..n]);

    // Dosya boyutunu da kimlige kat (ayni ornekler farkli boyutlari ayirir).
    hasher.update(&size.to_le_bytes());

    Ok(hasher.finalize().to_hex().to_string())
}

/// Tampon dolana veya dosya bitene kadar okur; okunan byte sayisini dondurur.
fn read_up_to(file: &mut File, buf: &mut [u8]) -> Result<usize, String> {
    let mut total = 0usize;
    while total < buf.len() {
        let n = file.read(&mut buf[total..]).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        total += n;
    }
    Ok(total)
}

# AGENTS.md - Smart File Organizer

## Kimsin?
**Smart File Organizer** projesinde çalışan bir geliştirme asistanısın. Proje bir Windows masaüstü uygulamasıdır:
- Arka uç: Tauri v2 (Rust) — `src-tauri/src/`
- Ön uç: React 19 + TypeScript (Vite) — `src/`
- Stil: Tailwind CSS v4, `lucide-react`; yeşil-amber palet (koyu cam yeşili kenar çubuğu, amber birincil düğme), Rethink Sans / Hanken Grotesk, açık ve koyu tema, sade ve yoğun olmayan düzen (`src/styles/tokens.css`, `tasarım_rehberi.md`)

## Ne yapıyorsun?
Uygulama klasörleri tarar, birebir kopya (`Exact`) ve isim benzerliği (`Similar`) gruplarını bulur, dosyaları etiket klasörlerine taşır (Undo destekli) ve Gelen Kutusu otomasyonuyla yeni dosyaları düzenler. Bu özelliklerin ayrıntılı kuralları skill'lerdedir; ilgili alanda çalışırken ilgili skill'i kullan:

| Skill | Ne zaman |
|---|---|
| `proje-kodlama-standartlari` | Yeni dosya, bağımlılık veya arayüz yazarken (teknoloji yığını, UI, 250 satır kuralı) |
| `tarama-ve-gruplama` | Kategoriler, `jwalk`/`rayon` tarama, `blake3` hash, 500 MB kuralı, Exact/Similar gruplama |
| `etiketleme-tasima-undo` | Etiket klasörü, taşıma, karantina, numaralandırma, Undo |
| `gelen-kutusu` | `settings.json`, watcher, Akşam Kontrolü, Bugün önerileri, Geri Dönüşüm Kutusu istisnası |

## Yapmaman gerekenler
- `npm` dışında paket yöneticisi (pnpm, yarn, bun) kullanma.
- `node_modules/`, `src-tauri/target/`, `.git/` ve `dist/` klasörlerini okuma veya tarama.
- 250 satırı aşan, 300 karakterden uzun satır içeren veya birden fazla sorumluluğu olan dosya yazma (Rust ve React için geçerli; `npm run audit:source` denetler).
- Küçük değişiklik için dosyayı baştan yazma; yalnızca ilgili modülü güncelle.
- Hedefte aynı adlı dosyanın üzerine yazma; otomatik numaralandır.
- Kalıcı silme yapma (`fs::remove_file` / `remove_dir_all` kullanıcı dosyalarında kullanılmaz). Tek istisna: kullanıcı onayıyla Gelen Kutusu ve Karantina dosyalarının Windows Geri Dönüşüm Kutusu'na gönderilmesi.
- Kullanıcı onayı olmadan hiçbir dosyayı taşıma veya silme; öneriler yalnızca başlangıç değeridir.
- Ayarlar için `settings.json` dışında ikinci bir kaynak oluşturma. Arayüz tercihleri (tema, son klasörler, yasaklı uzantılar, düzenleme profilleri) de oradadır (`uiPrefs`, `src/lib/prefs.ts`); `localStorage` yalnızca temanın ilk boyama önbelleğidir.

---
name: gelen-kutusu
description: Smart File Organizer'ın Akıllı Otomasyon (Gelen Kutusu) özelliği; settings.json ayarları, notify watcher, _GelenKutusu klasör yapısı, Akşam Kontrolü, Geri Dönüşüm Kutusu istisnası ve Bugün önerileri (inbox_dupes.rs, inbox_suggest.rs). Bu alanda kod yazılırken veya değiştirilirken kullan.
---

# Gelen Kutusu (Akıllı Otomasyon)

## Ayarlar
- Ayarlar Rust tarafında `settings.json` (uygulama veri dizini) içinde tutulur; **tek kaynak budur**. Arayüz tercihleri de aynı dosyadadır (`uiPrefs`); onları yalnızca `set_ui_pref` değiştirir, `save_settings` ön yüzden gelen kopyayı yok sayar.
- 5 uzantı grubu (Dokümanlar, Görseller, Videolar, Arşivler, Kurulum) yalnızca gelen kutusu içindir. "Türüne Göre Düzenle" kuralları ayrı kalır.
- Bir uzantı tek gruba bağlanır.
- Yedek klasörü ile kaynak klasörler iç içe olamaz.
- `_GelenKutusu`, `_Kopyalar_Karantina` ve `Diğer` grup adı olamaz.

## Watcher ve taşıma
- Watcher (`notify`) kaynak klasörleri **alt klasörsüz** izler.
- Dosya, boyutu ve değişiklik zamanı **2 saniye** sabit kalınca `<Yedek>/_GelenKutusu/<YYYY-MM-DD>_Bakilacaklar/<Grup>/` içine taşınır. Gün **yerel saatle** belirlenir.
- Geçici/sistem dosyaları alınmaz: `.crdownload`, `.part`, `~$`, `desktop.ini`, `.` ile başlayanlar.
- Hiçbir gruba uymayan dosya, "Diğer" açık değilse **kaynakta kalır**.
- **Otomatik taşı** (`autoMove`, varsayılan açık): kapalıyken watcher dosyaya dokunmaz, yalnızca `inbox-updated` gönderir; dosya izlenen klasörde kalır. `list_inbox` seçili günde **oluşturulmuş**, kaynak klasörlerin ilk seviyesindeki uygun dosyaları `pending: true` ve "İzlenen · <klasör>" grubuyla listeye ekler (`inbox_sources.rs`). Anahtar hem Kurallar > Otomasyon kartında hem Bugün başlığında (`set_auto_move`) bulunur.
- Gelen kutusunda çakışma numarası **`dosya_1.pdf`** biçimindedir; diğer tüm işlemlerde `dosya (1).pdf` kullanılır.
- Otomatik taşımalar önce geçmişe yazılır, sonra taşınır. Aynı dakikadaki taşımalar tek geçmiş kaydında (`Otomatik yerleştirme · Gelen kutusu`) birleştirilir.
- Pencere kapatılınca otomasyon açıksa uygulama tepside çalışmaya devam eder; tek kopya çalışır.
- `startWithWindows` ayarı `startup.rs` ile Windows başlangıç kaydına yansıtılır (kaynak yine `settings.json`); `--autostart` ile açılan kopya otomasyon açıksa pencere göstermez. Pencere `tauri.conf.json` içinde gizli oluşturulur, `startup::on_launch` gösterir.
- `notifyMoves` açıkken ve pencere önde değilken `notifier.rs` her yerleştirme için bildirim gösterir; bildirim yalnızca bilgi verir.

## Akşam Kontrolü
- **Geri Dönüşüm Kutusu istisnası:** Kalıcı silme yoktur. Akşam Kontrolü'ndeki "Sil" yalnızca kullanıcı onayıyla ve yalnızca `_GelenKutusu` altındaki ya da izlenen klasörde bekleyen dosyaları Windows Geri Dönüşüm Kutusu'na gönderir. Bu işlem İşlem Geçmişi'ne yazılmaz ve uygulama içinden geri alınamaz.
- Taşıma yalnızca gelen kutusu dosyalarına ve izlenen klasörde bekleyen (kaynak klasörün doğrudan içindeki) dosyalara uygulanır, geçmişe yazılır ve geri alınabilir. Hedef herhangi bir klasör olabilir; gelen kutusu ve karantina (içi dahil) hedef olamaz. Dosya zaten hedef klasördeyse atlanır.

## Bugün önerileri
- Her dosya için Rust (`inbox_dupes.rs`, `inbox_suggest.rs`) bir başlangıç kararı önerir:
  1. Birebir kopya → **At**
  2. Adı bir hedef klasördeki dosyaya (Jaro-Winkler >= 0.82) benziyorsa → o klasöre **Taşı**
  3. Benzer adlı dosyayla birlikte **Taşı**
  4. Aksi halde **Tut**
- Öneri yalnızca başlangıç değeridir. Hiçbir şey kullanıcı "kararı uygula" demeden taşınmaz ya da silinmez. At için onay penceresi çıkar.
- Taşıma hedefi alttaki uygulama çubuğundan Haritalar'daki `TargetFolderSelect` ile seçilir: "Önerilen klasörler (dosya başına)", "Bu klasör" (bekleyen dosyanın klasörü), Kökler (yedek kökü, izlenen klasörler), yedek kökündeki klasörler, diğer konumlar ve "Başka bir klasör seç…". "Önerilen" seçiliyken her dosya kendi önerilen klasörüne gider; bir hedef seçilirse tüm "Taşı" dosyaları oraya gider. "Grup adı (alt klasör)" doluysa dosyalar hedefin içindeki o alt klasöre taşınır (yoksa oluşturulur; `move_selected_files` `group`).

# Smart File Organizer

Tauri v2, Rust, React 19 ve TypeScript tabanlı klasör analizi ve dosya düzenleme uygulaması.

## Kurulum

Uygulama yalnızca Windows 10/11 (x64) için geliştirilir.

**Gereksinimler**

- [Node.js](https://nodejs.org/) 22 veya üzeri (LTS) ve `npm`. Başka paket yöneticisi (pnpm, yarn, bun) kullanılmaz.
- [Rust](https://rustup.rs/) 1.85 veya üzeri (`edition = "2024"`), MSVC araç zinciriyle (`stable-x86_64-pc-windows-msvc`).
- Visual Studio Build Tools: "Desktop development with C++" iş yükü ve Windows SDK.
- Microsoft Edge WebView2 Runtime (Windows 11'de hazır gelir).

**Adımlar**

```powershell
git clone https://github.com/mustafaozturk43/klasor-analiz.git
cd klasor-analiz
npm install
npm run tauri dev
```

İlk çalıştırmada Rust bağımlılıkları derlendiği için birkaç dakika sürebilir. `npm run tauri dev` Vite'ı `http://localhost:1420` adresinde başlatır; bu port doluysa önce onu boşaltın. Üretim paketi için aşağıdaki "Windows paketleri" bölümüne bakın.

Antivirüs yazılımı yeni derlenen build-script `.exe` dosyalarını kilitleyip derlemeyi durdurabilir (`os error 32` veya `os error 5`). Böyle olursa korumayı kısa süre duraklatıp derlemeyi yeniden deneyin.

## Çalıştırma

```powershell
npm install
npm run tauri dev
```

Doğrulama: proje kökünde `npm run build`, `src-tauri` içinde `cargo test --no-default-features --lib` (antivirüs build-script kilidine takılmamak için; tam derleme `cargo test`).

Kaynak bağlantılarını, 250 satır ve 300 karakterlik satır sınırını denetleme: `npm run audit:source`. Hepsi tek komutta (derleme + denetim + Rust testleri): `npm run check`.
Açık Windows uygulaması debug `.exe` dosyasını kilitliyorsa uygulamayı kapatmadan alternatif test: `src-tauri` içinde `cargo test --release` (ayrı release çıktısı kullanılır).

## Windows paketleri (setup + portable)

`npm run release` uygulamayı tek sefer derler ve `artifacts/` altına hem kurulum dosyasını (`…-setup-<tarih>.exe`, NSIS, yönetici izni istemeden kullanıcıya kurulur, Türkçe/İngilizce) hem de portable ZIP paketini SHA256 dosyalarıyla birlikte koyar. Yalnızca kurulum dosyası için `npm run setup`, yalnızca portable için `npm run portable` kullanılır. Uygulama tek kopya çalışır: zaten açıkken yeniden başlatılırsa açık pencere öne gelir.

### Portable Windows paketi

`npm run portable`, production Tauri EXE'sini kurulum paketi oluşturmadan derler; `artifacts/` altında tarihli Windows x64 klasörü, ZIP ve SHA256 dosyaları hazırlar. Paket kişisel dosya/işlem geçmişi içermez. Microsoft Edge WebView2 Runtime gereklidir; ayarlar ve geçmiş EXE yanında değil Windows kullanıcı profilinde saklanır. Paket dijital olarak imzalı değildir.

## Ekranlar

Kenar çubuğunda 5 hedef var; birkaçının içinde alt sekmeler bulunur (`src/lib/navigation.ts`).

- **Bugün:** günün gelen kutusu. Her dosya için **Tut / Taşı / At** kararı verilir (varsayılan Tut); sonra "Kararları uygula" tek dokunuşla hepsini yapar. Taşınanlar hedef klasöre gider ve İşlem Geçmişi'nden geri alınır; "At" onaylı olarak Windows Geri Dönüşüm Kutusu'na gönderilir. Gruplar, küçük resimler, metin özeti ve benzer isimliler satırlarda görünür; "Taşınacakları adlandır" şablonla adlandırır.
- **Kopyalar:** Dosya grupları (solda süzülebilir liste, sağda ana dosya ve kopyaların karşılaştırması, saklama politikası, "Bu grubu taşı"; kutularla toplu seçim alt çubuktan), İki klasör karşılaştırma ve Karantina.
- **Harita:** Klasör analizi (boyutla orantılı kutu haritası, türe göre alan, Temizlenecekler özeti), Klasördeki dosyalar (arama, sayfalama, ön izleme) ve Klasör sağlığı.
- **Kurallar:** Kurallar (otomasyon, kaynak ve yedek klasörleri, 5 uzantı grubu ve sağda "Türüne göre düzenle": seçenekler, profiller, önizleme ve uygulama) ve Otomasyon durumu (izleme durumu, şablonla adlandırma).
- **Geçmiş:** taşıma ve yeniden adlandırmaları geri alma.

Görünüm: koyu cam yeşili kenar çubuğu, açık yeşilimsi zemin, amber birincil düğme. Yazı tipleri (Rethink Sans, Hanken Grotesk, JetBrains Mono) uygulamayla birlikte paketlenir, internet gerekmez. Açık, koyu ve otomatik tema desteklenir.
## Akıllı otomasyon (gelen kutusu)

Kurallar ekranında kaynak klasör (ör. İndirilenler) ve yedek klasörü seçilip otomasyon açılınca, kaynağa düşen dosyalar yazımı bitince (2 sn sabit kalınca) `<Yedek>\_GelenKutusu\<YYYY-MM-DD>_Bakilacaklar\<Grup>\` içine taşınır. Gün yerel saatle belirlenir; aynı adlı dosyalar `ad_1.uzantı` olur; hiçbir gruba uymayanlar kaynakta kalır. Taşımalar dakikalık gruplar halinde İşlem Geçmişi'ne yazılır ve geri alınabilir.

Pencere kapatıldığında otomasyon açıksa uygulama sistem tepsisinde çalışmaya devam eder (tepsi menüsü: Paneli aç, Duraklat/Devam et, Bugünün klasörünü aç, Çıkış). Ayarlar ve arayüz tercihleri (tema, son klasörler, yasaklı uzantılar, düzenleme profilleri) uygulama veri dizinindeki `settings.json` dosyasında tutulur. Kurallar ekranındaki "Windows açılınca başlat" anahtarı uygulamayı kullanıcı başlangıç kaydına (HKCU `Run`) ekler; otomasyon açıksa pencere göstermeden tepside başlar. "Otomatik taşımaları bildir" açıkken, pencere önde değilse her yerleştirmede bir Windows bildirimi çıkar. İkisi de varsayılan olarak kapalıdır ve `settings.json` içinde tutulur (`startWithWindows`, `notifyMoves`). Akşam Kontrolü'ndeki "Sil" kalıcı silmez; yalnızca gelen kutusu dosyalarını onayla Geri Dönüşüm Kutusu'na gönderir ve bu işlem uygulama içinden geri alınamaz.

## Güvenlik ve davranış

Taşıma, grup kurallarını korur: Exact ana dosya etikete, kopyalar karantinaya; Similar tüm üyeler etikete. Her işlem önce onaylanır; dosyaların üzerine yazılmaz. Kısmi işlemler geçmişte tutulur. 500 MiB üzerindeki örnek hash eşleşmeleri tam içerik eşitliğinin kanıtı değildir; tam doğrulama seçeneği kullanılabilir.

Tarihle isimlendirme `dosya_DD-MM-YYYY.uzantı` biçimini, gerçek oluşturulma tarihini ve yerel saatle takvim gününü kullanır (gelen kutusu gün klasörüyle aynı saat dilimi). Tarihi okunamayan dosyalar atlanır; değişiklik tarihi yerine geçirilmez. Ön izleme planı backend'de tutulur, uygulama öncesinde dosya ve hedef yeniden denetlenir.

Karşılaştırmada aynı veya iç içe kökler reddedilir. Eşleşme grupları A ve B arasında en az bir üye içerir. Eşleşmeyen dosyalar da envanterde gösterilir. Dosya seçimi isimlendirme içindir; grup taşıma için tam gruplar seçilir.

## Mimari ve tasarım

- [Tasarım rehberi](tasarım_rehberi.md)
- [Proje mimarisi](proje_mimarisi.md)
- [Görsel prototip](mockup-organizer.html) — örnek verili tasarım referansı, gerçek uygulama değildir.

`src/components/Workspace*` uygulama kabuğunu; `FolderAnalysis`, `CompareView`, `FileBrowser`, `GroupsView` modülleri ekranları oluşturur. Rust tarafında `compare`, `folder_stats`, `rename`, `group_moves` işlevleri ayrı tutulur. Kaynak dosyaları 250 satırı, satırlar 300 karakteri geçmez.

Sayfalama şu anda istemci tarafındadır; büyük envanterlerde IPC veri aktarımını küçültmez. Liste satırları ayrıca sanallaştırılır. Resimlerde yerleşik ön izleme, diğer türlerde metadata ve varsayılan uygulamada açma bulunur.

Karantinadaki "Kutuya gönder" ve "Tümünü temizle" kalıcı silmez; dosyaları onayla Windows Geri Dönüşüm Kutusu'na gönderir. Okunamayan (kilitli, erişim izni olmayan) dosyalar taramayı durdurmaz; atlanır ve tarama özetinde bildirilir. Sayaçla ayrılan adlar (`IMG_0001`/`IMG_0002`) isim benzeri sayılmaz. Çalıştırılabilir dosyalar (exe, bat, lnk …) uygulama içinden açılmaz; "Klasörde göster" kullanılır. İşlem Geçmişi son 200 işlemi tutar ve atomik yazılır. CSV raporlarında `=`, `+`, `-`, `@` ile başlayan hücreler metne çevrilir. Düzenleme önizlemesinin iptali taramanın iptalinden bağımsızdır.

Hash önbelleği tarama sonunda uygulama veri dizinine (`hash_cache.json`) yazılır; uygulama yeniden açıldığında değişmemiş dosyalar tekrar hash'lenmez. Yalnızca çevrimiçi duran bulut dosyaları (OneDrive) kopya taramasında ve A↔B karşılaştırmasında indirilmemek için atlanır ve tarama özetinde bildirilir. Geçmiş ekranındaki "CSV dışa aktar" tüm taşımaları dosyaya yazar. Karantinadaki "30 günden eskileri seç" yalnızca seçim yapar; gönderme yine onaylıdır. Bozuk `settings.json` dosyasının kopyası `settings.json.bozuk` olarak saklanır. Arayüz bir içerik güvenlik politikasıyla (CSP) çalışır: yalnızca paketle gelen betik, stil ve yazı tipleri yüklenir.

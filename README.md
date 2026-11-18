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

Testler: `npm test` ön yüz saf mantığını (`*.test.ts`, vitest), `cargo test` Rust modüllerini çalıştırır; ikisi de `npm run check` içindedir. 5.000 dosyalı Tarihe Göre hız testi yavaş olduğu için ayrıdır: `npm run test:slow` (~40 sn, süre `--nocapture` ile yazılır).

Kaynak bağlantılarını, 250 satır ve 300 karakterlik satır sınırını denetleme: `npm run audit:source`. Hepsi tek komutta (derleme + denetim + Rust testleri): `npm run check`.
GitHub Actions (`.github/workflows/check.yml`) her push ve pull request'te `npm run check` çalıştırır. `artifacts/` altındaki eski sürüm paketlerini listelemek için `npm run prune:artifacts` (en yeni 3 sürüm tutulur; `-- --keep=5` ile değişir), silmek için `npm run prune:artifacts -- --yes`.
Açık Windows uygulaması debug `.exe` dosyasını kilitliyorsa uygulamayı kapatmadan alternatif test: `src-tauri` içinde `cargo test --release` (ayrı release çıktısı kullanılır).

## Windows paketleri (setup + portable)

`npm run release` uygulamayı tek sefer derler ve `artifacts/` altına hem kurulum dosyasını (`…-setup-<tarih>.exe`, NSIS, yönetici izni istemeden kullanıcıya kurulur, Türkçe/İngilizce) hem de portable ZIP paketini SHA256 dosyalarıyla birlikte koyar. Yalnızca kurulum dosyası için `npm run setup`, yalnızca portable için `npm run portable` kullanılır. Uygulama tek kopya çalışır: zaten açıkken yeniden başlatılırsa açık pencere öne gelir.

### Portable Windows paketi

`npm run portable`, production Tauri EXE'sini kurulum paketi oluşturmadan derler; `artifacts/` altında tarihli Windows x64 klasörü, ZIP ve SHA256 dosyaları hazırlar. Paket kişisel dosya/işlem geçmişi içermez. Microsoft Edge WebView2 Runtime gereklidir; ayarlar ve geçmiş EXE yanında değil Windows kullanıcı profilinde saklanır. Paket dijital olarak imzalı değildir.

## Ekranlar

Kenar çubuğunda 8 hedef var; birkaçının içinde alt sekmeler bulunur (`src/lib/navigation.ts`). Kopyalar, Harita, Adlandırma, Tarihe Göre ve Kural motoru "Klasör-Dosya İşlemleri" grubunun altındadır.

- **Bugün:** günün gelen kutusu. Her dosya için **Tut / Taşı / At** kararı verilir (varsayılan Tut); sonra "Kararları uygula" tek dokunuşla hepsini yapar. Taşınanlar hedef klasöre gider ve İşlem Geçmişi'nden geri alınır; "At" onaylı olarak Windows Geri Dönüşüm Kutusu'na gönderilir. Gruplar, küçük resimler, metin özeti ve benzer isimliler satırlarda görünür; "Taşınacakları adlandır" şablonla adlandırır.
- **Kopyalar:** Dosya grupları (solda süzülebilir liste, sağda ana dosya ve kopyaların karşılaştırması, saklama politikası, "Bu grubu taşı"; kutularla toplu seçim alt çubuktan), İki klasör karşılaştırma, Benzer görseller (aynı fotoğrafın yeniden boyutlandırılmış, sıkıştırılmış ya da hafifçe düzenlenmiş kopyaları; görsel parmak izi/dHash, hassasiyet seçilir, her grupta en yüksek çözünürlüklü görsel kalır, seçilenler karantinaya gider) ve Karantina.
- **Harita:** Klasör analizi (boyutla orantılı kutu haritası, türe göre alan, Temizlenecekler özeti), Klasördeki dosyalar (arama, sayfalama, ön izleme) ve Klasör sağlığı.
- **Adlandırma:** klasör ve dosya adlarını kurallarla (harf biçimi, boşluk, Türkçe harf, sıra numarası, tarih) toplu düzenler; önce önizleme, geri alınabilir.
- **Tarihe Göre:** dosyaları çekim (yoksa oluşturma) tarihine göre `Yıl\Ay`, `Yıl-Ay` veya sıra numaralı yıl klasörlerine toplar. Taşımadan önce kök klasöre `Tarih_Dizini.html` yazar: klasör ve dosya adları, arama kutusu, dosya adına tıklayınca klasör açılır. Taşıma sırasında yüzde ilerleme gösterilir; Geçmiş'ten geri alınır. "Dizini aç" ve "Dizini yenile" düğmeleri dizini tarayıcıda açar veya alt klasörlerin şimdiki durumundan baştan yazar.
- **Kural motoru:** seçili klasörde kendi kurallarını çalıştırır. Bir kural; uzantı, ad içeriyor, boyut (MB) ve yaş (gün) koşullarının hepsini (en az biri zorunlu) ve tek eylemi (klasöre taşı, Yıl\Ay klasörüne taşı, Geri Dönüşüm Kutusu'na gönder) içerir. Kurallar yukarıdan aşağıya denenir, ilk eşleşen kazanır. Önizleme canlıdır; taşıma Geçmiş'ten geri alınır. Geri Dönüşüm Kutusu eylemi ayrı bir ikinci onay ister ve geri alınamaz. Kurallar `settings.json` içindeki `uiPrefs.ruleEngine` alanında saklanır. Gelen Kutusu'na dokunmaz.
- **Kurallar:** Kurallar (otomasyon, kaynak ve yedek klasörleri, 5 uzantı grubu ve sağda "Türüne göre düzenle": seçenekler, profiller, önizleme ve uygulama) ve Otomasyon durumu (izleme durumu, şablonla adlandırma).
- **Geçmiş:** taşıma ve yeniden adlandırmaları geri alma.

Görünüm: koyu cam yeşili kenar çubuğu, açık yeşilimsi zemin, amber birincil düğme. Yazı tipleri (Rethink Sans, Hanken Grotesk, JetBrains Mono) uygulamayla birlikte paketlenir, internet gerekmez. Açık, koyu ve otomatik tema desteklenir.
## Akıllı otomasyon (gelen kutusu)

Kurallar ekranında kaynak klasör (ör. İndirilenler) ve yedek klasörü seçilip otomasyon açılınca, kaynağa düşen dosyalar yazımı bitince (2 sn sabit kalınca) `<Yedek>\_GelenKutusu\<YYYY-MM-DD>_Bakilacaklar\<Grup>\` içine taşınır. Gün yerel saatle belirlenir; aynı adlı dosyalar `ad_1.uzantı` olur; hiçbir gruba uymayanlar kaynakta kalır. Taşımalar dakikalık gruplar halinde İşlem Geçmişi'ne yazılır ve geri alınabilir.

Pencere kapatıldığında otomasyon açıksa uygulama sistem tepsisinde çalışmaya devam eder (tepsi menüsü: Paneli aç, Duraklat/Devam et, Bugünün klasörünü aç, Çıkış). Ayarlar ve arayüz tercihleri (tema, son klasörler, yasaklı uzantılar, düzenleme profilleri) uygulama veri dizinindeki `settings.json` dosyasında tutulur. Kurallar ekranındaki "Windows açılınca başlat" anahtarı uygulamayı kullanıcı başlangıç kaydına (HKCU `Run`) ekler; otomasyon açıksa pencere göstermeden tepside başlar. "Otomatik taşımaları bildir" açıkken, pencere önde değilse her yerleştirmede bir Windows bildirimi çıkar. İkisi de varsayılan olarak kapalıdır ve `settings.json` içinde tutulur (`startWithWindows`, `notifyMoves`). Akşam Kontrolü'ndeki "Sil" kalıcı silmez; yalnızca gelen kutusu dosyalarını onayla Geri Dönüşüm Kutusu'na gönderir ve bu işlem uygulama içinden geri alınamaz.

## Komut paleti (Ctrl+K)

Üst çubuktaki arama düğmesi ya da **Ctrl+K** komut paletini açar (girdi alanlarındayken de çalışır). Yazarak şunlara gidersiniz: tüm ekranlar ve alt sekmeleri, son klasörler ("Klasörü aç: …"), Taramayı başlat, Son işlemi geri al ve Tema. Arama Türkçe harf ve büyük/küçük harf farkını yok sayar ("gecmis" → "Geçmiş"); ↑ ↓ gezer, Enter seçer, Esc kapatır. Tarama ya da taşıma sürerken ilgili işlemler soluk görünür ve seçilemez. Mantık `src/lib/palette.ts`, arayüz `src/components/CommandPalette.tsx`.

## Raporlar (CSV)

Excel uyumlu CSV (UTF-8, noktalı virgül) dışa aktarma düğmeleri: Kopyalar > Dosya grupları ve Geçmiş (mevcut), ayrıca **Harita > Klasör sağlığı** (dört bölüm tek tabloda), **Kopyalar > Benzer görseller** (grup, önerilen, çözünürlük, benzerlik %) ve **Kural motoru** önizlemesi (hangi dosya hangi kurala göre nereye gidecek). Hepsi "CSV'ye aktar" ile kaydet diyaloğu açar; formül gibi başlayan hücreler (`=`, `+`, `-`, `@`) metne çevrilir. Ekranlar satırları hazırlar, yazma tek komutla yapılır (`export_table_csv`, `organizer/table_report.rs`).

## Güvenlik ve davranış

Taşıma, grup kurallarını korur: Exact ana dosya etikete, kopyalar karantinaya; Similar tüm üyeler etikete. Her işlem önce onaylanır; dosyaların üzerine yazılmaz. Kısmi işlemler geçmişte tutulur. 500 MiB üzerindeki örnek hash eşleşmeleri tam içerik eşitliğinin kanıtı değildir; tam doğrulama seçeneği kullanılabilir.

Tarihle isimlendirme `dosya_DD-MM-YYYY.uzantı` biçimini, gerçek oluşturulma tarihini ve yerel saatle takvim gününü kullanır (gelen kutusu gün klasörüyle aynı saat dilimi). Tarihi okunamayan dosyalar atlanır; değişiklik tarihi yerine geçirilmez. Ön izleme planı backend'de tutulur, uygulama öncesinde dosya ve hedef yeniden denetlenir.

Karşılaştırmada aynı veya iç içe kökler reddedilir. Eşleşme grupları A ve B arasında en az bir üye içerir. Eşleşmeyen dosyalar da envanterde gösterilir. Dosya seçimi isimlendirme içindir; grup taşıma için tam gruplar seçilir.

## Mimari ve tasarım

- [Tasarım rehberi](tasarım_rehberi.md)
- [Proje mimarisi](proje_mimarisi.md)
- [Görsel prototip](mockup-organizer.html) — örnek verili tasarım referansı, gerçek uygulama değildir.

### Klasör yapısı

Ön yüz (`src/`): ekrana özel bileşen, hook ve komut dosyaları `src/features/<ekran>/` altındadır (`today`, `duplicates`, `map`, `rename`, `dates`, `rules`, `history`). Birden fazla ekranın kullandığı parçalar (kabuk, gezinme, ortak hook ve yardımcılar) `src/components`, `src/hooks` ve `src/lib` altında kalır. Bir dosya yalnızca tek ekran kullanıyorsa o ekranın klasörüne konur.

Arka uç (`src-tauri/src/`): modüller konuya göre klasörlere ayrılır; `lib.rs` hepsini eski adlarıyla yeniden dışa aktarır, yani `crate::scanner` gibi yollar değişmez. Testler (`*_tests.rs`) modülünün yanındadır.

| Klasör | İçerik |
| --- | --- |
| `scan/` | tarama, hash, gruplama, benzerlik, karşılaştırma, dosya listesi, klasör istatistikleri, sağlık, bulut yer tutucuları |
| `inbox/` | gelen kutusu: kopya/öneri/ad, ayrıntı, Akşam Kontrolü işlemleri, Geri Dönüşüm |
| `auto/` | otomasyon, watcher, bildirim, başlangıç, tepsi, tek kopya, `settings.json` |
| `organizer/` | "Türüne göre düzenle" ve CSV rapor |
| `renaming/` | adlandırma kuralları, şablonla adlandırma, seçili dosyaları adlandırma |
| `by_date/` | Tarihe Göre gruplama: plan, klasör düzeni, HTML dizin |
| `fileops/` | taşıma (`batch_move`: önce Geçmiş'e yaz, sonra taşı), geri alma geçmişi, karantina, yol koruması |
| `engine/` | Kural motoru: kural modeli, plan, uygulama |
| `winshell/` | Windows kabuğu: küçük resim, çekim tarihi, asset kapsamı |

Kaynak dosyaları 250 satırı, satırlar 300 karakteri geçmez.

Sayfalama şu anda istemci tarafındadır; büyük envanterlerde IPC veri aktarımını küçültmez. Liste satırları ayrıca sanallaştırılır. Resimlerde yerleşik ön izleme, diğer türlerde metadata ve varsayılan uygulamada açma bulunur.

Karantinadaki "Kutuya gönder" ve "Tümünü temizle" kalıcı silmez; dosyaları onayla Windows Geri Dönüşüm Kutusu'na gönderir. Okunamayan (kilitli, erişim izni olmayan) dosyalar taramayı durdurmaz; atlanır ve tarama özetinde bildirilir. Sayaçla ayrılan adlar (`IMG_0001`/`IMG_0002`) isim benzeri sayılmaz. Çalıştırılabilir dosyalar (exe, bat, lnk …) uygulama içinden açılmaz; "Klasörde göster" kullanılır. İşlem Geçmişi son 200 işlemi tutar ve atomik yazılır. CSV raporlarında `=`, `+`, `-`, `@` ile başlayan hücreler metne çevrilir. Düzenleme önizlemesinin iptali taramanın iptalinden bağımsızdır.

Hash önbelleği tarama sonunda uygulama veri dizinine (`hash_cache.json`) yazılır; uygulama yeniden açıldığında değişmemiş dosyalar tekrar hash'lenmez. Yalnızca çevrimiçi duran bulut dosyaları (OneDrive) kopya taramasında ve A↔B karşılaştırmasında indirilmemek için atlanır ve tarama özetinde bildirilir. Geçmiş ekranındaki "CSV dışa aktar" tüm taşımaları dosyaya yazar. Karantinadaki "30 günden eskileri seç" yalnızca seçim yapar; gönderme yine onaylıdır. Bozuk `settings.json` dosyasının kopyası `settings.json.bozuk` olarak saklanır. Arayüz bir içerik güvenlik politikasıyla (CSP) çalışır: yalnızca paketle gelen betik, stil ve yazı tipleri yüklenir.

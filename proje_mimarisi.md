# Smart File Organizer — Proje Mimarisi

**Durum:** Bölüm 3–9 hedef mimariyi tanımlar. Bölüm 2 ve 10 **tarihî kayıttır** (yeniden yapılandırma öncesi durum ve o sırada açık olan sorular). Güncel uygulama ve alınan kararlar bölüm 11 ve 12'dedir. Mockup örnek verili referanstır. Bağlayıcı iş kuralları `AGENTS.md` içindedir.

## 1. Teknoloji ve sınırlar

- Tauri v2 + Rust; React 19 + TypeScript + Vite; Tailwind CSS v4 ve Lucide.
- Yalnız `npm`; liste performansı için `@tanstack/react-virtual`.
- Kaynak modülleri tek sorumluluklu ve en fazla 250 satır. Kullanıcı dosyaları otomatik silinmez veya üzerine yazılmaz.
- UI Rust'a Tauri komutlarıyla erişir; dosya sistemi işlemleri ve iş kurallarının nihai doğrulaması backend'dedir.

## 2. Yeniden yapılandırma öncesi durum (tarihî kayıt)

| Alan | Mevcut modüller ve davranış |
| --- | --- |
| UI | `src/App.tsx`, `components/`, `hooks/`; mevcut uygulama mockup'ın modüler sekmeleriyle aynı değildir. |
| Tarama | `scanner.rs`: kategori, yasaklı uzantı, klasör dışlama, alt klasör ve isteğe bağlı `Other` filtresi. |
| Gruplama | `grouper.rs`, `hasher.rs`, `similarity.rs`, `hash_cache.rs`; Exact ve Similar grupları. |
| Karşılaştırma | `compare.rs` iki taramanın dosyalarını birleştirip aynı gruplama hattına gönderir; sonuç yalnız iki kök arasına özgü gruplarla sınırlı değildir. |
| Analiz ve liste | `folder_stats.rs` klasör özeti; `file_list.rs` / `scanner.rs` hash'siz liste. Liste tüm sonucu döndürür. |
| İşlem ve geçmiş | `mover.rs`, `naming.rs`, `history.rs`: grup bazlı taşıma, çakışma çözümü ve geri alma. |
| Veri | `models.rs`, `src/types/file.ts`, `src/lib/commands.ts`; `FileItem.mtime` değişiklik tarihidir, oluşturulma tarihi değildir. |

## 3. Hedef modüller

```text
React uygulama kabuğu: sabit header / modül sekmeleri / kaydırılan içerik / sabit footer
  ├─ Klasör Analizi       → klasör istatistikleri ve grafikler
  ├─ İki Klasör Karşılaştırma → A/B özeti + açılabilir eşleşme ve işlemler
  ├─ Klasördeki Dosyalar  → filtreli ve sayfalı liste
  └─ Dosya Grupları       → gruplar, seçme, grup adı önerisi ve düzenleme
                │
        TypeScript komut katmanı
                │ Tauri IPC: komutlar / ilerleme / iptal
                ▼
   Rust: tarama → hash → gruplama / karşılaştırma → güvenli işlem → geçmiş
```

Önerilen UI sorumlulukları: `AppShell`, modül navigasyonu, `FolderAnalysis`, `CompareView`, `MatchDetails`, `FileListView`, `GroupsView`, ortak `Pagination` ve işlem onayı. Bunlar hedef isimlerdir; uygun yerlerde mevcut bileşenler bölünerek kullanılır. İş mantığı hook/servis katmanında, görsel parçalar bileşenlerde tutulur.

## 4. Temel veri akışları

1. Kök seçilir; A ve isteğe bağlı B ile kapsam/filtre ayarları tutulur.
2. Analiz bağımsız olarak bir veya iki kök için metadata toplar; her kökün yüklenme ve hata durumu ayrıdır. İki sonuç alt alta gösterilir.
3. Karşılaştırma A ve B için kaynak bilgisiyle dosyaları toplar, gruplar ve sonuç döndürür. Eşleşme detayları karşılaştırmanın alt panelidir.
4. Filtre → sıralama → sayfalama uygulanır; sayfa değişiminde dosya kimliğine dayalı seçim korunur. Grup ve dosya listelerinde sanallaştırma gerektiğinde sayfalama ile birlikte kullanılır.
5. İşlem istenince backend kaynakları yeniden doğrular, işlem planı ve çakışmaları oluşturur; kullanıcı onayından sonra uygular; sonuç ve geri alma kaydı gösterilir.

## 5. İş kuralları ve doğruluk

- Varsayılan kategoriler: Doküman, Resim ve Arşiv; isteğe bağlı `Other` açıkça etkinleştirilir. Yasaklı uzantılar tarama kapsamının önüne geçer. `_Kopyalar_Karantina` tarama dışında kalır.
- 500 MiB altı: ilk 4 KiB ön hash, eşleşmede tam BLAKE3. 500 MiB ve üzeri: ilk, orta ve son 1'er MiB ile boyut üzerinden örnek hash. **Örnek hash tam içerik eşitliğini kanıtlamaz**; arayüzde doğrulama durumu belirtilmeli ve riskli taşıma öncesi tam doğrulama seçeneği korunmalıdır.
- Exact: boyut ve ilgili hash aynı; Similar: aynı kategori, uzantısız adlarda Jaro-Winkler ≥ 0,82 (kullanıcı eşiği varsa uygulanır). İsim benzerliği içerik eşitliği değildir.
- Exact grupta seçili saklama politikasındaki ana dosya A kökündeki etiket klasörüne, diğerleri A kökündeki karantina/etiket klasörüne; Similar grubun tüm dosyaları etiket klasörüne taşınır. Hedef çakışmasında numaralandırılır; üzerine yazılmaz. En son işlem geri alınabilir.
- Birebir sonuçların otomatik seçimi mevcut karardır. Ayrı dosya bazlı seçim/taşıma bu grup politikasını belirsizleştirebilir: uygulamadan önce kapsam ve korunacak dosya açıkça belirlenmeli, mevcut politika sessizce aşılmamalıdır.

## 6. Hedef veri sözleşmeleri

Mevcut modeller genişletilir; Rust serde camelCase ve TypeScript tipleri birlikte güncellenir.

- **Dosya:** kararlı işlem kimliği veya normalize edilmiş yol, ad, kök kaynağı A/B, boyut, kategori, oluşturulma zamanı `createdAt: number | null`, değişiklik zamanı `mtime`, hash/kanıt türü. Oluşturulma zamanı desteklenmiyor veya okunamıyorsa `null`; `mtime` ile değiştirilmez.
- **Eşleşme grubu:** tarama oturumu içinde kararlı grup kimliği/ekran numarası, tür (`Exact` / `Similar`), üyeler ve kökenleri, varsa benzerlik/kanıt bilgisi. Grup numarası çift kimliği değildir; grup ikiden fazla dosya içerebilir.
- **Analiz:** kök, kategori bayt/sayıları, toplamlar, alt klasörler ve hata/erişim uyarıları. İki kök için iki ayrı sonuç.
- **İşlem planı:** kaynak, önerilen hedef, eylem, çakışma veya eksik tarih uyarısı; kullanıcıya onaydan önce gösterilir.
- **Sonuç/geçmiş:** başarılı, atlanan ve hatalı öğeler ile geri alma için eski/yeni yollar. Oluşturulma tarihi, dosya kimliği ve grup kimliğinin güncelliği işlem öncesi yeniden kontrol edilir.

## 7. Karşılaştırma ve dosya işlemleri — yapılacaklar

- A/B köklerini canonical path ve yol bileşenleriyle doğrula; aynı veya iç içe klasörlerin çift taranmasını önle ya da açıkça ele al. `startsWith` ile kaynak belirleme; Windows büyük/küçük harf ve ayraç durumlarını güvenli işle.
- Gerçek A↔B eşleşmelerini yalnız tek kök içindeki gruplardan ayır; eşleşmeyen dosyalar için tüm tarama envanterini koru. Mevcut `ScanResult` yalnız grupları içerdiğinden bu ayrıntıları tek başına sağlayamaz.
- Birebir/benzer düğmeleri dosya bazlı seçim yapar; kapsam ve diğer seçimlerin korunması davranışı görünür olmalıdır. Sayfalar arası seçim kimlik bazlıdır.
- Oluşturulma tarihi `metadata.created()` ile okunur; yoksa eksik gösterilir ve tarihli adlandırma ön izlemesinde dosya atlanır veya kullanıcıya sunulur. Önerilen biçim `stem_DD-MM-YYYY.ext`; uzantısız dosyalar ve çakışmalar ele alınır.
- İsimlendirme için ayrı backend plan/uygulama komutları ve güvenli geri alma gerekir; mevcut taşıma geçmişinin yeniden adlandırma desteği test edilmeden varsayılmaz.
- Taşıma/yeniden adlandırmada atomiklik, kısmi hata, dosyanın işlem öncesi değişmesi, yetki hatası ve diskler arası taşıma açıkça ele alınır. Dosyaları silme veya üzerine yazma yoktur.

## 8. Performans ve durum yönetimi

- Dosya ve grup listelerinde varsayılan 25, seçenekler 25/50/100. Önce filtrele/sırala, ardından sayfala; sayfa aralığını veri değişince sınırla. Çok büyük grupların içindeki dosyalar da DOM'u şişirmemelidir.
- İlk aşamada istemci tarafı sayfalama DOM maliyetini azaltır; ancak mevcut IPC tüm dosyaları döndürdüğünden bellek/serileştirme maliyetini azaltmaz. Büyük envanterlerde backend tarafında filtre/sıralama ve sayfalı veya akışlı API ayrıca değerlendirilir.
- Kök/ayar değişiminde eski IPC yanıtlarının yeni görünümü ezmesi engellenir; yükleme, iptal ve hata ayrı yönetilir. İşlem/geri alma sonrası etkilenen analiz ve listeler yenilenir.
- Uzun tarama UI thread'ini bloke etmez; ilerleme ve iptal korunur. Hash önbelleği dosya değişikliğinde geçersiz kılınır.

## 9. Test ve teslim aşamaları

1. **Kabuk ve tasarım sistemi:** tokenlar, responsive sekmeler, sabit alanlar, erişilebilirlik ve koyu tema.
2. **Analiz:** tek/çift kök istatistikleri, grafik verisi ve hata/boş durumlar.
3. **Karşılaştırma modeli:** köken, örtüşen kökler, A↔B grupları, eşleşmeyenler ve kararlı numaralar.
4. **Detay ve işlemler:** seçim, oluşturulma tarihi, isim ön izlemesi, güvenli onay, taşıma ve undo; başarısızlık senaryoları.
5. **Listeleme ve gruplama:** sayfalama, seçim kapsamı, otomatik/düzenlenebilir grup adları ve sanallaştırma.
6. **Doğrulama:** Rust birim/entegrasyon testleri, TypeScript derlemesi, erişilebilirlik ve farklı pencere boyutlarında Tauri manuel testi; `npm run build`, `cargo test` ve 250 satır kontrolü.

## 10. Uygulama öncesi karar gerektiren noktalar (tarihî; hepsi karara bağlandı)

Sonuçlar bölüm 11'dedir: benzer ad seçimi tam grubu taşır; tarihi okunamayan dosya atlanır, tarih yerel saatle `DD-MM-YYYY` yazılır; her grubun kendi adı vardır, ortak etiket bunların önüne geçer; büyük dosyalarda tam doğrulama isteğe bağlıdır.

- İsim benzerliği seçimi tüm grup üyelerini mi, yalnız A/B arasında eşleşen üyeleri mi seçecek? Mevcut otomatik Exact grup seçimi ile dosya bazlı seçim nasıl bağlanacak?
- Tarihi eksik dosyalar varsayılan olarak atlanacak mı, kullanıcıdan alternatif tarih mi istenecek? Tarih biçimi ve saat dilimi kesinleştirilmeli.
- Birden fazla seçili grubun otomatik adı her grup için ayrı mı oluşturulacak, tek ortak ad mı kullanılacak? Düzenlenen ad taşıma etiketiyle aynı mı olacak?
- Örnek hash ile eşleşen büyük dosyalarda otomatik seçim ve taşıma için zorunlu tam doğrulama isteniyor mu?

## 11. Uygulama teslim notları

- Yeni kabuk `WorkspaceHeader`, `WorkspaceSettings`, `WorkspaceFooter`; ekranlar `FolderAnalysis`, `CompareView`, `FileBrowser`, `GroupsView` olarak ayrıldı. Geçmiş ayrıca erişilebilir. İçerik en fazla 1366 px; ekran gövdesi kaydırılırken header/footer görünür kalır.
- Klasör analizinin kategori toplamları kök dosyalarını da kapsar; okuma hataları uyarılır. Aynı kök/revizyon için analiz istekleri paylaşılır. A ve B analizleri alt alta, karşılaştırma özetleri yan yana sunulur.
- `ScanResult.comparison` tüm karşılaştırma envanterini kaynak ve grup kimliğiyle taşır. Yalnız A ve B arasında üyeleri bulunan gruplar korunur; aynı/iç içe kökler reddedilir.
- `FolderFile.createdAt` gerçek oluşturulma zamanıdır. Tarihli isimlendirme `rename.rs` içinde backend'de saklanan tek kullanımlık plan, kaynak güncelliği kontrolü, çakışma kontrolü ve geçmiş kaydıyla uygulanır. Tarihsizler atlanır; yerel saatle `DD-MM-YYYY` kullanılır (`file_ops::local_date_suffix`; şablonla adlandırma ve yıl klasörü de aynı işlevi kullanır).
- Her grubun ayrı önerilen/düzenlenebilir adı vardır; ortak etiket girildiğinde bu adların önüne geçer. `group_moves.rs` kaynak/kök doğrulaması yapar ve taşıma öncesi geçmişi kaydeder. Dosya bazlı seçim tam grup taşıma politikasını değiştirmez.
- `file_ops.rs` hedefi üzerine yazmadan aktarır; aynı birimde hard link, desteklenmiyorsa yeni hedef dosyası oluşturma ve kopyalama kullanır. Kaynak yalnız başarılı aktarım sonrası kaldırılır. Cross-volume kopyalamada özel dosya metadata/ACL'lerinin tümünü koruma garantisi verilmez.
- Yeni ekranlar sayfalama ve sanallaştırmayı birlikte kullanır. Sayfalama istemcidedir; backend sayfalaması sonraki ölçümlere bağlıdır. Büyük dosyalarda örnek hash rozeti ve isteğe bağlı tam doğrulama korunur.
- Arayüz tercihleri (tema, son klasörler, yasaklı uzantılar, düzenleme profilleri) `settings.json` içindeki `uiPrefs` alanındadır; `src/lib/prefs.ts` açılışta bir kez yükler, `set_ui_pref` tek anahtarı yazar. `localStorage` yalnızca temanın ilk boyama önbelleğidir.
- Tarama/karşılaştırma `cancel_scan`, düzenleme önizlemesi `cancel_organize` bayrağını kullanır; biri diğerini sıfırlamaz. Klasör analizi ve sağlık raporu iptal edilemez.
- Ekranlar ilk açıldıklarında bağlanır, sonra gizlenerek durumlarını korur (`App.tsx`).
- Rust dosya sistemi testleri ve tarayıcıda temsili IPC verileriyle yapılan UI kontrolleri, gerçek Tauri penceresinde uçtan uca manuel dosya işlemi testiyle aynı şey değildir.

## 12. Akıllı otomasyon (gelen kutusu)

Kaynak: `docs/arsiv/fikir_2026-11-12.md` (arşiv), kararlar ve ekranlar `docs/akilli-otomasyon-mockup.html`. Bağlayıcı kurallar `AGENTS.md` ve `gelen-kutusu` skill'indedir.

| Katman | Modüller | Sorumluluk |
| --- | --- | --- |
| Ayarlar | `settings.rs`, `settings_cmds.rs` | `AppSettings` (kaynaklar, yedek, 5 grup, otomasyon, Diğer, açılışta bekleyenleri tara); doğrulama; `settings.json` geçici dosya + yeniden adlandırma ile kayıt. Kayıt sonrası watcher yeniden başlar. |
| İzleme | `watcher_core.rs`, `watcher.rs`, `inbox_path.rs` | `notify` ile alt klasörsüz izleme; `Pending` 2 sn sabitlik bekler; `move_to_inbox` (`_1` numarası); geçmişe önce yazma, dakikalık birleştirme; `inbox-updated` olayı. Gün yerel tarih. |
| Arka plan | `automation.rs`, `tray.rs`, `single_instance.rs` | Duraklatma bayrağı ve durum etiketi (`automation-status`), tepsi menüsü, kapatınca gizleme, tek kopya. |
| Listeleme | `inbox_list.rs`, `inspector.rs`, `thumbnail.rs` | Gün klasörünü gruplara göre listeler (en fazla 5.000 dosya); geliş saati geçmişten; 300 karakterlik UTF-8 güvenli metin özeti; Jaro-Winkler ≥ 0,82 benzer ad; 2 MB üstü görsellere önbellekli 96 px küçük resim. |
| İşlemler | `operations.rs`, `inbox_rename.rs`, `trash_ops.rs` | Yalnız gelen kutusu kaynaklı taşıma (geçmişli, boşalan klasör temizliği), `{grup}`/`{özelmetin}` alanlı şablonla adlandırma (mevcut plan/uygula/geri al hattı), Geri Dönüşüm Kutusu (geçmişsiz). |
| UI | `SettingsView`, `ExtGroupCard`, `InboxStatusPanel`, `EveningPanel`, `InboxGroupList`, `InboxFileRow`, `InboxActionBar`, `InboxRenameDialog`, `InboxTrashDialog`; `useSettings`, `useInbox` | shadcn tarzı Radix bileşenleri (`components/ui/`: accordion, popover, dialog), `cn()` yardımcısı ve mevcut renk token'ları. Grup satırları sanallaştırılır. |

Testler: `settings_tests`, `watcher_core_tests`, `inspector_tests`, `operations_tests`, `naming` ve `inbox_path` iç testleri; yerel tarih (00:30), `_1`/`_2` ve büyük/küçük harf çakışması, uzantı eşleşmesi, çok baytlı Türkçe metnin kesilmemesi, gelen kutusu dışı taşıma/silmenin reddi, yazımı süren dosyanın bekletilmesi ve watcher'ın kendi taşıdığını yeniden yakalamaması (yedek klasörü kaynağın içinde seçilemez).

Sınırlar: Windows'ta son erişim zamanı çoğu zaman güncellenmez (UI'da "≈"). Geri Dönüşüm Kutusu'na gönderilen dosyalar uygulamadan geri alınamaz. Watcher yalnız uygulama çalışırken (pencere veya tepsi) etkindir; kapalıyken gelenler "açılışta bekleyenleri tara" ile alınır.

# Seç ve Grupla — Tasarım

**Tarih:** 2026-11-15
**Durum:** Onay bekliyor
**Simülasyon:** [docs/sec-ve-grupla-mockup.html](../../sec-ve-grupla-mockup.html) (yayımlanmış hâli: https://claude.ai/artifact/5hjLR53cdaJ96SC2ghERcF)

## 1. Amaç ve gerekçe

Kullanıcı bir klasördeki dosyalara **bakarak** (ön izleme) ya da **adlarına göre** seçim yapmak, seçtiklerini bir grup adıyla bir klasöre taşımak ve istersen taşımadan önce şablonla adlandırmak istiyor. Örnekler:

- 30 fotoğraftan Haziran'da çekilmiş 10 tanesi → `Tatil fotoğraflarım\`.
- 5 faturadan Haziran'a ait 3 tanesi → `Faturalar\Haziran\`; sonra Mayıs'a ait olanlar → `Faturalar\Mayıs\`.

Bugün uygulamada **Harita > Klasördeki dosyalar** ekranı dosyaları yalnızca listeliyor: her satırda aynı ikon var, seçim ve taşıma yok. Taşıma ancak kopya grupları (Exact/Similar) ya da türe göre düzenleme üzerinden yapılabiliyor; elle seçip gruplamanın yolu yok.

## 2. Hedefler

1. Her satırda dosyanın **gerçek ön izlemesi**: fotoğraf, video karesi, PDF/Word/Excel/PowerPoint sayfası. Ön izlemesi olmayan dosyada **uzantının Windows ikonu**.
2. **Büyük ön izleme**: küçük resme veya ada tıklayınca açılır. `←` `→` ile dosyalar arasında gezilir, `Boşluk` ile seçilir.
3. **Seçim**: satır başında onay kutusu, satırın boş yerine tıklama, `Shift` ile aralık seçimi ve süzülen dosyaların tümünü seçen başlık kutusu.
4. **Süzme ve sıralama**: ada göre arama, **yıl** ve **ay**, tür (Resim, Video, PDF, Doküman, Diğer); tarihe, ada veya boyuta göre sıralama.
5. **Çekim tarihi öncelikli tarih**: fotoğraf ve videolarda kameranın yazdığı çekim tarihi, diğer dosyalarda değiştirme tarihi. Satırda hangisinin kullanıldığı simgeyle belli olur.
6. **Hedef klasör + grup adı**: Hedef seçiliyse grup adı onun alt klasörü olur. Seçili değilse grup adı açık klasörün içinde oluşturulur. Grup adında `\` alt klasör demektir (`Faturalar\Mayıs`).
7. **Adlandır**: Bugün ekranındaki "Taşınacakları adlandır" ile aynı şablon motoru. Seçilenler yerinde adlandırılır ya da "Adlandır, sonra taşı" ile iki iş art arda yapılır.
8. **Güvenlik**: her taşıma ve adlandırma onaydan sonra yapılır, Geçmiş'e yazılır ve "Son işlemi geri al" ile döner. Üzerine yazma yoktur, çakışan ad numaralanır.

## 3. Kapsam dışı

- PDF ya da belge **içindeki** tarihi okumak (faturalar değiştirme tarihine göre süzülür).
- Alt klasörlerin dosyalarını aynı listede göstermek. Liste bugünkü gibi açık klasörün **doğrudan** dosyalarını gösterir; alt klasörlere klasör kartlarından girilir.
- Kopyalama (yalnızca taşıma), sürükle-bırak, hedef/şablonları profil olarak kaydetmek.
- HEIC gibi kodek gerektiren biçimler için kodek kurmak: Windows ön izleme veremiyorsa ikon görünür.

## 4. Olmaması gerekenler

- Onaysız taşıma veya adlandırma; kalıcı silme; hedefteki dosyanın üzerine yazma.
- Büyük klasörde listenin geç açılması: ön izlemeler ve çekim tarihleri listeyi **bekletmez**.
- Süzgeç dışında kalan seçili dosyaların **habersizce** taşınması: seçim çubuğu bunu açıkça yazar.

## 5. Kısıtlar

- Tauri v2 + React 19 + TypeScript; yalnızca `npm`.
- Dosya başına en çok 250 satır ve 300 karakter, tek sorumluluk (`npm run audit:source`).
- Renkler `src/styles/tokens.css`, ikonlar `lucide-react`; ayarlar yalnızca `settings.json`.
- `assetProtocol.scope` zaten `**` olduğundan önbellekteki PNG'ler `convertFileSrc` ile gösterilebilir.
- Projede ön yüz test altyapısı yok. Rust tarafı `cargo test --no-default-features` ile test edilir.

## 6. Değerlendirilen yaklaşımlar

| Karar | Seçenekler | Seçilen ve neden |
|---|---|---|
| Ekran | **A)** Mevcut *Klasördeki dosyalar* ekranını genişletmek · B) Ayrı "Elle düzenle" alt sekmesi | **A.** Klasörde gezinme ve A/B seçimi zaten orada; B neredeyse aynı iki ekran üretirdi. |
| Ön izleme | **A)** Windows küçük resim servisi (`IShellItemImageFactory`) · B) Yalnızca tarayıcı motoru (`<img>`, `<video>`) | **A.** PDF, Office, MKV/AVI ön izlemesi ve gerçek uzantı ikonu tek kaynaktan gelir. `windows` paketi Tauri sayesinde zaten derleniyor (0.62). B'de PDF ve Office ön izlemesi yok. |
| Tarih | **A)** Çekim tarihi öncelikli · B) Yalnızca değiştirme tarihi | **A.** Kopyalanmış fotoğraflarda değiştirme tarihi yanıltıcı. Çekim tarihi aynı Windows servisinden okunur; ek kütüphane gerekmez. |
| Hedef | **A)** Hedef klasör seçici + grup adı (alt klasör, `\` desteği) · B) Yalnızca grup adına yol yazmak · C) Yalnızca seçici | **A.** Kullanıcının istediği akış: hedef bir kez seçilir, sonraki gruplar için yalnızca alt klasör adı yazılır. `\` yazmak hızlı yol olarak kalır. |

## 7. Kullanıcı tarafında kararlaştırılanlar

- **Akış:** Klasör açılır → süz/sırala → ön izlemeye bakarak seç → (isteğe bağlı) adlandır → hedef + grup adı → onay → taşı.
- **Tarih:** Liste önce değiştirme tarihiyle açılır, çekim tarihleri arka planda okunur. Okuma sürerken yıl/ay süzgeci kapalıdır ve "Çekim tarihleri okunuyor… 23/41" yazar.
- **Hedef:** Hedef seçilmemişse grup adı açık klasörün içinde klasör olur. Seçilmişse seçilen klasör ana klasör, grup adı alt klasör olur. Seçilen hedef taşımadan sonra **seçili kalır**.
- **Var olan klasör:** Yeni klasör açılmaz, dosyalar içine eklenir. Çakışan dosya adı numaralanır.
- **Adlandırma:** Bugün'deki pencereyle aynı alanlar: şablon, etiketler, özel metin, başlangıç, basamak, eski → yeni ön izlemesi.

## 8. Uygulamanın tasarımı

### 8.1 Arka uç (Rust, `src-tauri/src/`)

Yeni modüller tek sorumluluklu ve 250 satırın altında olacak.

| Modül | Sorumluluk |
|---|---|
| `shell_thumb.rs` | `IShellItemImageFactory::GetImage` ile küçük resim üretmek (HBITMAP → `GetDIBits` → `image` → PNG). Önbellek `app_cache_dir()/thumbs/shell`, anahtar yol + boyut + mtime + piksel boyutu (`thumbnail.rs` ile aynı mantık). Ön izleme yoksa Windows otomatik olarak ikonu döndürür; `SIIGBF_THUMBNAILONLY` kullanılmaz. COM her iş parçacığında `CoInitializeEx(COINIT_APARTMENTTHREADED)` ile başlatılır. |
| `media_dates.rs` | `IShellItem2::GetFileTime` ile `PKEY_Photo_DateTaken`, yoksa `PKEY_Media_DateEncoded`. UTC FILETIME → unix saniye. Ön yüz yerel tarihe çevirir. Değer yoksa `None`. |
| `selection_move.rs` | Ön izle → uygula akışı. `preview_selection_move(roots, files, targetDir)` dosyaları doğrular: kök A veya B içinde olmalı, karantinada olmamalı, boyut/mtime damgası tutmalı, aynı dosya iki kez seçilmemiş olmalı. Hedef yolu karantina içinde olamaz. Her dosya için `naming::unique_dest` (Paren: `ad (1).jpg`) ile hedefi ve oluşturulacak klasörleri hesaplar, planı `RenameState` benzeri bir bekletme alanında tutar. `apply_selection_move(planId)` klasörleri oluşturur ve `group_moves.rs` gibi **her taşımadan önce** Geçmiş'e yazıp `file_ops::transfer` çağırır. Geçmiş etiketi: "Seçilenleri taşı". |
| `inbox_rename.rs` (genelleştirme) | `plan_list_rename` bugün yalnızca gelen kutusu içini kabul ediyor ve tarihi mtime'dan alıyor. Kök sınırı ve dosya başına tarih (çekim tarihi öncelikli) parametre olur. Yeni komut `preview_selection_rename(roots, paths, dates, options)`, mevcut `apply_rename_plan` ile uygulanır. Geçmiş etiketi: "Seçilenleri adlandır". `{Grup}` alanına taşınacak son klasörün adı (`Faturalar\Mayıs` için "Mayıs") ön yüzden gelir. |
| `list_subfolders` komutu | Hedef seçici için kök(ler)in altındaki klasör ağacı. Karantina ve hariç klasörler atlanır. Derinlik ve adet sınırlıdır (öneri: derinlik 6, en çok 2.000). |
| `models.rs` | `FolderFile`'a `modifiedAt: Option<u64>` eklenir. Değiştirme tarihi ve sıralama bunu kullanır. |

**Geri alma:** Mevcut `undo_move` + `prune_after_undo` taşınan dosyaları geri getirir ve boşalan hedef klasörleri kaldırır. Bilinen davranış: önceden var olan ama geri almadan sonra boş kalan bir hedef klasör de kaldırılır. İçinde dosya olmadığı için veri kaybı yoktur.

**Bağımlılık:** `windows = { version = "0.62", features = ["Win32_UI_Shell", "Win32_UI_Shell_PropertiesSystem", "Win32_System_Com", "Win32_Graphics_Gdi", "Win32_Foundation"] }`. Lock dosyasındaki sürümle aynıdır.

### 8.2 Ön yüz (React, `src/`)

`FileBrowser.tsx` ince bir kabuk olarak kalır; parçalar ayrılır:

| Dosya | Sorumluluk |
|---|---|
| `hooks/useFileSelection.ts` | Seçili yollar, son tıklanan, `Shift` aralığı, süzgeç dışında kalan sayısı. Klasör değişince seçim temizlenir. |
| `hooks/useShellThumbs.ts` | Görünen satırların küçük resimlerini 30'luk paketlerle ister; önbellek `Map`. |
| `hooks/useMediaDates.ts` | Açık klasördeki dosyaların çekim tarihlerini arka planda 50'lik paketlerle okur, ilerlemeyi verir. Klasör değişince iptal eder. |
| `lib/fileKinds.ts` | Uzantı → tür (Resim, Video, PDF, Doküman, Diğer); etkin tarih (çekim ?? değiştirme). |
| `components/FileBrowserFilters.tsx` | Arama, yıl, ay, sıra, tür çipleri. |
| `components/FileBrowserRow.tsx` | Onay kutusu, küçük resim, ad, tarih (kamera/kalem simgesi), boyut, klasörde göster. |
| `components/SelectionBar.tsx` | Seçim özeti, mini küçük resimler, ay önerisi, hedef seçici, grup adı ve yol ön izlemesi ("var"/"yeni" rozetleri), Adlandır…, Klasöre taşı. |
| `components/TargetFolderSelect.tsx` | "Bu klasör", kök içindeki klasörler, "Diğer konumlar", "Başka bir klasör seç…" (`@tauri-apps/plugin-dialog`). |
| `components/SelectionMoveDialog.tsx` | Onay: hedef yol, oluşturulacak klasörler, numaralanacak dosyalar. |
| `components/TemplateRenameDialog.tsx` | `InboxRenameDialog` içeriğinden ortak bileşen çıkarılır (önizleme/uygulama fonksiyonları prop olur); hem Bugün hem bu ekran kullanır. "Adlandır, sonra taşı" düğmesi yalnızca bu ekranda bulunur. |
| `components/FilePreview.tsx` | Önceki/sonraki, seç/seçimden çıkar ve kısayollar eklenir. Resim olmayan dosyalarda büyük (768 px) Windows ön izlemesi gösterilir. |

**Grup adı doğrulaması:** `\` veya `/` ile parçalara ayrılır. Her parçada `: * ? " < > |` yasak; Windows'un ayrılmış adları (CON, NUL…) yasak; nokta ile bitemez; boş parça olamaz. Hedef açık klasörün kendisiyse taşıma kapalıdır.

**Kapsam "Tümü · A ve B":** Hedef seçicide "Bu klasör" yerine "Klasör A kökü" ve "Klasör B kökü" çıkar. Hedef seçilmeden taşıma düğmesi açılmaz.

**Kilit:** Tarama veya başka bir taşıma sürerken Adlandır ve Klasöre taşı kapalıdır (`app.isScanning`, `app.isMoving`).

**Taşımadan sonra:** Seçim ve grup adı temizlenir, hedef seçili kalır, liste ve klasör kartları yenilenir (`statsToken`). Bildirimde "Klasörü aç" (kök içindeyse) ve "Geri al" bulunur.

## 9. Benim verdiğim kararlar (simülasyonda görüldü, değiştirilebilir)

1. Büyük ön izlemede `←` `→` / `Boşluk` ile gezip seçme.
2. Satıra tıklayınca seçme ve `Shift` ile aralık seçme.
3. Seçilenlerin hepsi aynı aydansa ad önerisi: hedef seçiliyse "Mayıs", değilse "Mayıs 2024".
4. Süzgeç dışındaki seçili dosyalar da taşınır ve bu açıkça yazılır.
5. Klasör değiştirince seçim temizlenir.
6. Adlandırmada tarih alanları çekim tarihini kullanır (Bugün ekranında değiştirme tarihi).
7. Adlandırmada çift `__` ve baştaki/sondaki ayraçlar temizlenir.
8. Çakışma numarası: adlandırmada `ad_1` (Bugün gibi), taşımada `ad (1)` (taşıma kuralı gibi).
9. Hedef seçici yalnızca bu oturumda hatırlanır; uygulama yeniden açılınca "Bu klasör"e döner.
10. Rename + move iki ayrı Geçmiş kaydıdır; geri alma iki adımda döner.

## 10. Uygulayıcıya bırakılanlar

- COM iş parçacığı düzeni (küçük bir sabit havuz ya da `spawn_blocking`).
- Küçük resim paket boyutları ve görünür satır tespiti (`VirtualRows` görünen öğeler).
- Klasör ağacı sınırlarının kesin değerleri ve sınır aşıldığında gösterilecek uyarı.
- Test kapsamı: `selection_move` (hedef birleştirme, iç içe klasör, `unique_dest`, kök dışı ve karantina reddi, değişmiş dosya reddi), genelleştirilmiş adlandırma planı (kök sınırı, verilen tarihin kullanılması), grup adı doğrulaması (Rust tarafında da yapılmalı). Windows servis çağrıları elle doğrulanır.
- Doğrulama: `npm run check` ve uygulamayı çalıştırıp senaryoların elle denenmesi.

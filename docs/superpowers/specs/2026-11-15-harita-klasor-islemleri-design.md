# Kopyalar ve Harita alt menüleri, Dosya-Klasör işlemleri

Tarih: 2026-11-15
Durum: Taslak. Onay bekliyor. Onaydan önce ürün kodu yazılmayacak.
Simülasyon: `docs/simulasyon/klasor-dosya-islemleri.html` (güncel). Eski deneme: `docs/simulasyon/harita-klasor-islemleri.html`.

> **Güncelleme (2026-11-15, ikinci tur).** Kullanıcı yapıyı netleştirdi; aşağıdaki 4.1, 4.2, 8 ve 9. bölümler buna göre yeniden yazıldı:
> tek bir üst menü (**Klasör-Dosya İşlemleri**), altında **Kopyalar**, **Harita** ve **Adlandırma**; klasör seçimi kayan panelde değil, bu sayfaların başında.

## 1. Niyet

Kullanıcı üç şey istiyor:

1. **Kopyalar** ve **Harita** sol menüde alt menülü olsun. Harita'nın altında **Dosya-Klasör işlemleri** olsun. Buna tıklanınca bir panel açılsın.
2. Üst bardaki **klasör seçimi** ve ona bağlı bileşenler (Klasör A/B, son klasörler, karşılaştırma anahtarı, tara düğmesi) bu panele taşınsın.
3. **Klasör işlemleri** adlı bir alt menü, seçili klasörün içindeki dosya ve klasör adlarını kurallara göre düzeltsin: harf biçimi, ayraç (tire veya alt çizgi), Türkçe karakter dönüşümü, sıra numarası, ay-Yıl tarihi, uzunluk sınırı.

Kopyalar sayfasında, sayfanın altındaki **grup seçimi ve açıklama** çubuğu sabit alt bardan çıkıp sayfanın içine girsin.

## 2. Mevcut durum (kodda ne var)

| Konu | Dosya | Durum |
|---|---|---|
| Hedefler ve alt ekranlar | `src/lib/navigation.ts` | 5 hedef. Kopyalar: groups, compare, quarantine. Harita: analysis, files, health. |
| Sol menü | `src/components/AppNav.tsx` | Düz liste. Alt menü yok. |
| Alt sekmeler | `src/components/SubTabs.tsx` | Hedefin ekranlarını üst içerikte gösteriyor. |
| Klasör seçimi | `src/components/WorkspaceHeader.tsx` | Klasör A, son klasörler, karşılaştırma anahtarı, Klasör B, tara düğmesi üst barda. |
| Grup alt barı | `src/components/WorkspaceFooter.tsx` | Sayfa dışında sabit. Grup özeti, "Otomatik ad", ortak etiket, "Taşımayı incele", saklama politikası. Yalnızca compare ve groups ekranlarında (`MOVE_BAR`). |
| Yeniden adlandırma | `SelectionRename.tsx`, `TemplateRenamePanel.tsx`, `src-tauri/src/inbox_rename.rs` | Şablon (`{ad}`, `{sıra}`, `{tarih}`, `{yıl}`, `{ay}`, `{gün}`, `{klasör}`), önizleme, uygulama ve geri alma var. |
| Temizlik | `inbox_rename.rs` → `tidy()` | Yalnızca çift alt çizgiyi ve baş-son ayraçları temizliyor. Harf biçimi, Türkçe karakter, uzunluk ve klasör adı yok. |

Sonuç: geri alma ve önizleme altyapısını yeniden yazmaya gerek yok. Eksik olan, yeni kural hattı ve klasör adı desteği.

## 3. Yaklaşımlar

**A. Mevcut yeniden adlandırma hattını genişlet (önerilen).**
`tidy()` yerine bir kural hattı koyulur. Önizleme, uygulama ve geri alma mevcut akışta kalır. Kurallar Rust'ta yazılır, böylece önizleme ile uygulama aynı kodu kullanır.
- Artı: tek kod yolu, geri alma hazır, önizleme ile uygulama uyuşmaz.
- Eksi: klasör yeniden adlandırma ve geri alma sırası kontrol edilmeli.

**B. Ayrı bir "Klasör işlemleri" motoru.**
Kendi plan ve geri alma mantığıyla sıfırdan yazılır.
- Artı: diğer akıştan bağımsız.
- Eksi: iki ayrı geri alma sistemi, yüksek risk. Önerilmez.

**C. Yalnızca ön yüzde (TypeScript) adlandırma.**
- Artı: hızlı.
- Eksi: önizleme ile uygulama farklı olabilir, geri alma yok. Önerilmez.

Öneri: **A**. Dosya ve klasör adı kuralları tek bir Rust modülünde toplanır.

## 4. Arayüz

### 4.1 Sol menü
```
Bugün
Klasör-Dosya İşlemleri ▾
   Kopyalar      (sekmeler: Dosya grupları | İki klasör karşılaştırma | Karantina)
   Harita        (sekmeler: Klasör analizi | Klasördeki dosyalar | Klasör sağlığı)
   Adlandırma    (yeni ekran)
Kurallar
Geçmiş
```
- Üst öğe bir sayfa değil, açılıp kapanan bir gruptur. Alt öğeler mevcut `Destination` yapısını korur; her birinin alt sekmeleri (`SubTabs`) aynen kalır.
- Kopya sayısı rozeti "Kopyalar" alt öğesinde durur.
- Dar ekranda (60rem altı) alt öğeler tek satırlık sekme ızgarasına düşer.

### 4.2 Klasör seçimi (sayfa başı) ve Adlandırma ekranı
- **Klasör seçimi** üst bardan çıkar; Kopyalar, Harita ve Adlandırma sayfalarının en üstünde bir panel olur ve **yalnızca bu üç sayfada** görünür:
  seçili yol, Klasör A, son klasörler, "İki klasör karşılaştır" anahtarı, Klasör B, tara düğmesi. "Tarama kapsamı" satırı aynı panelin alt satırıdır.
  Adlandırma sayfasında tara düğmesi ve tarama kapsamı gizlenir (tarama gerektirmez).
- **Adlandırma ekranı** iki sütundur: solda kurallar (hazır kalıplar, kapsam, harf biçimi, boşluk yerine, Türkçe harfler, sıra numarası, tarih, uzunluk),
  sağda canlı önizleme: klasör ağacı, şimdiki ad → yeni ad, uzunluk, durum rozeti (Yeni ad, Yalnız harf, Numaralandı, Aynı kalır).
- Uygula düğmesi: "N adı uygula". Uygulanınca işlem geçmişine tek kayıt girer ve geri alınabilir.

### 4.3 Üst bar
Üst barda yalnızca şunlar kalır: geri al, işlem geçmişi, ••• menüsü, tema ve ayarlar.

### 4.4 Kopyalar sayfası
- Grup seçimi özeti ("N grup seçildi · M dosya") ve altındaki "Saklama politikası ve doğrulama" sayfanın içine, yeşil taşıma alanının hemen üstüne girer.
- Sabit alt bar kaldırılır. "Otomatik ad", ortak etiket alanı ve "Taşımayı incele" düğmesi çıkarılır (kullanıcı kararı).
- Toplu taşıma kaybolmasın diye yeşil alandaki düğme seçime göre değişir: tek grup için "Bu grubu taşı", kutularla 2+ grup seçiliyse "Seçili N grubu taşı"
  (her grup kendi etiket adıyla). Bu, kaldırılan "Taşımayı incele"nin yerini tutar. (Öneri; onay bekliyor.)
- İki klasör karşılaştırma ekranı da aynı alt barı kullanıyordu; orada da seçim özeti ve taşıma düğmesi sayfanın içine alınır.

### 4.5 Kartlar, daralan kenar çubuğu ve duyarlı düzen (üçüncü tur)

**Kopyalar: grup listesi kartı.** Soldaki grup listesi tek bir kartın içindedir (başlık: "Gruplar").
- Satırlar arasında 1px çizgi vardır; satırların kendi çerçevesi ve aralığı kalkar.
- Açık olan grup: solda 5px vurgu çizgisi (`--color-accent`) ve yeşil zemin (`--color-keep-tint`).
- Kutusu işaretli ama açık olmayan grup: solda soluk yeşil çizgi. Üzerine gelinen satır `--color-paper` zeminini alır.

**Kopyalar: karşılaştırma kartı.** Ortadaki karşılaştırma alanı (ad, ana dosya ve kopya kartları, doğrulama notu, seçim özeti, yeşil taşıma alanı)
tek bir kartın içindedir: 14px köşe (`--radius-card`), 1px çizgi (`--color-rule`) ve gölge.

**Gölge.** `tokens.css` içinde `--shadow-card` bugün `none`; rehber "kenar çizgisi var, gölge yok" diyor. Bu istekle `--shadow-card` yumuşak bir gölge olur
(açık tema: `0 1px 2px …/.06, 0 10px 28px …/.09`; koyu temada daha koyu). Yalnızca Kopyalar'daki bu iki kart kullanır; diğer `.panel`'ler gölgesiz kalır.
`tasarım_rehberi.md` içindeki ilgili satır buna göre güncellenir. (Kapsam varsayımdır, soru 4.)

**Daralan kenar çubuğu.** Kenar çubuğu 232px'ten 64px'lik ikon şeridine iner: yalnızca ikonlar görünür, adlar `title` ipucunda kalır.
- 60rem altında kendiliğinden daralır. Bugünkü "üstte sekme ızgarası" davranışının yerini alır.
- Geniş ekranda üst bardaki düğmeyle daraltılıp genişletilir; tercih `uiPrefs.navCollapsed` olarak `settings.json`'a yazılır.
- Şeritte alt öğeler (Kopyalar, Harita, Adlandırma) girintisiz ve hep açıktır; grup başlığı tıklanmaz. Sözcük işareti, rozet ve otomasyon durumu gizlenir.

**Duyarlı düzen (tüm sayfalar).**
- 60rem altı: sayı şeridi 2 sütun; Kopyalar ve Adlandırma'da iki sütun alt alta iner, kartlar tam genişlik olur.
- 44rem altı: Adlandırma önizlemesinde yalnızca yeni ad ve durum kalır; sekmeler yatay kayar; birincil düğmeler tam genişlik; iç boşluklar 24px'ten 16px'e iner.
- Hiçbir genişlikte yatay sayfa kaydırması olmaz (simülasyonda 700px'te üç sayfa da denetlendi).

### 4.6 Sayfa başlığı kartı ve menü çubuğu (dördüncü tur)

Başlık, alt açıklama, durum rozeti, alt sekmeler ve sayı şeridi bugün sayfada ayrı ayrı duruyor. Hepsi tek bir karta girer (çizgi, 14px köşe, `--shadow-card`):

1. **Menü çubuğu (kartın üstü).** Alt sekmeler (`SubTabs`) koyu yeşil (`--color-chrome`) bir çubuğa dönüşür. Her düğmede ikon vardır;
   seçili düğme amber (`--color-action`), diğerleri açık renk yazı, üzerine gelince `--color-chrome-2`.
   - Kopyalar: Dosya grupları (Copy), İki klasör karşılaştırma (Columns2), Karantina (Shield).
   - Harita: Klasör analizi (LayoutGrid), Klasördeki dosyalar (File), Klasör sağlığı (Activity).
   - Tek ekranlı sayfalarda (Adlandırma, Bugün, Geçmiş) çubuk görünmez.
2. **Başlık satırı.** Solda başlık ve alt açıklama, sağ üstte durum rozeti (yeşil hap, onay ikonu). Başlık kart içinde biraz küçülür (`clamp(1.75rem, 3.4vw, 2.75rem)`).
3. **Sayı şeridi (kartın altı).** Dört sütun, kartın alt satırı; ayrı üst-alt çizgili şerit kalkar.

Hiza: menü çubuğundaki ilk düğme, başlık ve ilk sayı aynı sol çizgide (24px) başlar. 44rem altında bu boşluk 16px olur, menü çubuğu yatay kayar, sayı şeridi 2 sütuna iner.

Kod: `SubTabs.tsx` (ikon ve menü çubuğu), `lib/navigation.ts` (ekran başına ikon), yeni `ScreenHeader.tsx` (`App.tsx` içindeki `screen-head`, `SubTabs` ve `stat-strip` bölümü buraya taşınır), `styles/workspace.css`.

## 5. Klasör işlemleri kuralları

### 5.1 Uygulama sırası
Kurallar her ad için bu sırayla uygulanır:

1. **Türkçe karakter** (açık ise): ç→c, ğ→g, ı→i, ö→o, ş→s, ü→u, İ→I, Ç→C, Ğ→G, Ö→O, Ş→S, Ü→U
2. **Ayırıcı normalize:** boşluk, `_` ve `-` tek boşluğa indirilir. Böylece ayraç değişimi tutarlı olur.
3. **Harf biçimi:** baş harfler büyük / hepsi küçük / HEPSİ BÜYÜK / olduğu gibi. Türkçe harf kuralı (`i`→`İ`, `ı`→`I`) için `tr` yerel ayarı kullanılır.
4. **Geçersiz karakterler** (`< > : " / \ | ? *`) kaldırılır.
5. **Ayraç:** kelimeler tire (`-`), alt çizgi (`_`) veya bitişik (boşluk kaldırılır) ile birleştirilir.
6. **Sıra numarası** (açık ise): ön ek. Başlangıç, basamak (01, 001) ve sıralama (tarihe göre veya ada göre) seçilir.
7. **Tarih** (açık ise): son ek. Biçimler: `ay-Yıl` (11-2026), `yıl-ay` (2026-11), `gg-aa-yyyy` (15-11-2026). Kaynak: değiştirme tarihi (varsayılan) veya fotoğraf çekim tarihi.
8. **Uzunluk:** ad gövdesi, uzantı hariç, en fazla N karakter. Gövde kesilir. Sıra numarası ve tarih korunur.
9. **Uzantı** olduğu gibi kalır.
10. **Çakışma:** hedef adı zaten varsa mevcut kuraldaki gibi ` (1)`, ` (2)` eklenir (`rename_template.rs` → `free_target`). Hiçbir dosyanın üzerine yazılmaz.
11. **Yinelenmezlik:** sıra numarası açıkken addaki eski baş numara, tarih açıkken eski tarih eki önce çıkarılır; kural ikinci kez uygulanınca ad `01-01-…` olmaz.
12. **Yalnızca harf değişimi** (`rapor.txt` → `Rapor.txt`): Windows büyük-küçük harf ayırmadığı için geçici ad üzerinden iki adımda yapılır.
    Mevcut şablon adlandırması bu durumu atlıyor; yeni hat atlamaz.

### 5.2 Örnek
Girdi: `Ağustos Raporu final.docx`
Ayarlar: Türkçe dönüştür, baş harfler büyük, tire, sıra açık (01), tarih ay-Yıl, uzunluk 80.
Çıktı: `01-Agustos-Raporu-Final-08-2026.docx`

Not: Örnekte tarih 2026-08 olduğu için sonunda `08-2026` yazar. Simülasyonda gerçek tarihler kullanılıyor.

### 5.3 Ayarlar tablosu

| Ayar | Seçenekler | Varsayılan |
|---|---|---|
| Harf biçimi | Baş harfler büyük / hepsi küçük / HEPSİ BÜYÜK / olduğu gibi | Baş harfler büyük |
| Ayraç | tire (-) / alt çizgi (_) / boşluğu kaldır | tire |
| Türkçe karakter | dönüştür / koru | dönüştür |
| Sıra numarası | kapalı / açık: başlangıç, basamak, sıralama | açık, 01'den, tarihe göre |
| Tarih | kapalı / ay-Yıl / yıl-ay / gg-aa-yyyy; kaynak değiştirme veya çekim | kapalı |
| Uzunluk | 5–255 karakter (uzantı hariç), kesme | 80 |
| Kapsam | yalnız dosyalar / yalnız klasörler / ikisi | ikisi |
| Uzantıyı koru | açık / kapalı (kapalıysa yalnızca klasör adları) | açık |

### 5.4 Genel adlandırma profilleri
Sık kullanılan ayarlar hazır profil olarak sunulur:
- **Web:** küçük harf, tire, Türkçe dönüştür, uzunluk 60
- **Arşiv:** baş harf büyük, alt çizgi, sıra 01, tarih ay-Yıl, uzunluk 80
- **Serbest:** harf biçimi olduğu gibi, tire, sıra ve tarih kapalı

Kullanıcı kendi profilini kaydedebilir. Profiller **`settings.json`** içinde tutulur (AGENTS.md: ikinci bir ayar kaynağı yok).

## 6. Güvenlik ve geri alma

- Önizleme zorunlu. Uygulama için açık onay gerekir.
- Kalıcı silme yok. Üzerine yazma yok, çakışmada numaralandırma.
- Bir uygulama işlemi işlem geçmişinde tek kayıt olur ve geri alınabilir.
- Klasör yeniden adlandırma: en derin klasör önce adlandırılır, sonra üst klasör. Bu sırayla yol bozulmaz.
- Windows kısıtları: 260 karakter yol sınırı kontrolü, rezerve adlar (CON, PRN, NUL, COM1 …), ad sonunda nokta veya boşluk olamaz.
- Gizli ve sistem dosyaları atlanır. Hariç tutulan klasörler (`node_modules`, `.git`, `dist`) hiç taranmaz.
- Adlandırma yalnızca seçili klasörün içinde çalışır. Kök klasör kendisi değişmez.

## 7. Yapılacaklar (uygulama aşaması için tahmin)

Rust (yeni modül, her dosya 250 satırın altında):
- `src-tauri/src/naming/mod.rs`: kural hattının girişi
- `naming/turkish.rs`: Türkçe karakter dönüşümü
- `naming/case.rs`: harf biçimi
- `naming/numbering.rs`: sıra numarası
- `naming/date_suffix.rs`: ay-Yıl ve diğer tarih biçimleri
- `naming/length.rs`: uzunluk kesme ve çakışma numarası
- `inbox_rename.rs`: `tidy()` yerine yeni hat bağlanır
- Testler: `selection_rename_tests.rs` deseni ile birim testleri

TypeScript (yeni ve taşınan bileşenler):
- `lib/navigation.ts`, `types/app.ts`: `rename` hedefi ve `naming` ekranı; hedeflere `group: "ops"` alanı
- `AppNav.tsx`: açılır grup ve alt öğeler; `styles/workspace.css`: `.app-nav-sub`
- `src/components/FolderPickerBar.tsx`: `WorkspaceHeader`'dan taşınan klasör seçimi; `WorkspaceHeader` yalnızca genel araçlarla kalır
- `App.tsx`: klasör seçimini yalnızca ops grubunda göster; `MOVE_BAR` ve `WorkspaceFooter` kalkar
- `src/components/SelectionSummary.tsx`: seçim özeti ve saklama politikası (GroupsView ve CompareView içinde)
- `GroupCompare.tsx`: seçime göre "Bu grubu taşı" / "Seçili N grubu taşı"
- `src/components/NamingView.tsx`, `NamingOptions.tsx`, `NamingPreview.tsx`, `hooks/useNaming.ts`, `styles/naming.css`: Adlandırma ekranı
- `lib/prefs.ts`: adlandırma kalıpları `uiPrefs` içinde (`settings.json`)

- `styles/duplicates.css`: grup listesi kartı (çizgili satırlar, sol vurgu), karşılaştırma kartı; `GroupsView.tsx` ve `GroupListItem.tsx` buna göre sadeleşir
- `styles/tokens.css`: `--shadow-card`; `tasarım_rehberi.md`: gölge kuralı
- `AppNav.tsx`, `WorkspaceHeader.tsx`, `styles/workspace.css`: ikon şeridi ve daraltma düğmesi; `lib/prefs.ts`: `navCollapsed`
- `styles/*.css`: 60rem ve 44rem kırılımları

Sıra: (1) menü, daralan kenar çubuğu ve klasör seçiminin taşınması, (2) Kopyalar alt barının sayfaya alınması ve kartlar, (3) Rust ad kuralı hattı ve testleri,
(4) Adlandırma ekranı, (5) duyarlı düzen geçişi ve dar ekran denetimi.
Her adım tek başına derlenir ve `npm run check` ile doğrulanır.

## 8. Varsayımlar (onay bekliyor)

Kararlaşanlar (kullanıcı, ikinci tur):
1. Kayan panel yok. Adlandırma tam sayfa ekrandır; klasör seçimi sayfa başındadır.
2. Alt barın sağındaki düğmeler ve etiket adı kaldırılır.
3. Ad düzeni klasör adlarına da uygulanır ("klasör başlarına sıra no"); kapsam ayarından kapatılabilir.

Hâlâ varsayım:
4. Tarih kaynağı değiştirme tarihidir.
5. Yeni alt menünün adı "Adlandırma"dır (üst menü "Klasör-Dosya İşlemleri" ile karışmasın diye).
6. Türkçe harfler varsayılan olarak korunur; "kullanma" seçilince c, g, i, o, s, u olur.
7. `Rules > Otomasyon durumu` içindeki "Şablonla toplu adlandırma" paneli yerinde kalır; ileride Adlandırma ekranına taşınabilir.

## 9. Açık sorular (kullanıcı kararı)

1. Kopyalar'da 2+ grup seçiliyken yeşil düğmenin "Seçili N grubu taşı" olması uygun mu, yoksa toplu taşıma tamamen kalksın mı?
2. Üst bar (geri al, •••, tema ve ayarlar) ince bir şerit olarak kalsın mı, yoksa bu araçlar da kenar çubuğuna mı insin?
3. İzlenen bir Gelen Kutusu kaynağı veya `settings.json`'da yolu kayıtlı bir klasör yeniden adlandırılmak istenirse: atlansın mı (öneri), yoksa ayar da güncellensin mi?
4. Gölge yalnızca Kopyalar'daki iki kartta mı kalsın (öneri), yoksa uygulamadaki bütün kartlara mı gelsin?
5. "Sol tarafta belirgin renk", açık grubun solundaki kalın yeşil çizgi ve yeşil zemin olarak yorumlandı. Kastedilen buysa sorun yok; kartın tamamının renkli olması isteniyorsa değişir.

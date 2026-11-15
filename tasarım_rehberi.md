# Smart File Organizer — Tasarım Rehberi

**Durum:** Bölüm 2 ve 4 uygulamanın güncel görünümünü ve gezinmesini anlatır; renk ve ölçülerin tek kaynağı `src/styles/tokens.css` dosyasıdır. `mockup-organizer.html` ilk (turuncu paletli, sekmeli) tasarımın tarihî prototipidir; yerleşim ve davranış ilkeleri için referans olmayı sürdürür, renk ve gezinme için değil.

## 1. Tasarım yaklaşımı

Sade, sakin ve güven veren masaüstü arayüzü. Ana işlemler görünür; ileri seçenekler gruplanmış ve gerektiğinde açılabilir olmalı. Dosya taşıma ve yeniden adlandırma gibi işlemlerde etki önceden gösterilmeli, eylemler açıkça adlandırılmalıdır.

## 2. Renk ve tipografi

Yeşile çekilmiş nötrler üzerine iki vurgu kullanılır: yeşil (seçim, bağlantı) ve amber (birincil eylem). Değerler OKLCH olarak `tokens.css` içinde tanımlıdır; bileşenler renk yazmaz, token kullanır.

| Rol | Token | Kullanım |
| --- | --- | --- |
| Zemin ve yüzeyler | `--color-paper`, `--color-paper-2`, `--color-paper-3` | Sayfa zemini, kart yüzeyi, hafif dolgu |
| Metin | `--color-ink`, `--color-ink-2`, `--color-muted` | Gövde, başlık, ikincil metin |
| Kenarlık | `--color-rule` | Kart ve satır çizgileri |
| Kenar çubuğu | `--color-chrome`, `--color-chrome-2`, `--color-chrome-ink` | Koyu cam yeşili gezinme çubuğu |
| Vurgu | `--color-accent` | Seçili durum, bağlantı, onay kutusu |
| Birincil eylem | `--color-action`, `--color-action-ink` | Amber birincil düğme |
| Kararlar | `--color-keep`, `--color-move`, `--color-bin` (+ `-tint`, `-ink`) | Tut (yeşil), Taşı (amber), At (kırmızı) |
| Durumlar | `--color-ok`, `--color-warn`, `--color-bad`, `--color-exact` | Rozet ve bildirim zeminleri |

Gövde metninde okunabilir, başlıklardan gerektiği kadar koyu bir ton kullanılır. İkincil metinler arka plandan yeterince ayrışmalıdır; normal metinde en az 4,5:1 kontrast hedeflenir. İkonlar `lucide-react` setindendir. Yazı tipleri uygulamayla paketlenir: başlıklarda Rethink Sans, gövdede Hanken Grotesk, yol ve kodda JetBrains Mono; yedek olarak Segoe UI. Kullanıcı metinleri Türkçe ve anlaşılırdır.

Koyu tema (`:root.dark`) aynı ailelerden türetilir; açık tema renkleri doğrudan ters çevrilmez. Tema açık, koyu veya otomatik (sistem) seçilebilir. Renk tek başına durum göstergesi değildir: rozetlerde ayrıca metin ve gerekirse ikon bulunur.

## 3. Yerleşim ve responsive davranış

- İçerik alanı ortalanır; genişliği **en çok 1366 px** olur.
- Üst başlık ve alt işlem çubuğu sabit görünür; aradaki ana alan kaydırılır. Sabit alanlar içeriği veya odaktaki elemanları örtmez.
- Kaydırma çubuğu yalnız taşma olduğunda görünür; fare, dokunmatik yüzey ve klavye ile kaydırma her zaman kullanılabilir.
- Masaüstünde iki karşılaştırma kartı yan yana; dar alanda alt alta yerleşir. Klasör analizi iki klasör seçildiğinde analizleri alt alta sunar.
- Dar pencerelerde eylemler sarılır veya ikincil menüye alınır; dosya adı ve temel işlem erişilebilir kalır. Uzun yollar kısaltılır, tam yol erişilebilir biçimde sunulur.
- Kartlarda ince kenarlık kullanılır. Yumuşak gölge (`--shadow-card`) yalnızca ekran başlığı kartında ve Kopyalar'daki grup listesi ile karşılaştırma kartında vardır; diğer paneller gölgesizdir. Açılır katmanlar `--shadow-overlay` kullanır.
- İşlem onayları ve durum bildirimleri SweetAlert2 ile gösterilir (`src/lib/alerts.ts`, `src/styles/alerts.css`): onay ortada kalıcı pencere (odak “Vazgeç”te), başarı ve bilgi sağ üstte 6 saniyelik bildirim, hata kullanıcı kapatana kadar kalır. Düğmeler `.btn` sınıflarını, renkler token'ları kullanır. İç kartlarda daha az görsel ağırlık tercih edilir.

## 4. Navigasyon ve modüller

Sol kenar çubuğunda 6 hedef vardır (`src/lib/navigation.ts`). Kopyalar, Harita ve Adlandırma, açılır “Klasör-Dosya İşlemleri” grubunun alt öğeleridir; klasör seçimi yalnızca bu grupta, sayfanın başında görünür (`FolderPickerBar`). Kenar çubuğu 60rem altında kendiliğinden, geniş ekranda üst çubuktaki düğmeyle 64 px'lik ikon şeridine iner (`useNavRail`, `uiPrefs.navCollapsed`). Alt sekmeler ekran başlığı kartının üstündeki koyu yeşil menü çubuğundadır; seçili düğme amberdir (`ScreenHeader`, `SubTabs`). Birkaç hedefin içinde alt sekmeler bulunur:

- **Bugün:** günün gelen kutusu (Tut / Taşı / At kararları).
- **Kopyalar:** Dosya Grupları, İki Klasör Karşılaştırma, Karantina.
- **Harita:** Klasör Analizi, Klasördeki Dosyalar, Klasör Sağlığı.
- **Kurallar:** Kurallar (otomasyon, uzantı grupları, Türüne Göre Düzenle) ve Otomasyon durumu.
- **Geçmiş:** taşıma ve yeniden adlandırmaları geri alma.

Ayrı bir Dosya Eşleşme sekmesi yoktur; eşleşmeler karşılaştırma içinde yer alır. Aşağıdaki alt başlıklar ilgili alt sekmelerin içeriğini tanımlar.

### Klasör Analizi

- Kapsam seçimi: yalnız A veya A ve B. İkinci seçenek yalnız B seçiliyse kullanılabilir.
- Her klasör için adı, yolu, toplam boyut ve dosya sayısı; kategori dağılımı ve en büyük alt klasörler.
- Grafikler başlık, lejant, sayısal değer ve metin karşılığıyla sunulur. Boş, yükleniyor ve hata durumları gösterilir.

### İki Klasör Karşılaştırma

- A ve B kartları isim, yol, dosya sayısı, boyut ve kategori dağılımını gösterir. A ana klasör ve varsayılan taşıma hedefidir.
- **Dosya detaylarını göster** anahtarı eşleşme panelini açar. Eşleşmelerde kaynak, ad, yol, boyut, oluşturulma tarihi, durum ve grup numarası gösterilir.
- Birebir, isim benzeri ve eşleşmeyen dosyalar metinli, ayırt edilebilir rozetlerle gösterilir. Aynı grup numarası aynı gruba aittir; isim benzerliği içerik eşitliği anlamına gelmez.
- Birebir/isim benzeri düğmeleri ilgili dosyaları seçer; seçimin kapsamı ve sayısı açıkça görünür. Ön izleme, tarihli isimlendirme ve taşıma işlemleri bu ekrandan erişilebilir.
- Önerilen isimlendirme: `dosya_adı_14-03-2025.pdf`. Önce mevcut/yeni adların ön izlemesi, çakışma ve eksik tarih uyarısı, ardından onay gösterilir.

### Klasördeki Dosyalar

Tüm dosyalar veya kategori dosyaları seçimi; ad, kategori, boyut ve yol görünümü. Yasaklı uzantı ve karantina davranışı ekranda açıklanır. Liste 25/50/100 satır seçenekli sayfalama ile sunulur; boş ve hata durumları ayrıdır.

### Dosya Grupları

Birebir ve isim benzeri gruplar; grup başlığında ikon, tür, numara, kategori ve dosya sayısı bulunur. Seçili gruplar için otomatik ad önerilir ve kullanıcı değiştirebilir. Adın hangi gruplara uygulanacağı açıkça gösterilir. Sayfa değişse bile seçimler kaybolmaz; toplu seçimin kapsamı açıklanır.

## 5. Bileşen davranışları

- Aç/kapat kontrollerinde mevcut `Switch` kullanılır; etiket, açık/kapalı durumu ve klavye desteği sağlanır.
- İkincil işlemler gerektiğinde menüde gruplanır. İkon düğmelerinin erişilebilir adı vardır.
- Başlıklar `--color-ink-2` tonunda ve ilgili Lucide ikonuyla kullanılır.
- Yükleme, boş sonuç, erişim hatası, iptal ve kısmi başarı farklı durumlar olarak gösterilir.
- Sayfalama: toplam, mevcut aralık, önceki/sonraki, sayfa büyüklüğü; filtre/kök değişiminde ilk sayfaya dönüş.
- Ön izleme ve işlem onayı klavye ile kullanılabilir; odak göstergesi ve kapatma kontrolü görünürdür.

## 6. Kabul kontrolü

1366 px üstünde içerik genişlemiyor; dar pencerelerde yatay taşma yok; header/footer erişilebilir; kontroller açık/koyu temada okunabilir; kenar çubuğu, alt sekmeler, anahtarlar ve sayfalama klavyeyle çalışıyor; renk dışında metinli durumlar var; gerçek dosya işlemi öncesi etki ve onay gösteriliyor.

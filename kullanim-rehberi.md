# Smart File Organizer — Kullanım Rehberi

Bu rehber uygulamayı ilk kez açan bir kullanıcı için hazırlanmıştır. Uygulamanın ne yaptığını, ekranların ve modüllerin ne işe yaradığını ve ilk adımları anlatır.

---

## 1. Uygulama ne yapar?

Smart File Organizer, Windows bilgisayarınızdaki klasörleri tarayıp düzeni sağlamanıza yardımcı olan bir masaüstü uygulamasıdır. Başlıca işleri şunlardır:

- **Kopyaları bulmak:** Birebir aynı olan dosyaları (içerik olarak eşit) ve adı çok benzeyen dosyaları gruplar halinde gösterir.
- **Kopyaları ayıklamak:** Bir grubun ana dosyasını bırakıp diğerlerini karantinaya ya da etiket klasörüne taşıyabilir.
- **Klasörleri analiz etmek:** Hangi klasörün ne kadar yer kapladığını, hangi türdeki dosyaların ağırlıkta olduğunu gösterir.
- **Gelen kutusunu düzenlemek:** İndirilenler gibi bir kaynak klasöre yeni düşen dosyaları otomatik olarak gruplara ayırıp bir "Bakılacaklar" klasörüne taşır.
- **Tarihe göre gruplamak:** Dosyaları çekim veya oluşturma tarihine göre `Yıl\Ay` klasörlerine toplar ve kök klasöre aranabilir bir HTML dizin yazar.
- **Türüne göre düzenlemek:** Dosyaları uzantılarına göre kategori ve tür klasörlerine yerleştirir (önce önizleme gösterilir).
- **Her şeyi geri almak:** Yapılan taşıma ve yeniden adlandırmalar İşlem Geçmişi'nde tutulur ve geri alınabilir.

> **Önemli güvence:** Uygulama kullanıcı dosyalarını kalıcı olarak silmez ve mevcut dosyaların üzerine yazmaz. "Silme" işlemleri dosyaları Windows **Geri Dönüşüm Kutusu**'na gönderir. Hiçbir taşıma ya da silme işlemi sizin onayınız olmadan yapılmaz.

---

## 2. İlk kullanım (hızlı başlangıç)

1. Uygulamayı açın. Sol tarafta koyu yeşil bir kenar çubuğu göreceksiniz; yedi ana bölüm vardır: **Bugün, Kopyalar, Harita, Adlandırma, Tarihe Göre, Kurallar, Geçmiş**.
2. Önce **Harita** bölümüne gidip bir klasörü seçerek **Klasör analizi** ekranında boyutları inceleyebilirsiniz. Bu ekran dosyalarınızı değiştirmez.
3. Kopyalarınızı görmek için **Kopyalar > Dosya grupları** ekranını açın; klasörü seçip taramayı başlatın.
4. Gelen kutusu otomasyonunu kullanmak istiyorsanız **Kurallar** bölümünden kaynak ve yedek klasörleri seçin (bkz. Bölüm 5).
5. Her işlemden önce uygulama sizden onay ister. Kararsız kaldığınızda önce yalnızca inceleme yapan ekranlarla başlamanızı öneririz.

---

## 3. Ekranlar ve modüller

Kenar çubuğundaki her ana bölümün içinde alt sekmeler bulunur. Aşağıda her ekranın ne işe yaradığı anlatılmıştır.

### 3.1 Bugün

Günün gelen kutusudur. Otomasyonun kaynak klasöre düşen dosyaları buraya listelenir.

- **Akşam Kontrolü** (tek ekran): Her dosya için **Tut / Taşı / At** kararı verilir. Varsayılan karar **Tut**'tur.
  - **Tut:** Dosya olduğu yerde kalır.
  - **Taşı:** Dosya hedef klasöre gider; İşlem Geçmişi'nden geri alınabilir.
  - **At:** Dosya onayınızla Windows Geri Dönüşüm Kutusu'na gönderilir. Bu işlem uygulama içinden geri alınamaz.
- **Kararları uygula:** Verdiğiniz bütün kararları tek seferde uygular.
- Satırlarda küçük resimler, metin özeti ve benzer adlı dosyalar gösterilir.
- **Taşınacakları adlandır:** Seçili dosyaları bir şablonla (ör. `{grup}` veya `{özelmetin}` alanlarıyla) yeniden adlandırır.

### 3.2 Kopyalar

Aynı dosyaları bulup yönetmek için kullanılır.

- **Dosya grupları:** Bulunan gruplar sol tarafta süzülebilir bir listede yer alır.
  - Sağ tarafta ana dosya ile kopyaların karşılaştırması görünür.
  - **Saklama politikası** hangi dosyanın ana dosya olarak kalacağını belirler.
  - **Bu grubu taşı** seçili grubu taşır; toplu seçim alt çubuktan yapılır.
  - Gruplar **Birebir (Exact)** veya **Benzer (Similar)** olarak etiketlenir:
    - *Birebir:* Boyutları ve içerikleri aynı dosyalardır.
    - *Benzer:* Adları birbirine çok yakın dosyalardır. Adın benzer olması içeriğin aynı olduğu anlamına gelmez.
- **İki klasör karşılaştırma:** İki klasörü (A ve B) karşılaştırır. Hangi dosyaların her iki klasörde de bulunduğunu, hangilerinin yalnız birinde olduğunu gösterir. Aynı veya iç içe iki klasör seçilemez.
- **Karantina:** Taşınmış kopyaların bulunduğu klasördür. Buradan dosyaları asıl yerlerine geri yükleyebilir, seçtiklerinizi Geri Dönüşüm Kutusu'na gönderebilir veya 30 günden eski dosyaları seçebilirsiniz (seçim yapar, gönderme yine onaylıdır).

### 3.3 Harita

Klasörleri anlamak ve temizlik yapmak için kullanılır.

- **Klasör analizi:** Klasörün boyutunu, dosyaların türe göre dağılımını ve **Temizlenecekler** özetini gösterir. Boyuta göre orantılı bir kutu haritası sunar.
- **Klasördeki dosyalar:** Seçili klasörün dosyalarını arama, sayfalama ve ön izleme ile listeler. Buradan seçtiğiniz dosyaları bir hedef klasöre ve grup adına taşıyabilir ya da şablonla yeniden adlandırabilirsiniz.
- **Klasör sağlığı:** En büyük dosyaları ve uzun süredir değişmemiş dosyaları raporlar.

### 3.4 Adlandırma

Seçili klasördeki dosya ve klasör adlarını kurallarla toplu düzenler.

- **Hazır kalıplar** ve ayrı ayrı kurallar: kapsam (klasör, dosya veya ikisi), harf biçimi, boşluk yerine tire/alt çizgi, Türkçe harfleri kaldırma, parantez ve noktalama temizliği, başa sıra numarası, tarih ve uzunluk sınırı.
- Önizleme görmeden hiçbir ad değişmez; uygulanan işlem Geçmiş'ten geri alınabilir.

### 3.5 Tarihe Göre

Seçili klasördeki dosyaları tarihlerine göre klasörlere toplar. Fotoğraf ve videolarda çekim tarihi, diğerlerinde oluşturma tarihi kullanılır; tarihi okunamayanlar `Tarihsiz` klasörüne gider.

- **Klasör düzeni:** `Yıl\Ay` (`2026\01`), yalnız `Aya göre` (`2026-01`) veya `Yıl (sıra no)` (`01_2025`, `02_2026`).
- **Hangi dosyalar** ve **alt klasörlerdeki dosyalar da** seçenekleriyle kapsam belirlenir.
- Önizleme klasörleri, dosya sayılarını ve örnek adları gösterir. "Grupla"ya basınca taşıma sırasında yüzdeli ilerleme çubuğu görünür.
- **Dizin dosyası:** Taşımadan önce kök klasöre `Tarih_Dizini.html` yazılır. Tarayıcıda açınca hangi dosyanın hangi klasörde olduğunu görür, arama kutusuyla bulabilir ve dosya adına tıklayarak o klasörü açabilirsiniz. Her gruplama dosyanın en üstüne yeni bir bölüm ekler. Geri alma bu dosyayı güncellemez. Önizlemenin altındaki **Dizini aç** dosyayı tarayıcıda açar; **Dizini yenile** hiçbir dosya taşımadan dizini alt klasörlerin şimdiki durumundan baştan yazar (geri alma sonrası bayat kalan bölümleri temizler; kökteki dosyalar dizine girmez).
- Hiçbir dosyanın üzerine yazılmaz; aynı ad varsa `ad (1).uzantı` olur. İşlem Geçmiş'ten geri alınır.

### 3.6 Kurallar

Uygulamanın ayarlarının bulunduğu yerdir.

- **Kurallar:**
  - Gelen kutusu için **kaynak** ve **yedek** klasörleri seçersiniz.
  - Beş **uzantı grubu** (ör. Doküman, Resim, Arşiv gibi) ve bunların hedef klasörleri burada tanımlanır.
  - Sağ tarafta **Türüne göre düzenle** bölümü vardır: düzenleme seçenekleri, profiller, önizleme ve uygulama adımları burada bulunur. Önizleme görmeden hiçbir şey taşınmaz.
  - **Windows açılınca başlat:** Uygulamayı Windows başlangıcına ekler. Varsayılan olarak kapalıdır.
  - **Otomatik taşımaları bildir:** Pencere önde değilken her yerleştirmede bir Windows bildirimi gösterir. Varsayılan olarak kapalıdır.
- **Otomasyon durumu:** İzlemenin açık mı, duraklatılmış mı olduğunu, izlenen kaynakları ve şablonla adlandırma ayarlarını gösterir.

### 3.7 Geçmiş

- **İşlem geçmişi:** Yapılan taşıma ve yeniden adlandırmaların listesidir. Geri alma buradan yapılır.
  - Son 200 işlem tutulur.
  - **CSV dışa aktar** tüm taşımaları bir dosyaya yazar.

---

## 4. Güvenlik ve sık sorulan sorular

**Bir dosyayı yanlışlıkla taşıdım, geri alabilir miyim?**
Evet. **Geçmiş** ekranından son işlemi geri alabilirsiniz. Geri Dönüşüm Kutusu'na gönderilen dosyalar ise uygulama içinden geri alınamaz; Windows Geri Dönüşüm Kutusu'ndan geri yükleyebilirsiniz.

**Bir hedefte aynı adlı dosya varsa ne olur?**
Üzerine yazılmaz. Dosya adına otomatik olarak numara eklenir (ör. `ad_1.uzantı`).

**Büyük dosyalarda kopya tespiti neden farklı?**
500 MiB ve üzeri dosyalarda hızlı olması için yalnızca örnek bölgeler karşılaştırılır. Bu, tam içerik eşitliğinin kanıtı değildir. Bu tür eşleşmelerde riskli bir işlemden önce tam doğrulama seçeneğini kullanın.

**Tarihle adlandırma nasıl çalışır?**
Dosyaya `dosya_DD-MM-YYYY.uzantı` biçiminde, dosyanın gerçek oluşturulma tarihini (yerel saatle) ekler. Oluşturulma tarihi okunamayan dosyalar atlanır; değişiklik tarihi yerine kullanılmaz.

**Bulut dosyaları (OneDrive) nasıl ele alınır?**
Yalnızca çevrimiçi duran bulut dosyaları, indirilmemesi için kopya taramasında ve karşılaştırmada atlanır. Tarama özetinde bildirilir.

**Açılamayan veya kilitli dosyalar taramayı durdurur mu?**
Hayır. Atlanır ve tarama özetinde bildirilir.

**Çalıştırılabilir dosyalar (exe, bat, lnk vb.) uygulamadan açılır mı?**
Hayır. Güvenlik nedeniyle "Klasörde göster" seçeneğini kullanın.

**Uygulamayı kapatırsam gelen kutusu ne olur?**
Otomasyon açıksa ve pencere kapatıldıysa uygulama sistem tepsisinde çalışmaya devam eder. Tepsi menüsünden paneli açabilir, otomasyonu duraklatabilir veya devam ettirebilirsiniz. Uygulama tamamen kapalıyken gelen dosyalar otomatik taşınmaz; uygulamayı açtığınızda "açılışta bekleyenleri tara" ile alabilirsiniz.

---

## 5. Akıllı otomasyon (gelen kutusu) adım adım

1. **Kurallar** ekranında **kaynak klasörü** seçin (ör. İndirilenler).
2. **Yedek klasörü** seçin. Yedek klasörü kaynağın içinde olamaz.
3. Otomasyonu açın.
4. Kaynağa yeni bir dosya düştüğünde, dosya yazımı bittikten sonra (2 saniye boyunca değişmediğinde) dosya şu klasöre taşınır:

   ```text
   <Yedek>\_GelenKutusu\<YYYY-MM-DD>_Bakilacaklar\<Grup>\
   ```

5. Hiçbir gruba uymayan dosyalar kaynakta kalır.
6. Taşımalar dakikalık gruplar halinde İşlem Geçmişi'ne yazılır ve geri alınabilir.
7. **Bugün** ekranında gelen dosyaları inceleyip **Tut / Taşı / At** kararlarını verin ve **Kararları uygula** ile onaylayın.

Ayarlar `settings.json` dosyasında saklanır. Bu dosya uygulamanın veri klasöründedir; bozulursa bozuk kopyası `settings.json.bozuk` olarak saklanır.

---

## 6. Kısa sözlük

| Terim | Anlamı |
| --- | --- |
| **Birebir (Exact)** | Boyutu ve içeriği aynı dosyalar. |
| **Benzer (Similar)** | Adları çok benzer dosyalar; içerik eşitliği garanti değildir. |
| **Ana dosya** | Bir grupta saklama politikasına göre bırakılan dosya. |
| **Karantina** | Taşınan kopyaların tutulduğu klasör. |
| **Etiket klasörü** | Grup taşıma sırasında oluşturulan hedef klasör. |
| **Gelen kutusu** | Kaynak klasöre yeni düşen ve otomasyonla düzenlenen dosyalar. |
| **Yedek klasörü** | Gelen kutusunun ve düzenlenmiş dosyaların yerleştirildiği klasör. |
| **Geri alma (Undo)** | Son işlemi ya da seçili işlemi geri çevirme. |

---

## 7. Sorun giderme

- **Tarama yavaş:** Büyük klasörlerde ilk tarama uzun sürebilir. Sonraki taramalar hash önbelleği sayesinde hızlanır (`hash_cache.json`).
- **Uygulama açılmıyor:** Microsoft Edge WebView2 Runtime yüklü olmalıdır.
- **Derleme sırasında antivirüs kilidi:** Geliştiriciyseniz, `README.md` dosyasındaki alternatif test komutlarını kullanabilirsiniz.

Geliştirici bilgileri, derleme ve paketleme komutları için `README.md` ve `proje_mimarisi.md` dosyalarına bakın.

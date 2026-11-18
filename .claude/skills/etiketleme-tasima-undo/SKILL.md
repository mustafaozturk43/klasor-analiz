---
name: etiketleme-tasima-undo
description: Smart File Organizer'da etiket klasörü oluşturma, Exact/Similar gruplarının taşınması, çakışma numaralandırma ve "Son İşlemi Geri Al (Undo)" kurallarını içerir. Taşıma, karantina veya geri alma kodu yazılırken kullan.
---

# Etiketleme, Taşıma ve Undo

- Kullanıcının girdiği `<Etiket_Adı>` klasörü, seçilen **Ana Klasörün (Root Path)** hemen içinde oluşturulur: `<Ana_Klasör>/<Etiket_Adı>/`.
- **`Exact` (Birebir Kopya) grubu:**
  - **Ana dosya** saklama politikasıyla (`KeepPolicy`) seçilir: `first` (varsayılan, gruptaki ilk dosya), `newest`, `oldest` veya `shortestPath`. Ana dosya `<Ana_Klasör>/<Etiket_Adı>/` içine taşınır.
  - Geri kalan kopyalar silinmez; `<Ana_Klasör>/_Kopyalar_Karantina/<Etiket_Adı>/` klasörüne taşınır.
- **`Similar` (İsim Benzerliği) grubu:** Dosya içerikleri farklı olduğu için **tüm dosyalar** `<Ana_Klasör>/<Etiket_Adı>/` içine taşınır.
- **Çakışma:** Hedefte aynı isimde dosya varsa üzerine yazılmaz; `dosya (1).pdf` şeklinde otomatik numaralandırılır. (Gelen Kutusu'nda biçim `dosya_1.pdf`'dir.)
- **Undo:** Her taşıma ve yeniden adlandırma, dosyaya dokunulmadan **önce** uygulama veri dizinindeki `move_history.json` dosyasına yazılır. **"Son İşlemi Geri Al"** ve Geçmiş ekranındaki tek tek geri alma bu kayıttan çalışır.
- Kalıcı silme yoktur. Karantinadaki "Kutuya gönder" ve "Tümünü temizle" dosyaları kullanıcı onayıyla Windows Geri Dönüşüm Kutusu'na gönderir (`trash` crate); İşlem Geçmişi'nden geri alınamaz. Gelen Kutusu istisnası için `gelen-kutusu` skill'ine bak. Kural motoru (`engine/`) taşımaları ortak `fileops/batch_move.rs` ile yapar (önce Geçmiş'e yaz, sonra taşı); kullanıcının yazdığı "Geri Dönüşüm Kutusu'na gönder" kuralı yalnızca ikinci onayla (`allowTrash`) çalışır ve Geçmiş'e yazılmaz. Klasör sağlığı da seçili büyük/eski/0 baytlık dosyaları Geri Dönüşüm Kutusu'na gönderebilir (`scan/health_trash.rs`): tüm yollar kök içinde, karantinada değil, düz dosya ve rapordaki boyut/tarihle aynı olmalı; biri bile uymazsa hiçbiri gönderilmez.
- Taşıma (`file_ops::transfer`) aynı birimde hard link, değilse kopya kullanır; kopyada değişiklik/oluşturulma tarihleri kaynaktan aktarılır.
- İşlem Geçmişi son 200 işlemi tutar, atomik yazılır (geçici dosya + yeniden adlandırma); bozuk dosya `.bozuk` adıyla saklanır.

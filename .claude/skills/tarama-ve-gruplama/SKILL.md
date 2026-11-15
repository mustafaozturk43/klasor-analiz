---
name: tarama-ve-gruplama
description: Smart File Organizer'da klasör tarama, dosya kategorileri, hash (blake3) huni mantığı ve Exact/Similar gruplama kurallarını içerir. Tarama, hash, kopya tespiti veya isim benzerliği kodu yazılırken veya değiştirilirken kullan.
---

# Tarama ve Gruplama

## 1. Hedef Dosya Kategorileri
- `Doküman`: pdf, docx, doc, xlsx, xls, pptx, ppt, txt, csv, odt
- `Resim`: jpg, jpeg, png, webp, bmp, tif, tiff, gif, heic
- `Arşiv`: zip, rar, 7z, tar, gz, iso

Bu liste "Türüne Göre Düzenle" içindir. Gelen Kutusu'nun 5 uzantı grubundan ayrıdır (bkz. `gelen-kutusu` skill'i).

## 2. Huni (Funnel) Tarama ve 500 MB Kuralı
- Klasör taraması `jwalk` ve `rayon` ile çok çekirdekli (multi-thread) yapılır.
- `_Kopyalar_Karantina` ve `_GelenKutusu` klasörleri taramadan hariç tutulur.
- Okunamayan klasör veya dosya (erişim izni yok, başka programca kilitli) taramayı durdurmaz: hiçbir gruba girmez ve `ScanResult.warnings` içinde bildirilir.
- **Boyut < 500 MB:** Önce ilk 4 KB hızlı hash (`blake3`) alınır. Eşleşme varsa tam dosya `blake3` hash'i alınır.
- **Boyut >= 500 MB (524.288.000 Byte):** Diski yormamak için `Seek` ile sadece **İlk 1 MB + Tam Orta 1 MB + Son 1 MB** okunur. Bu veri dosya boyutuyla birleştirilip `blake3` hash ID üretilir.

## 3. Çift Katmanlı Gruplama
- **Grup Tipi 1 (`Exact` - Birebir Kopya):** Boyutu ve BLAKE3 Hash ID'si %100 aynı olan dosyalar. Grup kimliği `GRP-XXXX`.
- **Grup Tipi 2 (`Similar` - İsim/Versiyon Benzerliği):** Birebir kopya olmayan ama aynı kategoride olup dosya adı (uzantısız `stem`) **Jaro-Winkler (`strsim`)** algoritmasına göre **>= 0.82** benzeyen dosyalar.
  - **Sayaç istisnası (`similarity::is_sequence_pair`):** Adlar yalnızca en az 3 haneli bir sayaç veya zaman damgasıyla ayrılıyorsa (`IMG_0001`/`IMG_0002`, `scan001`/`scan002`, `Screenshot_20240101-101500`) benzer sayılmaz; aksi halde binlerce fotoğraf tek gruba zincirlenir. Kısa numaralar (`v1`/`v2`, `(1)`/`(2)`) ve yıllar (`fatura_2024`/`fatura_2025`) benzer kalır.

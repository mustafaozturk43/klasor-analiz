---
name: proje-kodlama-standartlari
description: Smart File Organizer'ın teknoloji yığını, npm zorunluluğu, UI/stil prensipleri, tablo performansı, token tasarrufu ve 250 satır / tek sorumluluk kodlama kuralları. Yeni dosya oluştururken, bağımlılık eklerken veya arayüz yazarken kullan.
---

# Proje Kodlama Standartları

## Teknoloji Yığını
- **Paket yöneticisi:** Sadece `npm` (pnpm, yarn veya bun kesinlikle yasak).
- **Çerçeve:** Tauri v2 (Rust backend) + React 19 + TypeScript (Vite tabanlı).
- **Stil & UI:** Tailwind CSS v4 (`@tailwindcss/vite`), `lucide-react` ikon seti, `shadcn/ui` tarzı Radix bileşenleri; yeşil-amber palet (koyu cam yeşili kenar çubuğu, amber birincil düğme), Rethink Sans / Hanken Grotesk, açık ve koyu tema, sade ve yoğun olmayan düzen. Renk ve ölçüler `src/styles/tokens.css` token'larından alınır; ayrıntı `tasarım_rehberi.md` içindedir.
- **Tablo performansı:** Binlerce dosyada DOM şişmesini önlemek için `@tanstack/react-virtual`.

## Token Tasarrufu ve Kodlama
- `node_modules/`, `src-tauri/target/`, `.git/` ve `dist/` klasörlerini ASLA okuma veya tarama.
- Spagetti/monolitik kod yazma. Hem Rust (`src-tauri/src/`) hem React (`src/`) tarafında her dosya tek bir sorumluluğa (Single Responsibility) sahip olur ve **250 satırı geçmez**. Kural uzun satırlarla aşılmaz: tek satır **en fazla 300 karakterdir**. İkisini de `npm run audit:source` denetler.
- Dosyaları düzenlerken tüm dosyayı baştan yazmak yerine sadece ilgili modülü güncelle.

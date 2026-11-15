# Klasör-Dosya İşlemleri Uygulama Planı

> **Durum:** Uygulandı (2026-11-15). `npm run build`, `npm run audit:source` ve `cargo test --no-default-features` geçti; arayüz geliştirme taklidiyle tarayıcıda 1360 ve 640 piksel genişlikte doğrulandı. Gerçek Tauri penceresinde, gerçek bir klasörde adlandır ve geri al denemesi yapılmadı. Rust modülleri planlanandan farklı adlandırıldı: `name_rules.rs`, `name_rules_plan.rs`, `name_rules_apply.rs`. `tagName`/`autoTag` durumu `useAppState` içinde duruyor (arayüzü kalktı).

**Amaç:** Kenar çubuğunda "Klasör-Dosya İşlemleri" grubu (Kopyalar, Harita, Adlandırma), sayfa başında klasör seçimi, başlık kartı ve menü çubuğu, Kopyalar kartları, daralan kenar çubuğu ve kural tabanlı klasör/dosya adlandırma.

**Tasarım:** `docs/superpowers/specs/2026-11-15-harita-klasor-islemleri-design.md`
**Simülasyon:** `docs/simulasyon/klasor-dosya-islemleri.html`

## Genel kısıtlar

- Yalnızca `npm`. Her kaynak dosya en fazla 250 satır, satır en fazla 300 karakter (`npm run audit:source`; `.css` dahil).
- `src/styles/workspace.css` 206 satır: yeni stiller oraya sığmaz, yeni dosyalara yazılır ve `src/index.css` içine eklenir.
- Üzerine yazma yok, kalıcı silme yok, onaysız değişiklik yok. Ayarlar yalnızca `settings.json` (`uiPrefs`, `src/lib/prefs.ts`).
- Yeni her `invoke` komutu `src-tauri/src/lib.rs` içindeki `generate_handler!` listesine ve `src/dev/tauriMock.ts` içine eklenir.
- Proje git deposu değil; "commit" adımı yok. Her görev `npm run check` ile kapanır.

## Kod incelemesinden çıkan kararlar

- `src-tauri/src/naming.rs` zaten var (etiket temizleme, `unique_dest`). Yeni kural hattı `naming/` klasörü değil, ayrı düz modüllerdir: `name_rules.rs` (saf dönüşüm), `name_rules_plan.rs` (ağaç planı), `name_rules_apply.rs` (uygulama), testleri `*_tests.rs`.
- `file_ops::transfer` hard link + `remove_file` kullanır: klasörlerde ve yalnızca harf değişiminde çalışmaz. Adlandırma uygulaması `fs::rename` kullanır; yalnızca harf değişiminde geçici ad üzerinden iki adım.
- `mover::undo_move` da `transfer` kullanır: klasör kayıtlarını geri alamaz. Geri alma, kaynağı klasör olan veya yalnızca harfi değişen kayıtlar için `fs::rename` yoluna ayrılmalı (kayıtlar ters sırada işlenir, bu sıra klasörler için doğru).
- Uygulama sırası: en derin öğe önce. `UndoRecord.moves` bu sırayla yazılır.
- `rename::apply_pending` dosya damgası (boyut, mtime) doğrular; klasörlerde yalnızca varlık ve hedefin boş olması doğrulanır.
- `App.tsx` (135 satır) ekranları `hidden` ile saklar; yeni ekran aynı kalıba uyar. `Module` tipine `naming`, `Destination` tipine `rename` eklenir.
- Geliştirme taklidi (`src/dev/tauriMock.ts`) sayesinde arayüz `npm run dev` ile tarayıcıda doğrulanabilir.

## Görevler

1. **Menü grubu ve daralan kenar çubuğu.** `types/app.ts`, `lib/navigation.ts` (hedeflere `group?: "ops"`, ekranlara `icon`), `AppNav.tsx`, yeni `styles/nav.css` (workspace.css'teki `.app-nav` kuralları buraya taşınır), `lib/prefs.ts` anahtarı `navCollapsed`. 60rem altında kendiliğinden ikon şeridi.
2. **Klasör seçimi ve başlık kartı.** Yeni `FolderPickerBar.tsx` (`WorkspaceHeader`'dan taşınır, `WorkspaceSettings` satırı altına girer), yeni `ScreenHeader.tsx` (başlık, rozet, `SubTabs` menü çubuğu, sayı şeridi), `App.tsx` sadeleşir, yeni `styles/screen-head.css`.
3. **Kopyalar.** `styles/duplicates.css` (liste kartı, çizgili satırlar, sol vurgu, karşılaştırma kartı), `tokens.css` `--shadow-card`, yeni `SelectionSummary.tsx`, `GroupCompare.tsx` düğmesi seçime göre; `WorkspaceFooter.tsx` ve `MOVE_BAR` kalkar; `CompareView` içindeki "alt çubukta" metni ve taşıma düğmesi sayfaya alınır; `useAppState`'ten `tagName`/`autoTag` temizliği.
4. **Rust ad kuralı hattı.** Tasarımın 5.1 bölümündeki 12 kural; `NameRuleOptions` (camelCase serde); komut `preview_name_rules(rootPath, options) -> RenamePlan`, uygulama `apply_name_rules(planId)`. Testler: Türkçe harf (`IŞIK` → `Işık`, `isik`), çakışma ` (1)`, yinelenmezlik, uzunluk kesme, yalnızca harf değişimi, klasör + içindeki dosya birlikte, geri alma, 260 karakter, rezerve adlar (CON, NUL), karantina ve izlenen kaynak klasörünün atlanması.
5. **Adlandırma ekranı.** `lib/namingCommands.ts`, `hooks/useNaming.ts`, `NamingView.tsx`, `NamingOptions.tsx`, `NamingPreview.tsx`, `styles/naming.css`; kalıplar `uiPrefs.namingPresets`; taklit komutlar.
6. **Duyarlı düzen.** 60rem ve 44rem kırılımları, `tasarım_rehberi.md` güncellemesi (gölge, ikon şeridi), `npm run check`.
7. **Uygulamada doğrulama.** `npm run dev` ile geniş ve dar ekranda üç sayfa; gerçek klasörde adlandır ve geri al.

## Açık sorular (varsayılanla ilerlenecek)

Tasarımın 9. bölümündeki beş soru yanıtsız; plan oradaki önerileri esas alır.

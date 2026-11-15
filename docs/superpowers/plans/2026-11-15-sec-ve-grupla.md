# Seç ve Grupla — Uygulama Planı

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harita > Klasördeki dosyalar ekranına Windows ön izlemeli liste, çekim tarihi öncelikli süzme, seçim, şablonla adlandırma ve "hedef klasör + grup adı" ile geri alınabilir taşıma eklemek.

**Architecture:** Rust tarafında dört yeni sorumluluk eklenir: Windows Shell küçük resim/çekim tarihi, alt klasör listesi, seçilenleri taşıma (ön izle → bekleyen plan → uygula, Geçmiş'e önce yazarak) ve gelen kutusu adlandırma motorunun kök sınırlı genellemesi. React tarafında `FileBrowser` ince bir kabuk olur; seçim, tarih okuma ve küçük resimler hook'lara, çubuk/diyaloglar küçük bileşenlere ayrılır.

**Tech Stack:** Tauri v2 (Rust 2024, `windows` 0.62), React 19 + TypeScript, Tailwind v4, `lucide-react`, `@tauri-apps/plugin-dialog`.

**Spec:** [docs/superpowers/specs/2026-11-15-sec-ve-grupla-design.md](../specs/2026-11-15-sec-ve-grupla-design.md) · Simülasyon: [docs/sec-ve-grupla-mockup.html](../../sec-ve-grupla-mockup.html)

## Global Constraints

- Paket yöneticisi yalnızca `npm`; yeni npm bağımlılığı eklenmez.
- Her kaynak dosya ≤ 250 satır, satır ≤ 300 karakter, tek sorumluluk (`npm run audit:source`).
- Kalıcı silme yok (`fs::remove_file`/`remove_dir_all` kullanıcı dosyasında kullanılmaz); hedefte üzerine yazma yok.
- Hiçbir taşıma/adlandırma kullanıcı onayı olmadan yapılmaz; her biri Geçmiş'e **işlemden önce** yazılır.
- Renkler yalnızca `src/styles/tokens.css` token'ları; ikonlar `lucide-react`; kullanıcı metinleri Türkçe.
- Ayarlar yalnızca `settings.json`; bu özellik yeni kalıcı ayar eklemez (hedef seçimi yalnızca oturumluk).
- Taşıma çakışma biçimi `ad (1).uzantı` (`naming::Suffix::Paren`), adlandırma çakışma biçimi `ad_1.uzantı`.
- Geçmiş etiketleri: `"Seçilenleri taşı · <hedef klasör adı>"`, `"Seçilenleri adlandır"`.
- Proje bir git deposu değildir: "commit" adımları yoktur; her görevin sonu doğrulama komutudur.
- Rust testleri: `cargo test --manifest-path src-tauri/Cargo.toml --no-default-features <filtre>`.

## Review Focus

1. Ön izleme ile uygulama arasında seçili dosya değişti/silindi → uygula hiçbir şey taşımadan "Dosya veya hedef değişti; yeniden ön izleyin." der (Task 2 testi `apply_refuses_changed_file`).
2. A ve B kökünden aynı adlı iki dosya aynı hedefe → `x.jpg` ve `x (1).jpg`; hedefte zaten `x.jpg` varsa `x (1).jpg`, `x (2).jpg` (Task 2 testi `same_name_from_two_roots_gets_numbered`).
3. Grup adında `..\Windows`, `a\\b`, `CON` gibi değerler → reddedilir, kök dışına çıkılamaz (Task 2 testi `group_rejects_traversal_reserved_and_empty_segments`).
4. Çekim tarihleri okunurken başka klasöre geçilir → eski klasörün sonuçları yeni listeye yazılmaz (Task 7 tarayıcı doğrulaması adım 4).
5. Hedef olarak karantina klasörü veya içi seçilir → reddedilir (Task 2 testi `rejects_outside_roots_quarantine_and_same_folder`).

---

## Dosya haritası

| Dosya | Durum | Sorumluluk |
|---|---|---|
| `src-tauri/src/models.rs`, `scanner.rs`, `folder_stats.rs`, `compare.rs` | Değişir | `FolderFile.modified_at` |
| `src-tauri/src/selection_paths.rs` | Yeni | Grup adı parçalama/doğrulama, `free_dest` |
| `src-tauri/src/selection_move.rs` (+ `_tests.rs`) | Yeni | Taşıma planı, bekleyen plan, komutlar |
| `src-tauri/src/inbox_rename.rs` | Değişir | `plan_rename_in` genellemesi, `group`/`dates`/`tidy` seçenekleri |
| `src-tauri/src/selection_rename.rs` (+ `_tests.rs`) | Yeni | `preview_selection_rename` komutu |
| `src-tauri/src/subfolders.rs` | Yeni | Hedef seçici için klasör ağacı |
| `src-tauri/src/shell_thumb.rs`, `media_dates.rs`, `shell_cmds.rs` | Yeni | Windows Shell küçük resim, çekim tarihi, komutlar |
| `src-tauri/src/lib.rs`, `Cargo.toml` | Değişir | Modül/komut kaydı, `windows` özellikleri |
| `src/types/file.ts` | Değişir | `FolderFile.modifiedAt` |
| `src/lib/selectionCommands.ts` | Yeni | invoke sarmalayıcıları + tipler |
| `src/lib/fileKinds.ts`, `src/lib/groupPath.ts` | Yeni | Tür çipleri, etkin tarih, hedef yol çözümü |
| `src/dev/tauriMock.ts`, `src/dev/mockData.ts` | Değişir | Yeni komut mock'ları |
| `src/hooks/useFileSelection.ts`, `useMediaDates.ts`, `useShellThumbs.ts`, `useFolderTargets.ts`, `useSelectionMove.ts` | Yeni | Durum mantığı |
| `src/components/FileBrowser.tsx` | Değişir | Kabuk/kompozisyon |
| `src/components/FileBrowserFilters.tsx`, `FileBrowserRow.tsx`, `BrowserPreview.tsx`, `SelectionBar.tsx`, `TargetFolderSelect.tsx`, `SelectionMoveDialog.tsx`, `TemplateRenameDialog.tsx` | Yeni | UI parçaları |
| `src/components/InboxRenameDialog.tsx` | Değişir | `TemplateRenameDialog` sarmalayıcısı |
| `src/App.tsx` | Değişir | `FileBrowser`'a `onPreview` geçilmez |
| `src/styles/browser.css` | Yeni | Satır/küçük resim/çubuk stilleri (`index.css`'ten import) |

> Spec'ten tek sapma: büyük ön izleme için global `FilePreview` değiştirilmez; `FileBrowser` kendi `BrowserPreview` bileşenini kullanır (diğer ekranlar etkilenmez, önceki/sonraki/seç bu ekrana özgüdür).

---

### Task 1: `FolderFile.modifiedAt`

**Files:**
- Modify: `src-tauri/src/models.rs:178-185`, `src-tauri/src/scanner.rs:236-243`, `src-tauri/src/folder_stats.rs:147`, `src-tauri/src/compare.rs:52`
- Modify: `src/types/file.ts` (`FolderFile`), `src/dev/mockData.ts` (`folderFile`)
- Test: `src-tauri/src/file_list_tests.rs` (yeni; `file_list.rs` sonuna `#[cfg(test)] #[path = "file_list_tests.rs"] mod tests;`)

**Interfaces:**
- Produces: Rust `FolderFile { ..., pub modified_at: Option<u64> }` (unix saniye); TS `FolderFile.modifiedAt: number | null`.

- [ ] **Step 1: Failing test** — `list_files_reports_modified_time`: geçici klasöre `a.txt` yaz, `list_files(root, Some(false), None, None, Some(true))` → tek öğe, `modified_at.is_some()` ve `created_at`'tan bağımsız olarak `SystemTime::now()`'dan en çok 60 sn eski.
- [ ] **Step 2:** `cargo test ... list_files_reports_modified_time` → derleme hatası (`modified_at` yok).
- [ ] **Step 3:** Alanı ekle; dört kurulum yerinde `metadata.modified()` → unix saniye. TS tipine ve `mockData.folderFile`'a `modifiedAt` ekle (mock: sabit `1718366400`).
- [ ] **Step 4:** `cargo test ... list_files_reports_modified_time` PASS; `npm run build` hatasız.

---

### Task 2: Seçilenleri taşıma (arka uç)

**Files:**
- Create: `src-tauri/src/selection_paths.rs`, `src-tauri/src/selection_move.rs`, `src-tauri/src/selection_move_tests.rs`
- Modify: `src-tauri/src/lib.rs` (modüller, `.manage(selection_move::SelectionMoveState::default())`, komutlar)

**Interfaces:**
- Consumes: `organize_rules::check_folder_name`, `path_guard::{canonical, ensure_inside, in_quarantine}`, `rename::FileStamps`, `operations::MoveOutcome`, `organize_models::FailedMove`, `file_ops::transfer`, `history::{push, save}`.
- Produces:
  - `selection_paths::split_group(group: &str) -> Result<Vec<String>, String>`
  - `selection_paths::free_dest(dir: &Path, name: &str, reserved: &mut HashSet<String>) -> PathBuf`
  - `#[derive(Deserialize)] #[serde(rename_all = "camelCase")] pub struct SelectionMoveRequest { pub roots: Vec<String>, pub paths: Vec<String>, pub target_base: String, pub group: String }`
  - `#[derive(Serialize, Clone)] #[serde(rename_all = "camelCase")] pub struct SelectionMovePlan { pub id: String, pub target_dir: String, pub created_dirs: Vec<String>, pub entries: Vec<MoveRecord>, pub renamed: usize }`
  - `pub fn plan_selection_move(req: &SelectionMoveRequest) -> Result<(SelectionMovePlan, FileStamps), String>`
  - `pub(crate) fn apply_selection_core(app: &AppState, plan: &SelectionMovePlan, stamps: &FileStamps) -> Result<MoveOutcome, String>`
  - Komutlar: `preview_selection_move(state, request) -> SelectionMovePlan`, `apply_selection_move(state, app, plan_id: String) -> MoveOutcome`

Kurallar (spec §8.1, §8.2):
- `split_group`: `\` veya `/` ile böl, her parçayı `trim`; boş metin → `Ok(vec![])`; ardışık ayraç veya boş parça → `Err("Klasör adları arasında boş bölüm var.")`; her parça `check_folder_name("Klasör", ..)` (`.` ve `..` boşa döner → hata); `CON PRN AUX NUL COM1-9 LPT1-9` (noktadan önceki kısım, büyük/küçük harf duyarsız) → `Err("“X” Windows'ta ayrılmış bir ad.")`.
- `plan_selection_move`: `paths` boş → hata; her yol kanonik olarak bir kökün içinde, karantinada değil, dosya, tekrar yok; `target_base` var olan klasör ve hiçbir kökün karantinasında değil (bileşenlerde `QUARANTINE_DIR` geçemez); `target_dir = target_base + parçalar`; herhangi bir dosyanın üst klasörü `target_dir` ise `Err("Dosyalar zaten bu klasörde.")`; `created_dirs` = `target_base`'den itibaren var olmayan ara klasörler; her dosya için `free_dest(target_dir, name, reserved)` (`ad (n).uzantı`, n=1'den); yeni yol > 259 karakter → `Err("<ad>: yeni yol 260 karakteri aşıyor.")`; `id = "SM-<nanos>"`.
- `apply_selection_core`: önce tüm damgaları ve hedeflerin yokluğunu doğrula (biri tutmazsa hiçbir şey yapmadan `Err("Dosya veya hedef değişti; yeniden ön izleyin.")`); `create_dir_all(target_dir)`; `UndoRecord { id: plan.id, tag: "Seçilenleri taşı · <target_dir adı>", moves: vec![] }` push+save; her dosyada kaydı ekle → save → `transfer`; hata olursa kaydı çıkar, `failed`'a ekle, devam et; hiç taşınmadıysa geçmiş kaydını kaldır ve `prune_empty_dirs(target_base, created_dirs, &|_| false)`.

- [ ] **Step 1: Failing tests** (`selection_move_tests.rs`, `operations_tests.rs`'deki `Fx` düzeninde geçici kök + `AppState`):

```rust
#[test] fn group_segments_split_on_both_slashes() {
    assert_eq!(split_group(" Faturalar \\ Mayıs/ Su ").unwrap(), vec!["Faturalar", "Mayıs", "Su"]);
    assert!(split_group("  ").unwrap().is_empty());
}
#[test] fn group_rejects_traversal_reserved_and_empty_segments() {
    for g in ["..\\Windows", "a\\\\b", "CON", "lpt1.txt", "a:b", "x/./y"] { assert!(split_group(g).is_err(), "{g}"); }
}
#[test] fn moves_into_nested_new_group_under_base() {
    // kök/a.pdf, kök/Faturalar/ var; base = kök/Faturalar, group = "Mayıs"
    // plan.target_dir == kök/Faturalar/Mayıs, plan.created_dirs == [o yol], entries[0].to == .../Mayıs/a.pdf, renamed == 0
}
#[test] fn empty_group_moves_directly_into_base() { /* group "" base kök/Faturalar → target_dir kök/Faturalar, created_dirs boş */ }
#[test] fn same_name_from_two_roots_gets_numbered() {
    // A/x.jpg, B/x.jpg, hedefte x.jpg var → to adları ["x (1).jpg", "x (2).jpg"], renamed == 2
}
#[test] fn rejects_outside_roots_quarantine_and_same_folder() {
    // kök dışı dosya → Err; base = kök/_Kopyalar_Karantina → Err; base = dosyanın klasörü & group "" → Err "Dosyalar zaten bu klasörde."
}
#[test] fn apply_moves_and_writes_history_before_transfer() {
    // apply → dosyalar hedefte, kaynakta yok; app.history[0].tag == "Seçilenleri taşı · Mayıs"; moves.len() == 2
    // crate::mover::undo_move(&app.history[0]) → dosyalar eski yerinde
}
#[test] fn apply_refuses_changed_file() {
    // plan sonrası dosyaya bayt ekle → apply Err("Dosya veya hedef değişti; yeniden ön izleyin."), dosya yerinde, app.history boş
}
```

- [ ] **Step 2:** `cargo test ... selection_move` → derleme hatası.
- [ ] **Step 3:** `selection_paths.rs` ve `selection_move.rs`'i yukarıdaki imza ve kurallarla yaz; `SelectionMoveState { pending: Mutex<Option<(SelectionMovePlan, FileStamps)>> }`; komutlar `#[tauri::command(async)]`, `apply` bekleyen planın `id`'si eşleşmezse `Err("Plan güncel değil.")`. `lib.rs`'e kaydet.
- [ ] **Step 4:** `cargo test ... selection` PASS (8 test); `npm run audit:source` (Rust satır sınırı dahil) temiz.

---

### Task 3: Seçilenleri adlandırma (arka uç)

**Files:**
- Modify: `src-tauri/src/inbox_rename.rs`
- Create: `src-tauri/src/selection_rename.rs`, `src-tauri/src/selection_rename_tests.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `rename_template::{render, Fields}`, `rename::{Pending, RenameState, RenamePlan}`, `apply_rename_plan` (değişmez).
- Produces:
  - `ListRenameOptions`'a eklenen alanlar: `#[serde(default)] pub group: Option<String>`, `#[serde(default)] pub dates: Vec<Option<u64>>`, `#[serde(default)] pub tidy: bool`
  - `pub fn plan_rename_in(bases: &[PathBuf], outside_msg: &str, paths: &[String], opts: &ListRenameOptions) -> Result<(RenamePlan, FileStamps), String>`; `plan_list_rename(inbox, paths, opts)` bunu `&[inbox.to_path_buf()]` ve mevcut mesajla çağırır.
  - Komut `selection_rename::preview_selection_rename(state, roots: Vec<String>, paths: Vec<String>, options: ListRenameOptions) -> RenamePlan` (etiket `"Seçilenleri adlandır"`, dış dosya mesajı `"Yalnızca açık klasördeki dosyalar adlandırılabilir: {raw}"`).

Kurallar: `group` doluysa `{Grup}` her dosyada bu değeri alır (yoksa bugünkü klasör türetmesi); `dates[i]` `Some` ise `Fields.mtime` yerine kullanılır; `tidy` ise üretilen kökte `_{2,}` → `_` ve baştaki/sondaki `_`, `-`, boşluk kırpılır.

- [ ] **Step 1: Failing tests:**
  - `selection_rename_uses_given_dates_and_group`: kök/`IMG_1.jpg`, `dates = [Some(1718366400)]`, template `"{Grup}_{tarih}_{sıra}"`, group `Some("Mayıs")`, pad 2, start 1 → `to` dosya adı `"Mayıs_2024-06-14_01.jpg"`.
  - `selection_rename_tidies_empty_fields`: template `"{OzelMetin}_{Grup}_{sıra}"`, custom `""`, group `Some("Mayıs")`, `tidy: true` → `"Mayıs_01.jpg"`.
  - `selection_rename_rejects_paths_outside_roots`: kök dışı yol → `Err` mesajı `"Yalnızca açık klasördeki dosyalar adlandırılabilir"` ile başlar.
- [ ] **Step 2:** `cargo test ... selection_rename` → derleme hatası.
- [ ] **Step 3:** Genellemeyi yap, yeni komutu yaz ve kaydet. `inbox_rename.rs` 250 satırın altında kalmalı.
- [ ] **Step 4:** `cargo test ... rename` PASS (yeni 3 + mevcut gelen kutusu/rename testleri).

---

### Task 4: Hedef seçici için klasör ağacı

**Files:**
- Create: `src-tauri/src/subfolders.rs` (testler dosya içinde `mod tests`)
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `#[derive(Serialize)] #[serde(rename_all = "camelCase")] pub struct SubfolderList { pub paths: Vec<String>, pub truncated: bool }`; `pub fn collect_subfolders(root: &Path, excludes: &[String], max_depth: usize, limit: usize) -> SubfolderList`; komut `list_subfolders(root: String, exclude_folders: Vec<String>) -> Result<SubfolderList, String>` (`max_depth = 6`, `limit = 2000`).

Kurallar: mutlak yollar, kökün kendisi hariç, ada göre (`to_lowercase`) sıralı; `QUARANTINE_DIR`, `.` ile başlayanlar ve `excludes` (ad, büyük/küçük harf duyarsız) alt ağaçlarıyla birlikte atlanır.

- [ ] **Step 1: Failing tests:** `lists_nested_folders_skipping_quarantine_and_excludes` (kök/A/B, kök/_Kopyalar_Karantina/x, kök/node_modules; excludes `["node_modules"]` → `[kök\A, kök\A\B]`, `truncated == false`); `marks_truncated_at_limit` (limit 1 → `paths.len() == 1`, `truncated == true`).
- [ ] **Step 2:** FAIL → **Step 3:** uygula (`std::fs::read_dir` ile BFS) → **Step 4:** `cargo test ... subfolders` PASS.

---

### Task 5: Windows Shell küçük resim ve çekim tarihi

**Files:**
- Create: `src-tauri/src/shell_thumb.rs`, `src-tauri/src/media_dates.rs`, `src-tauri/src/shell_cmds.rs`
- Modify: `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`

**Interfaces:**
- Produces:
  - `shell_thumb::shell_thumbnail(path: &Path, size: u32, cache_dir: &Path) -> Option<PathBuf>` (PNG yolu)
  - `media_dates::date_taken(path: &Path) -> Option<u64>` (unix saniye, UTC)
  - Komutlar: `get_shell_thumbnails(app: AppHandle, paths: Vec<String>, size: u32) -> Vec<Option<String>>`, `read_media_dates(paths: Vec<String>) -> Vec<Option<u64>>` (ikisi de `async`, giriş sırasını korur)

Kurallar: `Cargo.toml` → `windows = { version = "0.62", features = ["Win32_Foundation", "Win32_Graphics_Gdi", "Win32_System_Com", "Win32_UI_Shell", "Win32_UI_Shell_PropertiesSystem"] }`. Önbellek `app_cache_dir()/thumbs/shell`, anahtar `blake3(yol küçük harf | boyut | mtime | size)`'ın ilk 24 hex'i (`thumbnail.rs::cache_key` gibi), `.tmp.png` → `rename`. `size` 32..=1024 aralığına kıstırılır. `GetImage(SIZE{size,size}, SIIGBF_RESIZETOFIT)`; HBITMAP `GetDIBits` ile 32 bit üstten-alta BGRA okunur, tüm alfa 0 ise 255 yapılır, `DeleteObject` her yolda çağrılır. Tarih: `IShellItem2::GetFileTime(&PKEY_Photo_DateTaken)`, yoksa `PKEY_Media_DateEncoded`; FILETIME → unix (`(ft - 116444736000000000) / 10_000_000`). Her işçi iş parçacığı `CoInitializeEx(None, COINIT_APARTMENTTHREADED)` + bitişte `CoUninitialize`; en çok 4 iş parçacığı (`std::thread::scope`, parçalara bölerek).

- [ ] **Step 1: Failing tests** (`shell_thumb.rs`/`media_dates.rs` içinde): `thumbnail_for_png_is_written_as_png` (image crate ile 64×48 PNG üret → `Some(p)`, `p` dosyası `\x89PNG` ile başlar); `thumbnail_for_text_file_falls_back_to_icon` (`a.txt` → `Some`); `thumbnail_for_missing_file_is_none`; `png_without_exif_has_no_date_taken` (`date_taken` → `None`).
- [ ] **Step 2:** FAIL → **Step 3:** uygula → **Step 4:** `cargo test ... shell_thumb media_dates` PASS.
- [ ] **Step 5: Elle doğrulama** (gerçek dosya): telefondan bir JPEG için `date_taken` değerinin Dosya Gezgini "Çekim tarihi" ile aynı gün olduğunu geçici bir `#[ignore]` test veya `cargo test -- --ignored` ile kontrol et; sonucu görev notuna yaz.

---

### Task 6: Ön yüz altyapısı (tipler, komutlar, saf yardımcılar, mock'lar)

**Files:**
- Create: `src/lib/selectionCommands.ts`, `src/lib/fileKinds.ts`, `src/lib/groupPath.ts`
- Modify: `src/dev/tauriMock.ts`, `src/dev/mockData.ts`

**Interfaces:**
- Produces:
  - `selectionCommands.ts`: `SelectionMoveRequest { roots: string[]; paths: string[]; targetBase: string; group: string }`, `SelectionMovePlan { id: string; targetDir: string; createdDirs: string[]; entries: MoveRecord[]; renamed: number }`, `SubfolderList { paths: string[]; truncated: boolean }`, `SelectionRenameOptions = ListRenameOptions & { group: string; dates: (number | null)[]; tidy: true }`; fonksiyonlar `listSubfolders(root, excludeFolders)`, `getShellThumbnails(paths, size)`, `readMediaDates(paths)`, `previewSelectionMove(request)`, `applySelectionMove(planId)` (`MoveOutcome` — `inboxCommands.ts`), `previewSelectionRename(roots, paths, options)`.
  - `fileKinds.ts`: `type KindChip = "all" | "image" | "video" | "pdf" | "doc" | "other"`; `kindOf(name: string): Exclude<KindChip, "all">` (image: jpg jpeg png gif bmp webp tif tiff heic heif dng cr2 nef arw svg · video: mp4 mkv avi mov wmv m4v webm 3gp mts · pdf · doc: doc docx odt rtf txt md xls xlsx ods csv ppt pptx odp); `KIND_LABELS`; `effectiveDate(modifiedAt: number | null, takenAt: number | null | undefined): number | null`; `ymd(seconds: number): { y: string; m: string; d: string }` (yerel saat).
  - `groupPath.ts`: `type GroupTarget = { ok: true; base: string; target: string; parts: { name: string; exists: boolean }[]; created: string[] } | { ok: false; error: boolean; message: string }`; `resolveGroupTarget(base: string, group: string, existing: string[], currentDir: string): GroupTarget` — Task 2'deki `split_group` kurallarının aynısı ve mesajları; var olan klasör eşleşmesi `toLocaleLowerCase("tr")`; `target === currentDir` → `{ ok: false, error: false, message: grup boşsa "Grup adı yazın ya da bir hedef klasör seçin." değilse "Dosyalar zaten bu klasörde." }`. `suggestGroupName(dates: (number | null)[], hasTarget: boolean): string | null` (hepsi aynı yıl-ay → `"Mayıs"` ya da `"Mayıs 2024"`).
  - Mock'lar: `list_subfolders` → mock kökü altındaki 5 klasör + `Faturalar\\Nisan`; `get_shell_thumbnails` → hepsi `null`; `read_media_dates` → `.jpg`'lerde `1718366400`, diğerlerinde `null` (her çağrıda 300 ms gecikme); `preview_selection_move` → istekten türetilmiş plan; `apply_selection_move` → `{ moved: entries, failed: [], removedDirs: 0 }`; `preview_selection_rename` → `preview_inbox_rename` mock'u gibi.
- [ ] **Step 1:** Dosyaları yaz. **Step 2:** `npm run build` ve `npm run audit:source` hatasız ("Unregistered invoke commands: None").

---

### Task 7: Seçim, tarih ve küçük resim hook'ları

**Files:** Create `src/hooks/useFileSelection.ts`, `src/hooks/useMediaDates.ts`, `src/hooks/useShellThumbs.ts`

**Interfaces:**
- Produces:
  - `useFileSelection(order: string[], resetKey: string) => { selected: Set<string>; isSelected(path): boolean; toggle(path: string, shift: boolean): void; setMany(paths: string[], on: boolean): void; clear(): void; hiddenCount: number }` — `resetKey` değişince temizlenir; `shift` aralığı `order` içinde son tıklanandan tıklanana, tıklananın yeni durumuyla; `hiddenCount` = seçili olup `order`'da olmayanlar.
  - `useMediaDates(paths: string[], key: string) => { taken: Map<string, number | null>; done: number; total: number; reading: boolean }` — 50'lik paketler, `key` değişince eski istek sonuçları yok sayılır.
  - `useShellThumbs(size: number) => { thumbOf(path: string): string | null | undefined; request(paths: string[]): void }` — `undefined` = henüz istenmedi/bekliyor, `null` = yok; 30'luk paketler, yinelenen istek yok; sonuç `convertFileSrc` ile döner.
- [ ] **Step 1:** Yaz. **Step 2:** `npm run build` hatasız. (Davranış doğrulaması Task 8'in tarayıcı adımlarında; adım 4 Review Focus #4'ü kapsar.)

---

### Task 8: Liste, süzgeçler, satırlar ve büyük ön izleme

**Files:**
- Create: `src/components/FileBrowserFilters.tsx`, `src/components/FileBrowserRow.tsx`, `src/components/BrowserPreview.tsx`, `src/styles/browser.css`
- Modify: `src/components/FileBrowser.tsx`, `src/App.tsx:120` (`onPreview` kaldırılır), `src/index.css` (import)

**Interfaces:**
- Consumes: Task 6–7 çıktıları, `useFolderBrowser`, `VirtualRows`, `usePagination`.
- Produces: `FileBrowserRow` props `{ file: FolderFile; source: "A" | "B"; selected: boolean; taken: number | null | undefined; thumb: string | null | undefined; onToggle(shift: boolean): void; onPreview(): void; onReveal(): void }`; `BrowserPreview` props `{ items: FolderFile[]; index: number | null; onIndex(i: number | null): void; isSelected(path): boolean; onToggle(path): void; bigThumb(path): string | null | undefined }`.

Davranış (simülasyondaki gibi): arama, yıl/ay (okuma bitene kadar `disabled` + "Çekim tarihleri okunuyor… {done}/{total}. Yıl ve ay süzgeci bitince açılır."), sıra (Tarih yeni→eski varsayılan, Dosya adı, Boyut ↓), tür çipleri sayılarla; satır: onay kutusu, 72×54 küçük resim (`undefined` → iskelet, `null` → `lucide` `FileText`/`Film`/`Image`/`File` ikonu), ad, alt satır tür, tarih (`Camera` simgesi + "çekim" ya da `PenLine` + "değişiklik"), boyut, klasörde göster; satırın boş yerine tıklama seçer; başlık kutusu görünenleri seçer (yarı seçili durum dahil). `BrowserPreview`: resimde `convertFileSrc(path)`, diğerlerinde 768 px Windows ön izlemesi; `←`/`→` gezinir, `Boşluk` seçer, `Esc` kapatır.

- [ ] **Step 1:** Yaz. **Step 2:** `npm run build` + `npm run audit:source` hatasız.
- [ ] **Step 3: Tarayıcı doğrulaması** (`preview_start` → `vite`, Harita > Klasördeki dosyalar, Listele açık): (1) satırlar ikon/iskeletle görünür; (2) okuma sürerken yıl/ay kapalı, bitince açık ve `.jpg` satırlarında "çekim"; (3) `Shift` ile aralık seçimi ve başlık kutusu çalışır; (4) okuma sürerken grafikten alt klasöre gir → yeni listede eski klasörün tarihleri görünmez, sayaç yeni klasörün toplamıyla başlar; (5) ön izlemede `→` ve `Boşluk` seçer; konsolda hata yok.

---

### Task 9: Seçim çubuğu, hedef seçici ve taşıma onayı

**Files:**
- Create: `src/hooks/useFolderTargets.ts`, `src/hooks/useSelectionMove.ts`, `src/components/SelectionBar.tsx`, `src/components/TargetFolderSelect.tsx`, `src/components/SelectionMoveDialog.tsx`
- Modify: `src/components/FileBrowser.tsx`

**Interfaces:**
- Produces:
  - `useFolderTargets(app: OrganizerState) => { folders: string[]; truncated: boolean; reload(): void }` (A ve varsa B için `listSubfolders`, `app.excludeText`'ten hariçler).
  - `useSelectionMove(app: OrganizerState) => { busy: boolean; preview(req: SelectionMoveRequest): Promise<SelectionMovePlan>; apply(plan: SelectionMovePlan): Promise<MoveOutcome> }` — `apply` sonrası `app.refreshHistory()` + `app.bumpStatsToken()`; hata `app.setStatus({ type: "error", text })`.
  - `TargetFolderSelect` props `{ value: string | null; current: string; roots: { label: string; path: string }[]; folders: string[]; external: string[]; scopeAll: boolean; onChange(v: string | null): void; onPickExternal(path: string): void }` — "Bu klasör · <ad>" (`scopeAll`'da yerine "Klasör A kökü"/"Klasör B kökü", varsayılan seçimsiz), kök içi klasörler (`Faturalar\Nisan` biçiminde), "Diğer konumlar", "Başka bir klasör seç…" (`open({ directory: true })`).

Davranış: çubuk ≥1 seçimde görünür; "N dosya seçili · boyut · K tanesi şu an süzgeç dışında, onlar da taşınır"; mini küçük resimler (8 + "+N"); öneri çipi; hedef + grup adı + `resolveGroupTarget` yol satırı ("var"/"yeni" rozetleri, "doğrudan bu klasöre"); düğmeler "Seçimi temizle", "Adlandır…", "N dosyayı taşı…" (geçersizse/`app.isScanning`/`app.isMoving`/`busy` iken kapalı). Onay penceresi `previewSelectionMove` sonucunu gösterir: hedef yol, "Var olan klasör"/"Yeni … klasörü oluşturulur", numaralanan dosyalar `→ yeni ad`; "N dosyayı taşı". Sonrası: seçim ve grup adı temizlenir, hedef korunur, bildirim "N dosya “<klasör>” klasörüne taşındı." + "Klasörü aç" (kök içindeyse `browser.enter`) + "Geri al" (`undoLastMove`).

- [ ] **Step 1:** Yaz. **Step 2:** `npm run build` + `npm run audit:source`.
- [ ] **Step 3: Tarayıcı doğrulaması:** (1) Hedef "Faturalar", grup "Haziran" → yol satırı `…\Faturalar` (var) `\Haziran` (yeni); onayda "Yeni “Haziran” klasörü oluşturulur."; (2) taşımadan sonra hedef hâlâ "Faturalar"; (3) grup `a\\b` → "Klasör adları arasında boş bölüm var." ve düğme kapalı; (4) Tümü kapsamında hedef seçilmeden düğme kapalı; (5) konsolda hata yok. Ekran görüntüsü al.

---

### Task 10: Ortak adlandırma penceresi ve "Adlandır, sonra taşı"

**Files:**
- Create: `src/components/TemplateRenameDialog.tsx`
- Modify: `src/components/InboxRenameDialog.tsx` (sarmalayıcı), `src/components/SelectionBar.tsx` / `FileBrowser.tsx`

**Interfaces:**
- Produces: `TemplateRenameDialog` props `{ open: boolean; onOpenChange(o: boolean): void; count: number; busy: boolean; tokens: string[]; hint?: ReactNode; preview(o: ListRenameOptions): Promise<RenamePlan>; apply(plan: RenamePlan): Promise<void>; secondary?: { label: string; run(plan: RenamePlan): Promise<void> } }`. Gelen kutusu: mevcut başlık/açıklama/şablon varsayılanı `{OzelMetin}_{Grup}_{tarih}_{sıra}` korunur, `secondary` yok. Bu ekran: varsayılan şablon `{Grup}_{tarih}_{sıra}`, pad 2, `{Grup}` = `resolveGroupTarget` hedefinin son klasör adı, `dates` = `useMediaDates` çekim tarihleri (yoksa `null`), `tidy: true`, `secondary = { label: "Adlandır, sonra taşı…", run }` (grup/hedef geçersizse düğme kapalı). Uygulamadan sonra seçili yollar `plan.entries` ile yeni yollara eşlenir (`useFileSelection.setMany`).

- [ ] **Step 1:** Yaz. **Step 2:** `npm run build` + `npm run audit:source`.
- [ ] **Step 3: Tarayıcı doğrulaması:** (1) Bugün > Taşınacakları adlandır penceresi öncekiyle aynı alanlar ve çalışma; (2) bu ekranda "Adlandır…" → ön izleme listesi, "Adlandır, sonra taşı…" → taşıma onayı açılır; (3) adlandırmadan sonra seçim korunur.

---

### Task 11: Bütün doğrulama ve gerçek uygulamada deneme

- [ ] **Step 1:** `npm run check` → build, audit ve tüm Rust testleri PASS (çıktıyı not et).
- [ ] **Step 2:** `npm run tauri dev` ile gerçek uygulamayı aç (Kaspersky derleme kilidi olursa kullanıcıdan korumayı duraklatmasını iste). Gerçek bir test klasöründe (telefondan kopyalanmış JPEG, MP4, PDF, DOCX, ZIP): küçük resimlerin/ikonların göründüğünü, fotoğrafın çekim tarihinin Gezgin'deki ile aynı olduğunu, Haziran süzgecini, `Faturalar\Mayıs` taşımasını, Geçmiş'te "Seçilenleri taşı · Mayıs" kaydını ve geri almayı dene.
- [ ] **Step 3:** Spec'e göre eksik kalan madde varsa listele; kullanıcıya sonucu, doğrulanmayan noktalarla birlikte bildir.

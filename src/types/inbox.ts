/** Baslangic karari onerisi (Rust `inbox_suggest.rs`). */
export interface Suggestion {
  verdict: "keep" | "move" | "bin";
  /** Tasinacaksa hedef klasorun adi. */
  target: string | null;
  reason: string;
}

/** Gelen kutusu (Rust `inspector.rs` / `inbox_list.rs` ile ayni sekil). Zamanlar ms. */
export interface FileDetail {
  path: string;
  name: string;
  ext: string;
  size: number;
  group: string;
  arrivedMs: number | null;
  createdMs: number | null;
  modifiedMs: number | null;
  /** Windows'ta cogu zaman guncellenmez; "yaklasik" gosterilir. */
  accessedMs: number | null;
  /** Metin dosyalarinda ilk 300 karakter. */
  contentPreview: string | null;
  thumbnailPath: string | null;
  similarTo: string | null;
  similarity: number | null;
  /** Icerigi birebir ayni baska dosyanin yolu. */
  duplicateOf: string | null;
  /** O dosya yedek klasorundeyse klasor adi; gelen kutusundaysa null. */
  duplicateIn: string | null;
  suggestion: Suggestion | null;
}

export interface InboxGroup {
  name: string;
  bytes: number;
  files: FileDetail[];
}

export interface InboxDay {
  date: string;
  files: number;
}

export interface InboxListing {
  date: string;
  dir: string;
  exists: boolean;
  days: InboxDay[];
  groups: InboxGroup[];
  totalFiles: number;
  totalBytes: number;
  truncated: boolean;
}

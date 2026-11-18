import type { FileCategory, FileItem } from "../../types/file";

/** Oneri uretiminde dikkate alinmayan (gurultu) tokenlar. */
const NOISE = new Set([
  "v1", "v2", "v3", "v4", "final", "son", "yeni", "eski", "kopya", "copy",
  "yedek", "backup", "draft", "taslak", "img", "image", "doc", "new", "old",
  "the", "file", "resim", "belge", "fotograf",
]);

const MIN_TOKEN_LEN = 3;
const MAX_TOKEN_LEN = 24;

const CATEGORY_FALLBACK: Record<FileCategory, string> = {
  Document: "Doküman",
  Image: "Resim",
  Archive: "Arşiv",
  Other: "Diğer",
};

function capitalize(s: string): string {
  return s.charAt(0).toLocaleUpperCase("tr-TR") + s.slice(1);
}

/**
 * Dosya adlarindan (stem) otomatik etiket adi onerir.
 * Ornek: "fatura_ocak.pdf", "fatura_v2.pdf" -> "2026_Fatura"
 */
export function suggestTagName(files: FileItem[]): string {
  if (files.length === 0) return "Etiket";

  const freq = new Map<string, number>();
  const yearFreq = new Map<string, number>();

  for (const file of files) {
    const tokens = file.stem
      .toLocaleLowerCase("tr-TR")
      .split(/[^a-z0-9çğıöşü]+/)
      .filter(Boolean);

    for (const token of tokens) {
      if (/^(19|20)\d{2}$/.test(token)) {
        yearFreq.set(token, (yearFreq.get(token) ?? 0) + 1);
        continue;
      }
      if (
        token.length < MIN_TOKEN_LEN ||
        token.length > MAX_TOKEN_LEN ||
        NOISE.has(token) ||
        /^\d+$/.test(token)
      ) {
        continue;
      }
      freq.set(token, (freq.get(token) ?? 0) + 1);
    }
  }

  // En sik gecen yil (varsa).
  let year: string | null = null;
  for (const [y, c] of yearFreq) {
    if (!year || c > (yearFreq.get(year) ?? 0)) year = y;
  }

  // En sik ve en uzun anlamlı token.
  let top: string | null = null;
  let topCount = 0;
  for (const [token, count] of freq) {
    if (
      count > topCount ||
      (count === topCount && top !== null && token.length > top.length)
    ) {
      top = token;
      topCount = count;
    }
  }

  const base = top
    ? capitalize(top)
    : capitalize(CATEGORY_FALLBACK[files[0].category]);
  return year ? `${year}_${base}` : base;
}

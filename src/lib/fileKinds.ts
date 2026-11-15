/** Klasordeki dosyalar ekrani: uzantiya gore tur cipleri ve etkin tarih. */

export type KindChip = "all" | "image" | "video" | "pdf" | "doc" | "other";
export type FileKind = Exclude<KindChip, "all">;

export const KIND_LABELS: Record<KindChip, string> = {
  all: "Tümü", image: "Resim", video: "Video", pdf: "PDF", doc: "Doküman", other: "Diğer",
};
export const KIND_ORDER: KindChip[] = ["all", "image", "video", "pdf", "doc", "other"];

const EXTS: [FileKind, string[]][] = [
  ["image", ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tif", "tiff", "heic", "heif", "dng", "cr2", "nef", "arw", "svg"]],
  ["video", ["mp4", "mkv", "avi", "mov", "wmv", "m4v", "webm", "3gp", "mts"]],
  ["pdf", ["pdf"]],
  ["doc", ["doc", "docx", "odt", "rtf", "txt", "md", "xls", "xlsx", "ods", "csv", "ppt", "pptx", "odp"]],
];
const KIND_BY_EXT = new Map(EXTS.flatMap(([kind, exts]) => exts.map(e => [e, kind] as const)));

export const extOf = (name: string) => {
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
};

export const kindOf = (name: string): FileKind => KIND_BY_EXT.get(extOf(name)) ?? "other";

/** Cekim tarihi varsa o, yoksa degisiklik tarihi (unix sn). */
export const effectiveDate = (modifiedAt: number | null, takenAt: number | null | undefined): number | null =>
  takenAt ?? modifiedAt;

const pad = (n: number) => String(n).padStart(2, "0");

/** Yerel saatle yil, ay, gun ("2024", "06", "14"). */
export function ymd(seconds: number): { y: string; m: string; d: string } {
  const t = new Date(seconds * 1000);
  return { y: String(t.getFullYear()), m: pad(t.getMonth() + 1), d: pad(t.getDate()) };
}

export const MONTHS = ["Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"];

export const formatDay = (seconds: number) => { const { y, m, d } = ymd(seconds); return `${d}.${m}.${y}`; };

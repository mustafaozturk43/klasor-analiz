import type { FileCategory, GroupType } from "../types/file";

/** Kategori rozet/sekme etiketleri. */
export const CATEGORY_LABELS: Record<FileCategory, string> = {
  Document: "Doküman",
  Image: "Resim",
  Archive: "Arşiv",
  Other: "Diğer",
};

/** Grup tipi rozet etiketleri. */
export const GROUP_TYPE_LABELS: Record<GroupType, string> = {
  Exact: "Birebir Kopya",
  Similar: "İsim Benzeri",
};

/** Byte degerini okunabilir MB/GB metnine cevirir (tr-TR bicimi). */
export function formatBytes(bytes: number): string {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const exp = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1,
  );
  const value = bytes / 1024 ** exp;
  const text =
    exp === 0
      ? String(Math.round(value))
      : value < 10
        ? value.toFixed(2)
        : value.toFixed(1);
  return `${text.replace(".", ",")} ${units[exp]}`;
}

/** Sayiyi tr-TR biciminde gruplar (1.234 gibi). */
export function formatCount(n: number): string {
  return n.toLocaleString("tr-TR");
}

/** Unix saniyeyi tr-TR tarihine cevirir (gg.aa.yyyy). */
export function formatDate(seconds: number): string {
  return seconds ? new Date(seconds * 1000).toLocaleDateString("tr-TR") : "—";
}

/** "3 yıl önce", "5 ay önce", "12 gün önce" gibi kaba bir yas. */
export function formatAge(seconds: number, nowMs = Date.now()): string {
  if (!seconds) return "tarih yok";
  const days = Math.max(0, Math.floor((nowMs / 1000 - seconds) / 86400));
  if (days >= 365) return `${Math.floor(days / 365)} yıl önce`;
  if (days >= 30) return `${Math.floor(days / 30)} ay önce`;
  return days ? `${days} gün önce` : "bugün";
}

/** Verilen Windows/Unix yolunun ust klasorunu dondurur; kok dizindeyse null. */
export function parentPath(path: string): string | null {
  const trimmed = path.replace(/[\\/]+$/, "");
  const idx = Math.max(trimmed.lastIndexOf("\\"), trimmed.lastIndexOf("/"));
  if (idx < 0) return null;
  const parent = trimmed.slice(0, idx + 1);
  return parent.length > 2 ? parent : null;
}

/** Yolun son parcasini (klasor/dosya adi) dondurur. */
export function baseName(path: string): string {
  const trimmed = path.replace(/[\\/]+$/, "");
  const idx = Math.max(trimmed.lastIndexOf("\\"), trimmed.lastIndexOf("/"));
  return idx >= 0 ? trimmed.slice(idx + 1) : trimmed;
}

/** Tarama ozetine eklenen "okunamayan dosya" notu (yoksa bos). */
export const skippedNote = (warnings?: string[]): string =>
  warnings?.length ? ` ${formatCount(warnings.length)} dosya/klasör atlandı (ilk: ${warnings[0]}).` : "";

/** WebView'in dogrudan gosterebildigi gorsel uzantilari (heic gosterilemez; `thumbnail.rs` ile ayni liste). */
export const PREVIEW_IMAGE_EXTS = new Set(["jpg", "jpeg", "png", "webp", "gif", "bmp", "tif", "tiff"]);

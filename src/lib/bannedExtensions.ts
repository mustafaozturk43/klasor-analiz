/**
 * Yasakli uzanti listesi: varsayilanlar, normalizasyon ve kalici tercih (`prefs`).
 * Rust tarafindaki DEFAULT_BANNED_EXTS ile ayni varsayilanlar.
 */
import { getPref, setPref } from "./prefs";

export const DEFAULT_BANNED_EXTENSIONS = [
  "exe",
  "com",
  "dll",
  "sys",
  "msi",
  "bat",
  "cmd",
  "scr",
  "ps1",
  "vbs",
  "lnk",
  "reg",
] as const;

const KEY = "sfo-banned-ext";

/** Uzantiyi normalize eder: kucuk harf, bastaki nokta ve bosluk atilir. */
export function normalizeExt(raw: string): string {
  return raw.trim().replace(/^\.+/, "").toLowerCase();
}

/** Kayitli listeyi okur; yoksa varsayilanlarin kopyasini doner. */
export function loadBannedExtensions(): string[] {
  const parsed = getPref<unknown>(KEY);
  if (!Array.isArray(parsed)) return [...DEFAULT_BANNED_EXTENSIONS];
  return parsed
    .filter((p): p is string => typeof p === "string")
    .map(normalizeExt)
    .filter(Boolean);
}

export function saveBannedExtensions(list: string[]): void {
  setPref(KEY, list);
}

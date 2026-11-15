/**
 * Son taranan klasorler (kalici tercih, en fazla 8 kayit).
 * "sfo-recent" anahtariyla saklanir; en yeni klasor listenin basinda.
 */
import { getPref, setPref } from "./prefs";

const KEY = "sfo-recent";
const MAX_RECENT = 8;

export function loadRecent(): string[] {
  const parsed = getPref<unknown>(KEY);
  return Array.isArray(parsed) ? parsed.filter((p): p is string => typeof p === "string") : [];
}

/** Klasoru listenin basina ekler; tekrar varsa once siler, fazlasini kirpar. */
export function pushRecent(path: string): void {
  if (!path) return;
  const list = loadRecent().filter((p) => p !== path);
  list.unshift(path);
  save(list.slice(0, MAX_RECENT));
}

/** Kaydi listeden cikarir. */
export function removeRecent(path: string): void {
  save(loadRecent().filter((p) => p !== path));
}

function save(list: string[]): void {
  setPref(KEY, list);
}

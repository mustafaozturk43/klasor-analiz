/**
 * Arayuz tercihleri (tema, son klasorler, yasakli uzantilar, duzenleme profilleri).
 * Tek kaynak Rust tarafindaki `settings.json` (`uiPrefs`). Uygulama acilirken bir kez
 * yuklenir (`loadPrefs`), sonra bellekten eszamanli okunur; her degisiklik diske yazilir.
 */
import { invoke } from "@tauri-apps/api/core";

/** Tema ilk boyamadan once `index.html` tarafindan okunur; localStorage yalnizca bu onbellek icindir. */
export const THEME_KEY = "sfo-theme";
const LEGACY_KEYS = ["sfo-banned-ext", "sfo-organize-current", "sfo-organize-profiles", "sfo-recent", THEME_KEY];
let cache: Record<string, unknown> = {};

const store = (key: string, value: unknown) => invoke("set_ui_pref", { key, value });

/** React baslamadan once cagrilir. Eski localStorage kayitlarini bir kez `settings.json`a aktarir. */
export async function loadPrefs(): Promise<void> {
  try {
    cache = (await invoke<{ uiPrefs?: Record<string, unknown> }>("get_settings"))?.uiPrefs ?? {};
  } catch {
    cache = {};
  }
  for (const key of LEGACY_KEYS) {
    try {
      const raw = localStorage.getItem(key);
      if (key in cache || raw === null) continue;
      const value: unknown = key === THEME_KEY ? raw : JSON.parse(raw);
      await store(key, value);
      cache[key] = value;
      if (key !== THEME_KEY) localStorage.removeItem(key);
    } catch {
      // Bozuk eski kayit ya da yazma hatasi: eski deger yerinde kalir, sonraki acilista yeniden denenir.
    }
  }
}

export const getPref = <T>(key: string): T | null => (cache[key] as T | undefined) ?? null;

export function setPref(key: string, value: unknown): void {
  cache[key] = value;
  void store(key, value).catch(() => undefined); // Yazilamazsa tercih bu oturumla sinirli kalir.
}

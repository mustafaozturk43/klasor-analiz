/** Ayarlar komutlari: Rust tarafindaki settings.json okunur ve yazilir. */
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, ExtGroup } from "../types/settings";

export const getSettings = () => invoke<AppSettings>("get_settings");

/** Dogrulanip kaydedilen (temizlenmis) ayarlari dondurur; gecersizse hata metni firlatir. */
export const saveSettings = (settings: AppSettings) => invoke<AppSettings>("save_settings", { settings });

export const defaultExtGroups = () => invoke<ExtGroup[]>("default_ext_groups");

/** Uzanti girdisini ("PDF, .docx txt") ayri, kucuk harfli uzantilara boler. */
export function splitExtensions(text: string): string[] {
  return text.split(/[\s,;]+/).map(e => e.trim().replace(/^\.+/, "").toLowerCase()).filter(Boolean);
}

/** Yerel tarih "YYYY-MM-DD" (gelen kutusu gun klasoru bicimi). */
export function localDate(now = new Date()): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

/** Bugunun gelen kutusu klasoru (yerel tarih). Watcher ayni bicimi kullanir. */
export function todayInboxPath(backupRoot: string, now = new Date()): string {
  const day = localDate(now);
  const sep = backupRoot.includes("/") && !backupRoot.includes("\\") ? "/" : "\\";
  return `${backupRoot.replace(/[\\/]+$/, "")}${sep}_GelenKutusu${sep}${day}_Bakilacaklar`;
}

/** Ayni uzanti birden fazla gruptaysa uzanti -> ilk grup adi; cakisanlar listesi. */
export function extensionConflicts(groups: ExtGroup[]): Map<string, string> {
  const owner = new Map<string, string>();
  const clash = new Map<string, string>();
  for (const g of groups) {
    for (const e of g.extensions) {
      const prev = owner.get(e);
      if (prev && prev !== g.name) clash.set(`${g.name}:${e}`, prev);
      else owner.set(e, g.name);
    }
  }
  return clash;
}

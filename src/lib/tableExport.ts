/** Tablo raporlari: ekranlar basliklari ve satirlari hazirlar, Rust `export_table_csv` Excel uyumlu CSV yazar. */
import { invoke } from "@tauri-apps/api/core";
import { pickCsvTarget } from "./commands";

export type TableRows = string[][];

/** Kaydet diyalogu acar; iptal edilirse null. Yazilan veri satiri sayisi ve dosya yolu doner. */
export async function exportTableCsv(defaultName: string, header: string[], rows: TableRows): Promise<{ rows: number; path: string } | null> {
  const path = await pickCsvTarget(defaultName);
  if (!path) return null;
  const written = await invoke<number>("export_table_csv", { filePath: path, header, rows });
  return { rows: written, path };
}

/** Unix saniyeyi "gg.aa.yyyy ss:dd" yapar; 0 bos metin. */
export function dateText(seconds: number): string {
  if (!seconds) return "";
  const d = new Date(seconds * 1000);
  const p = (n: number) => String(n).padStart(2, "0");
  return `${p(d.getDate())}.${p(d.getMonth() + 1)}.${d.getFullYear()} ${p(d.getHours())}:${p(d.getMinutes())}`;
}

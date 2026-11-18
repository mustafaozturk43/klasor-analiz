/** Klasor sagligi raporunu tablo satirlarina cevirir (CSV disa aktarma). */
import { baseName } from "../../lib/format";
import { dateText, type TableRows } from "../../lib/tableExport";
import type { HealthReport } from "../../types/health";
import { relative } from "../rules/organizeView";

export const HEALTH_HEADER = ["Bölüm", "Ad", "Klasör içi yol", "Tam yol", "Boyut (bayt)", "Değişiklik tarihi"];

/** Dort bolumun hepsi tek tabloda, "Bölüm" sutunuyla ayrilir. */
export function healthRows(r: HealthReport): TableRows {
  const files = (section: string, list: HealthReport["largest"]): TableRows =>
    list.map(f => [section, baseName(f.path), relative(r.rootPath, f.path), f.path, String(f.size), dateText(f.mtime)]);
  return [
    ...files("En büyük dosyalar", r.largest),
    ...files("En uzun süredir değişmeyen", r.oldest),
    ...files("0 baytlık dosyalar", r.emptyFiles),
    ...r.emptyDirs.map(p => ["Boş klasörler", baseName(p), relative(r.rootPath, p), p, "", ""]),
  ];
}

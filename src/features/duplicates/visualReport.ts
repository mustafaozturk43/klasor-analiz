/** Benzer gorsel gruplarini tablo satirlarina cevirir (CSV disa aktarma). */
import { baseName } from "../../lib/format";
import type { TableRows } from "../../lib/tableExport";
import { similarityPct, type VisualReport } from "./visualCommands";

export const VISUAL_HEADER = ["Grup", "Önerilen", "Ad", "Tam yol", "Genişlik", "Yükseklik", "Boyut (bayt)", "Benzerlik %"];

/** Her grubun ilk dosyasi "Evet" (varsayilan olarak tutulan), digerleri "Hayır". */
export function visualRows(r: VisualReport): TableRows {
  return r.groups.flatMap(g => g.files.map((f, i) => [g.id, i === 0 ? "Evet" : "Hayır", baseName(f.path), f.path, String(f.width), String(f.height), String(f.size), String(similarityPct(f.distance))]));
}

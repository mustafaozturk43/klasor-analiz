/** Klasordeki dosyalar: arama, tur, yil/ay suzgeci ve siralama (saf). */
import type { FolderFile } from "../types/file";
import { effectiveDate, kindOf, KIND_ORDER, ymd, type KindChip } from "./fileKinds";

export type BrowserSort = "date" | "name" | "size";
export interface BrowserFilters { q: string; year: string; month: string; sort: BrowserSort; kind: KindChip }
export const DEFAULT_FILTERS: BrowserFilters = { q: "", year: "all", month: "all", sort: "date", kind: "all" };
export interface BrowserEntry { file: FolderFile; source: "A" | "B" }

export interface BrowserView {
  items: BrowserEntry[];
  counts: Record<KindChip, number>;
  years: string[];
  months: string[];
}

/** `taken`: okunmus cekim tarihleri; `datesReady` degilse yil/ay suzgeci uygulanmaz. */
export function viewFiles(entries: BrowserEntry[], taken: Map<string, number | null>, f: BrowserFilters, datesReady: boolean): BrowserView {
  const dateOf = (e: BrowserEntry) => effectiveDate(e.file.modifiedAt, taken.get(e.file.path));
  const counts = Object.fromEntries(KIND_ORDER.map(k => [k, 0])) as Record<KindChip, number>;
  entries.forEach(e => { counts.all++; counts[kindOf(e.file.name)]++; });
  const parts = entries.map(dateOf).filter((d): d is number => d !== null).map(ymd);
  const years = [...new Set(parts.map(p => p.y))].sort().reverse();
  const months = [...new Set(parts.filter(p => f.year === "all" || p.y === f.year).map(p => p.m))].sort();
  const q = f.q.trim().toLocaleLowerCase("tr");
  const items = entries.filter(e => {
    if (q && !e.file.name.toLocaleLowerCase("tr").includes(q)) return false;
    if (f.kind !== "all" && kindOf(e.file.name) !== f.kind) return false;
    if (!datesReady || (f.year === "all" && f.month === "all")) return true;
    const d = dateOf(e);
    if (d === null) return false;
    const p = ymd(d);
    return (f.year === "all" || p.y === f.year) && (f.month === "all" || p.m === f.month);
  });
  const byName = (a: BrowserEntry, b: BrowserEntry) => a.file.name.localeCompare(b.file.name, "tr");
  items.sort(f.sort === "size" ? (a, b) => b.file.size - a.file.size
    : f.sort === "name" ? byName
    : (a, b) => (dateOf(b) ?? -1) - (dateOf(a) ?? -1) || byName(a, b));
  return { items, counts, years, months };
}

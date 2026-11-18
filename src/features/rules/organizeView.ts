/** "Türüne Göre Düzenle" ekranlari icin saf yardimcilar (hesap, filtre, metin). */
import type { KindCount, OrganizeEntry, OrganizePlan } from "../../types/organize";

export type IssueFilter = "attention" | "renamed" | "quarantine" | "similar" | "skip" | "all";

export const ISSUE_FILTERS: { id: IssueFilter; label: string }[] = [
  { id: "attention", label: "Dikkat gerektirenler" },
  { id: "renamed", label: "Numaralananlar" },
  { id: "quarantine", label: "Birebir kopyalar" },
  { id: "similar", label: "Benzer adlar" },
  { id: "skip", label: "Atlananlar" },
  { id: "all", label: "Tüm dosyalar" },
];

export function matchesIssue(e: OrganizeEntry, filter: IssueFilter): boolean {
  switch (filter) {
    case "renamed": return e.renamed;
    case "quarantine": return e.action === "Quarantine";
    case "similar": return !!e.similarTo;
    case "skip": return e.action === "Skip";
    case "all": return true;
    default: return e.renamed || e.action === "Quarantine" || e.action === "Skip" || !!e.similarTo;
  }
}

export interface PlanCounts { move: number; inPlace: number; quarantine: number; renamed: number; skip: number }

export function countPlan(plan: OrganizePlan): PlanCounts {
  const c: PlanCounts = { move: 0, inPlace: 0, quarantine: 0, renamed: 0, skip: 0 };
  for (const e of plan.entries) {
    if (e.action === "Move") c.move++;
    else if (e.action === "InPlace") c.inPlace++;
    else if (e.action === "Quarantine") c.quarantine++;
    else c.skip++;
    if (e.renamed) c.renamed++;
  }
  return c;
}

/** Kok klasore gore goreli yol (kok disindaysa tam yol). */
export function relative(root: string, path: string): string {
  const r = root.replace(/[\\/]+$/, "");
  return path.toLowerCase().startsWith(r.toLowerCase()) ? path.slice(r.length).replace(/^[\\/]+/, "") || "." : path;
}

function parentOf(rel: string): string {
  const i = Math.max(rel.lastIndexOf("\\"), rel.lastIndexOf("/"));
  return i < 0 ? "" : rel.slice(0, i);
}

/** Tasinacak dosyalarin bugun bulundugu klasorler (en kalabalik ilk). */
export function sourceFolders(plan: OrganizePlan): { folder: string; count: number }[] {
  const map = new Map<string, number>();
  for (const e of plan.entries) {
    if (e.action !== "Move" && e.action !== "Quarantine") continue;
    const folder = parentOf(relative(plan.rootPath, e.from)) || "(ana klasör)";
    map.set(folder, (map.get(folder) ?? 0) + 1);
  }
  return [...map].map(([folder, count]) => ({ folder, count })).sort((a, b) => b.count - a.count);
}

/** Tur sayimlarini kategori klasorune gore gruplar. */
export function byCategory(kinds: KindCount[]): { category: string; total: number; kinds: KindCount[] }[] {
  const map = new Map<string, KindCount[]>();
  for (const k of kinds) map.set(k.category, [...(map.get(k.category) ?? []), k]);
  return [...map].map(([category, list]) => ({ category, total: list.reduce((s, k) => s + k.count, 0), kinds: list }));
}

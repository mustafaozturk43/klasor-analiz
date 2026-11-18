/** Kural motoru icin saf yardimcilar: yeni kural, ozet metni, eylem etiketleri. */
import type { Rule, RuleAction } from "./ruleEngineCommands";

export const ACTION_LABELS: Record<RuleAction, string> = {
  move: "Klasöre taşı",
  moveYearMonth: "Yıl\\Ay klasörüne taşı",
  trash: "Geri Dönüşüm Kutusu'na gönder",
};

export const newRule = (n: number): Rule => ({
  name: `Kural ${n}`, enabled: true, exts: [], nameContains: "", minMb: null, maxMb: null, minAgeDays: null, action: "move", target: "",
});

/** "pdf, docx · adı “fatura” · ≥ 1 MB · 30+ gün" gibi koşul özeti; koşul yoksa boş. */
export function describeConditions(r: Rule): string {
  const parts: string[] = [];
  if (r.exts.length) parts.push(r.exts.join(", "));
  if (r.nameContains.trim()) parts.push(`adı “${r.nameContains.trim()}” içerir`);
  if (r.minMb !== null && r.maxMb !== null) parts.push(`${r.minMb}–${r.maxMb} MB`);
  else if (r.minMb !== null) parts.push(`≥ ${r.minMb} MB`);
  else if (r.maxMb !== null) parts.push(`≤ ${r.maxMb} MB`);
  if (r.minAgeDays !== null) parts.push(`${r.minAgeDays}+ gün eski`);
  return parts.join(" · ");
}

/** Bos ya da gecersiz metin null; aksi halde en az 0 olan sayi. */
export function parseOptionalNumber(text: string): number | null {
  const n = Number(text.replace(",", "."));
  return text.trim() === "" || !Number.isFinite(n) ? null : Math.max(0, n);
}

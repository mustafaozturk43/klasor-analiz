/**
 * Yalnizca gelistirme: `preview_name_rules` icin ornek agac ve kurallarin sade bir taklidi.
 * Gercek kurallar Rust tarafindadir (`name_rules.rs`); bu dosya yalnizca arayuzu tarayicida gostermek icindir.
 */
import type { NameEntry, NameRuleOptions } from "../features/rename/namingCommands";

const TREE: [string, boolean][] = [
  ["FATURALAR 2026", true], ["FATURALAR 2026\\fatura ocak.pdf", false], ["FATURALAR 2026\\Fatura Şubat.PDF", false],
  ["müşteri sözleşmeleri", true], ["müşteri sözleşmeleri\\acme sözleşme son (2).docx", false], ["müşteri sözleşmeleri\\ACME sözleşme son.docx", false],
  ["toplantı notları", true], ["toplantı notları\\Haftalık özet.txt", false], ["proje klasörü", true], ["yeni klasör", true],
  ["IŞIK ölçüm sonuçları.csv", false], ["Okuma Listesi.txt", false],
];
const TR: Record<string, string> = { ç: "c", Ç: "C", ğ: "g", Ğ: "G", ı: "i", İ: "I", ö: "o", Ö: "O", ş: "s", Ş: "S", ü: "u", Ü: "U" };

function rename(stem: string, o: NameRuleOptions, seq: number | null): string {
  let s = stem.trim().replace(/\s+/g, " ");
  if (o.clean) s = s.replace(/[()[\]{}!,;'’`~@#$%^&+=]/g, " ").replace(/\s+/g, " ").trim();
  if (o.ascii) s = s.replace(/[çÇğĞıİöÖşŞüÜ]/g, c => TR[c]);
  const lo = (x: string) => o.ascii ? x.toLowerCase() : x.toLocaleLowerCase("tr");
  const up = (x: string) => o.ascii ? x.toUpperCase() : x.toLocaleUpperCase("tr");
  if (o.case === "lower") s = lo(s);
  else if (o.case === "upper") s = up(s);
  else if (o.case === "sentence") s = up(s.slice(0, 1)) + lo(s.slice(1));
  else if (o.case === "title") s = s.split(/([ \-_.])/).map(w => up(w.slice(0, 1)) + lo(w.slice(1))).join("");
  const sep = o.space === "underscore" ? "_" : o.space === "keep" ? " " : "-";
  if (o.space !== "keep") s = s.replace(/ /g, o.space === "none" ? "" : sep);
  const prefix = seq === null ? "" : String(seq).padStart(o.pad, "0") + sep;
  const suffix = o.date === "my" ? `${sep}11-2026` : o.date === "ym" ? `${sep}2026-11` : "";
  if (o.maxLen > 0) s = s.slice(0, Math.max(1, o.maxLen - prefix.length - suffix.length));
  return prefix + s.replace(/^[-_ .]+|[-_ .]+$/g, "") + suffix;
}

export function mockNamePlan(root: string, o: NameRuleOptions) {
  const counters = new Map<string, number>();
  const entries: NameEntry[] = [];
  for (const [rel, isDir] of TREE) {
    const cut = rel.lastIndexOf("\\"), parent = cut < 0 ? "" : rel.slice(0, cut), name = rel.slice(cut + 1);
    if (parent && !o.includeSubdirs) continue;
    const from = `${root}\\${rel}`;
    if (name === "proje klasörü") { entries.push({ from, to: null, warning: "Proje klasörü; içindekilerle birlikte korunur.", isDir }); continue; }
    const inScope = o.scope === "both" || (o.scope === "dirs") === isDir;
    const key = `${parent}|${isDir}`, numbered = isDir ? o.seqDirs : o.seqFiles;
    const seq = inScope && numbered ? (counters.set(key, (counters.get(key) ?? o.start - 1) + 1), counters.get(key)!) : null;
    const dot = isDir ? -1 : name.lastIndexOf(".");
    const next = rename(dot > 0 ? name.slice(0, dot) : name, o, seq) + (dot > 0 ? name.slice(dot) : "");
    entries.push({ from, to: inScope && next !== name ? `${root}\\${parent ? `${parent}\\` : ""}${next}` : null, warning: null, isDir });
  }
  return { id: "mock-names", entries };
}

/**
 * "Hedef klasor + grup adi" yol cozumu (arka uctaki `selection_paths::split_group`
 * ile ayni kurallar). Saf fonksiyonlar: diske dokunmaz, var olan klasor listesini alir.
 */
import { MONTHS, ymd } from "../map/fileKinds";

export type GroupTarget =
  | { ok: true; base: string; target: string; parts: { name: string; exists: boolean }[]; created: string[] }
  | { ok: false; error: boolean; message: string };

const BAD = /[<>:"|?*]/;
const DEVICE = /^(con|prn|aux|nul|com[1-9]|lpt[1-9])$/i;
const lower = (s: string) => s.toLocaleLowerCase("tr");
export const joinPath = (base: string, name: string) => `${base.replace(/[\\/]+$/, "")}\\${name}`;
export const leafName = (path: string) => path.replace(/[\\/]+$/, "").split(/[\\/]/).pop() ?? path;

/** Grup adini parcalara ayirir; hata metni ya da parcalar. */
export function splitGroup(group: string): string[] | string {
  const text = group.trim();
  if (!text) return [];
  const out: string[] = [];
  for (const raw of text.split(/[\\/]/)) {
    const seg = raw.trim().replace(/\.+$/, "");
    if (!raw.trim()) return "Klasör adları arasında boş bölüm var.";
    if (!seg) return `“${raw.trim()}” klasör adı olarak kullanılamaz.`;
    if (BAD.test(seg)) return `“${seg}” klasör adında kullanılamayan karakter var (< > : " | ? *). Alt klasör için \\ kullanın.`;
    if (DEVICE.test(seg.split(".")[0])) return `“${seg}” Windows'ta ayrılmış bir ad.`;
    if (lower(seg) === lower("_Kopyalar_Karantina")) return "“_Kopyalar_Karantina” klasör adı olarak kullanılamaz.";
    out.push(seg);
  }
  return out;
}

/** `existing`: bilinen klasorlerin tam yollari (buyuk/kucuk harf duyarsiz eslesir). */
export function resolveGroupTarget(base: string, group: string, existing: string[], currentDir: string): GroupTarget {
  const segs = splitGroup(group);
  if (typeof segs === "string") return { ok: false, error: true, message: segs };
  const known = new Map(existing.map(p => [lower(p), p]));
  let cur = base;
  const parts: { name: string; exists: boolean }[] = [];
  const created: string[] = [];
  for (const seg of segs) {
    const want = joinPath(cur, seg);
    const found = created.length ? undefined : known.get(lower(want));
    cur = found ?? want;
    parts.push({ name: found ? leafName(found) : seg, exists: !!found });
    if (!found) created.push(cur);
  }
  if (lower(cur) === lower(currentDir)) {
    return { ok: false, error: false, message: segs.length ? "Dosyalar zaten bu klasörde." : "Grup adı yazın ya da bir hedef klasör seçin." };
  }
  return { ok: true, base, target: cur, parts, created };
}

/** Secilenlerin hepsi ayni aydansa ad onerisi: hedef seciliyse "Mayıs", degilse "Mayıs 2024". */
export function suggestGroupName(dates: (number | null)[], hasTarget: boolean): string | null {
  if (!dates.length || dates.some(d => d === null)) return null;
  const keys = new Set(dates.map(d => { const { y, m } = ymd(d as number); return `${y}-${m}`; }));
  if (keys.size !== 1) return null;
  const [y, m] = [...keys][0].split("-");
  const month = MONTHS[Number(m) - 1];
  return hasTarget ? month : `${month} ${y}`;
}

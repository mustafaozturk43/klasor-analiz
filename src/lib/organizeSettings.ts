/**
 * "Türüne Göre Düzenle" ayarlari: son kullanilan ayarlarin ve kayitli
 * profillerin kalici tercih (`prefs`, settings.json) olarak saklanmasi + kural dogrulamasi.
 * Dogrulama Rust'taki RuleSet ile aynidir; burada aninda geri bildirim icindir.
 */
import type { OrganizeProfile, OrganizeToggles, RuleDef } from "../types/organize";
import { normalizeExt } from "./bannedExtensions";
import { getPref, setPref } from "./prefs";

export const DEFAULT_TOGGLES: OrganizeToggles = {
  quarantineDuplicates: true, protectProjects: true, flat: false, byYear: false, removeEmptyDirs: false,
};

const CURRENT_KEY = "sfo-organize-current";
const PROFILES_KEY = "sfo-organize-profiles";
const BAD_CHARS = /[<>:"/\\|?*]/;

/** Kayitli ayarlar; `rules: null` varsayilan kurallar demektir. */
export interface StoredSettings { rules: RuleDef[] | null; toggles: OrganizeToggles }

const read = getPref;
const write = setPref;

const isRule = (r: unknown): r is RuleDef =>
  !!r && typeof r === "object" && typeof (r as RuleDef).category === "string"
  && typeof (r as RuleDef).kind === "string" && Array.isArray((r as RuleDef).extensions);

export function loadSettings(): StoredSettings {
  const s = read<Partial<StoredSettings>>(CURRENT_KEY);
  const rules = Array.isArray(s?.rules) ? s.rules.filter(isRule) : null;
  return { rules: rules?.length ? rules : null, toggles: { ...DEFAULT_TOGGLES, ...(s?.toggles ?? {}) } };
}

export const saveSettings = (s: StoredSettings) => write(CURRENT_KEY, s);

export function loadProfiles(): OrganizeProfile[] {
  const list = read<OrganizeProfile[]>(PROFILES_KEY);
  if (!Array.isArray(list)) return [];
  return list
    .filter(p => p && typeof p.name === "string" && Array.isArray(p.rules))
    .map(p => ({ name: p.name, rules: p.rules.filter(isRule), toggles: { ...DEFAULT_TOGGLES, ...(p.toggles ?? {}) } }));
}

export const saveProfiles = (list: OrganizeProfile[]) => write(PROFILES_KEY, list);

/** "xls, .XLSX  csv" -> ["xls", "xlsx", "csv"] */
export function parseExtensions(text: string): string[] {
  return [...new Set(text.split(/[\s,;]+/).map(normalizeExt).filter(Boolean))];
}

/** Ilk hatayi Turkce mesaj olarak dondurur; kurallar gecerliyse null. */
export function validateRules(rules: RuleDef[]): string | null {
  const owner = new Map<string, string>();
  for (const r of rules) {
    const category = r.category.trim();
    const kind = r.kind.trim();
    if (!category || !kind) return "Her kuralın kategori ve tür klasörü adı olmalı.";
    const bad = [category, kind].find(n => BAD_CHARS.test(n));
    if (bad) return `“${bad}” klasör adında kullanılamayan karakter var (< > : " / \\ | ? *).`;
    if ([category, kind].some(n => n.toLowerCase() === "_kopyalar_karantina")) return "“_Kopyalar_Karantina” klasör adı olarak kullanılamaz.";
    const exts = r.extensions.map(normalizeExt).filter(Boolean);
    if (!exts.length) return `“${kind}” kuralında en az bir uzantı olmalı.`;
    for (const ext of exts) {
      const prev = owner.get(ext);
      if (prev) return `“.${ext}” uzantısı iki kurala bağlı: ${prev} ve ${kind}.`;
      owner.set(ext, kind);
    }
  }
  return null;
}

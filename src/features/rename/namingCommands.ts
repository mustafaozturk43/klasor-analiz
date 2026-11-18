/** Kural tabanli adlandirma: secenekler, hazir kaliplar ve Tauri komutlari (`name_rules_apply.rs`). */
import { invoke } from "@tauri-apps/api/core";

export interface NameRuleOptions {
  scope: "both" | "dirs" | "files";
  includeSubdirs: boolean;
  case: "keep" | "title" | "sentence" | "lower" | "upper";
  /** Bosluk yerine ne konacagi. */
  space: "keep" | "dash" | "underscore" | "none";
  /** Turkce harfleri sadelestir (ç→c, ğ→g, ı→i, ö→o, ş→s, ü→u). */
  ascii: boolean;
  clean: boolean;
  seqDirs: boolean;
  seqFiles: boolean;
  start: number;
  pad: number;
  date: "none" | "my" | "ym";
  /** Ad govdesinin en fazla karakter sayisi (uzanti haric); 0 = sinir yok. */
  maxLen: number;
}

/** `custom`: ad, kullanicinin bu oge icin yazdigi ozel metinden geliyor. */
export interface NameEntry { from: string; to: string | null; warning: string | null; isDir: boolean; custom: boolean; guarded: boolean }
export interface NamePlan { id: string; entries: NameEntry[] }

export const NAMING_DEFAULTS: NameRuleOptions = {
  scope: "both", includeSubdirs: true, case: "title", space: "dash", ascii: false, clean: false,
  seqDirs: true, seqFiles: false, start: 1, pad: 2, date: "none", maxLen: 0,
};

export const NAMING_PRESETS: { id: string; label: string; hint: string; options: NameRuleOptions }[] = [
  { id: "web", label: "Web uyumlu", hint: "küçük harf, tire, Türkçe harf yok, en fazla 60 karakter",
    options: { ...NAMING_DEFAULTS, case: "lower", ascii: true, clean: true, seqDirs: false, maxLen: 60 } },
  { id: "archive", label: "Arşiv", hint: "İlk Harfler Büyük, alt çizgi, klasörlere sıra no, sona Ay-Yıl",
    options: { ...NAMING_DEFAULTS, space: "underscore", date: "my", maxLen: 80 } },
  { id: "plain", label: "Sade", hint: "yalnız ilk harf büyük, boşluklar kalır",
    options: { ...NAMING_DEFAULTS, case: "sentence", space: "keep", seqDirs: false } },
];

/** `customNames`: tam yol -> o dosya icin yazilan ozel ad (uzantisiz); kurallarin urettigi adin yerine gecer. */
export type NamePreviewOptions = NameRuleOptions & { excludeFolders: string[]; customNames: Record<string, string> };

export const previewNameRules = (rootPath: string, options: NamePreviewOptions) =>
  invoke<NamePlan>("preview_name_rules", { rootPath, options });

export const applyNameRules = (planId: string) => invoke<number>("apply_name_rules", { planId });

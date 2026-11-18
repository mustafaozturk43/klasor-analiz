/** Kural motoru komutlari: onizleme plani sunucuda bekletilir, `applyRules(plan.id)` ile uygulanir. */
import { Channel, invoke } from "@tauri-apps/api/core";
import type { MoveRecord, ScanProgress } from "../../types/file";
import type { FailedItem } from "../today/inboxCommands";

export type RuleAction = "move" | "moveYearMonth" | "trash";

export interface Rule {
  name: string;
  enabled: boolean;
  /** Bos: uzanti kosulu yok. */
  exts: string[];
  /** Bos: ad kosulu yok. */
  nameContains: string;
  minMb: number | null;
  maxMb: number | null;
  minAgeDays: number | null;
  action: RuleAction;
  /** Secili klasore gore goreli hedef; `trash` icin kullanilmaz. */
  target: string;
}

export interface RuleEntry { from: string; to: string; rule: string; action: RuleAction }
export interface RuleCount { rule: string; action: RuleAction; count: number; samples: string[] }
export interface RulePlan {
  id: string;
  root: string;
  entries: RuleEntry[];
  counts: RuleCount[];
  scanned: number;
  unmatched: number;
  inPlace: number;
  trashCount: number;
  createdDirs: string[];
}
export interface RuleOutcome { moved: MoveRecord[]; trashed: number; failed: FailedItem[]; removedDirs: number }

/** Diske dokunmaz; kurallardan ilk eslesen her dosyaya uygulanir. */
export const previewRules = (root: string, rules: Rule[], recursive: boolean) =>
  invoke<RulePlan>("preview_rules", { root, rules, recursive });

/** `allowTrash` yalnizca kullanici Geri Donusum Kutusu onayini verdiyse true olur. */
export async function applyRules(planId: string, allowTrash: boolean, onProgress: (p: ScanProgress) => void): Promise<RuleOutcome> {
  const channel = new Channel<ScanProgress>();
  channel.onmessage = onProgress;
  return invoke<RuleOutcome>("apply_rules", { planId, allowTrash, onProgress: channel });
}

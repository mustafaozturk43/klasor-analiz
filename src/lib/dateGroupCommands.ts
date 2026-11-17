/** Tarihe Gore (Yıl\Ay) gruplama komutlari. */
import { Channel, invoke } from "@tauri-apps/api/core";
import type { MoveRecord, ScanProgress } from "../types/file";
import type { MoveOutcome } from "./inboxCommands";

export interface DateGroupOptions { extensions: string[]; recursive: boolean }
export interface DateBucket { folder: string; count: number; samples: string[] }
export interface DateGroupPlan {
  id: string;
  root: string;
  entries: MoveRecord[];
  buckets: DateBucket[];
  /** Zaten dogru Yıl\Ay klasorunde olanlar. */
  inPlace: number;
  /** Tarihi cekim tarihinden alinanlar. */
  fromTaken: number;
  renamed: number;
  createdDirs: string[];
}

export const UNDATED = "Tarihsiz";

/** Diske dokunmaz; plan sunucuda bekletilir, `applyDateGroup(plan.id)` ile uygulanir. */
export async function previewDateGroup(root: string, options: DateGroupOptions, onProgress: (p: ScanProgress) => void): Promise<DateGroupPlan> {
  const channel = new Channel<ScanProgress>();
  channel.onmessage = onProgress;
  return invoke<DateGroupPlan>("preview_date_group", { root, options, onProgress: channel });
}

export const applyDateGroup = (planId: string) => invoke<MoveOutcome>("apply_date_group", { planId });

/** Tarihe Gore (Yıl\Ay) gruplama komutlari. */
import { Channel, invoke } from "@tauri-apps/api/core";
import type { MoveRecord, ScanProgress } from "../types/file";
import type { MoveOutcome } from "./inboxCommands";

export type DateLayout = "yearMonth" | "month" | "yearNumbered";
export interface DateGroupOptions { extensions: string[]; recursive: boolean; layout: DateLayout }

export const DATE_INDEX_FILE = "Tarih_Dizini.html";
export const LAYOUTS: { id: DateLayout; label: string; example: string }[] = [
  { id: "yearMonth", label: "Yıl\\Ay", example: "Örn. 2026\\01 (yıl klasörü içinde ay klasörleri)" },
  { id: "month", label: "Aya göre", example: "Örn. 2026-01 (tek düzeyde ay klasörleri)" },
  { id: "yearNumbered", label: "Yıl (sıra no)", example: "Örn. 01_2025, 02_2026 (yıla göre, sıra numaralı)" },
];
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

/** "Türüne Göre Düzenle" Tauri komutlari. */
import { Channel, invoke } from "@tauri-apps/api/core";
import type { ScanProgress } from "../../types/file";
import type { OrganizeEntry, OrganizeOptions, OrganizeOutcome, OrganizePlan, RuleDef } from "../../types/organize";

export async function previewOrganize(
  rootPath: string,
  options: OrganizeOptions,
  onProgress: (p: ScanProgress) => void,
): Promise<OrganizePlan> {
  const channel = new Channel<ScanProgress>();
  channel.onmessage = onProgress;
  return invoke<OrganizePlan>("preview_organize", { rootPath, options, onProgress: channel });
}

export const applyOrganize = (planId: string) =>
  invoke<OrganizeOutcome>("apply_organize", { planId });

export const defaultOrganizeRules = () => invoke<RuleDef[]>("default_organize_rules");

export const exportOrganizeCsv = (filePath: string, entries: OrganizeEntry[]) =>
  invoke<number>("export_organize_csv", { filePath, entries });

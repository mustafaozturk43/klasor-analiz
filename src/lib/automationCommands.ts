/** Otomasyon komutlari: gelen kutusu durumu ve sablonla adlandirma. */
import { invoke } from "@tauri-apps/api/core";
import type { RenamePlan } from "./fileOperations";

export interface AutomationStatus {
  enabled: boolean;
  paused: boolean;
  running: boolean;
  sources: number;
  label: string;
}

export interface InboxMove { from: string; to: string; group: string }
export interface InboxUpdate { moved: InboxMove[]; failed: { path: string; error: string }[] }

export interface TemplateOptions {
  template: string;
  start: number;
  pad: number;
  sort: "name" | "date";
  extensions: string[];
  includeSubdirs: boolean;
  excludeFolders: string[];
}

export const getAutomationStatus = () => invoke<AutomationStatus>("get_automation_status");

export const setAutomationPaused = (paused: boolean) => invoke<AutomationStatus>("set_automation_paused", { paused });

export const previewTemplateRename = (rootPath: string, options: TemplateOptions) =>
  invoke<RenamePlan>("preview_template_rename", { rootPath, options });

export const applyRenamePlan = (planId: string) => invoke<number>("apply_rename_plan", { planId });

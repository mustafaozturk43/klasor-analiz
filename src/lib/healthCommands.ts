/** Klasor sagligi, karantina yonetimi ve gecmisten geri alma komutlari. */
import { invoke } from "@tauri-apps/api/core";
import type { UndoRecord } from "../types/file";
import type { BatchOutcome, HealthReport, QuarantineListing } from "../types/health";

export const folderHealth = (rootPath: string, includeSubdirs: boolean, excludeFolders: string[]) =>
  invoke<HealthReport>("folder_health", { rootPath, includeSubdirs, excludeFolders });

export const removeEmptyFolders = (rootPath: string, paths: string[]) =>
  invoke<number>("remove_empty_folders", { rootPath, paths });

export const quarantineFiles = (rootPath: string, paths: string[], label: string) =>
  invoke<BatchOutcome>("quarantine_files", { rootPath, paths, label });

export const listQuarantine = (rootPath: string) =>
  invoke<QuarantineListing>("list_quarantine", { rootPath });

export const restoreQuarantined = (rootPath: string, paths: string[]) =>
  invoke<BatchOutcome>("restore_quarantined", { rootPath, paths });

export const deleteQuarantined = (rootPath: string, paths: string[]) =>
  invoke<number>("delete_quarantined", { rootPath, paths });

export const undoHistoryEntry = (id: string) =>
  invoke<UndoRecord>("undo_history_entry", { id });

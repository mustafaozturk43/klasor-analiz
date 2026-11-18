/** Gelen kutusu komutlari. Tarih "YYYY-MM-DD"; verilmezse bugun (yerel saat). */
import { invoke } from "@tauri-apps/api/core";
import type { MoveRecord } from "../../types/file";
import type { InboxListing } from "../../types/inbox";
import type { RenamePlan } from "../../lib/fileOperations";

export interface TargetFolder { name: string; path: string }
export interface FailedItem { path: string; error: string }
export interface MoveOutcome { moved: MoveRecord[]; failed: FailedItem[]; removedDirs: number }
export interface TrashOutcome { trashed: string[]; failed: FailedItem[] }
export interface ListRenameOptions { template: string; customText: string; start: number; pad: number }

export const listInbox = (date?: string) => invoke<InboxListing>("list_inbox", { date: date ?? null });

export const getTargetSubfolders = () => invoke<TargetFolder[]>("get_target_subfolders");

export const moveSelectedFiles = (paths: string[], target: string, group = "") => invoke<MoveOutcome>("move_selected_files", { paths, target, group });

/** Onizleme; uygulamak icin `applyRenamePlan(plan.id)` (automationCommands). */
export const previewInboxRename = (paths: string[], options: ListRenameOptions) => invoke<RenamePlan>("preview_inbox_rename", { paths, options });

/** Kalici silmez: Windows Geri Donusum Kutusu'na gonderir. Islem Gecmisi'nden geri alinamaz. */
export const deleteFilesToTrash = (paths: string[]) => invoke<TrashOutcome>("delete_files_to_trash", { paths });

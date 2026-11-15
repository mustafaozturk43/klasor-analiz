import { invoke } from "@tauri-apps/api/core";
import type { FileItem, FolderFile } from "../types/file";
import type { FileGroup, KeepPolicy } from "../types/file";

export interface RenamePlan {
  id: string;
  entries: { from: string; to: string | null; warning: string | null }[];
}
export const previewDateRename = (roots: string[], paths: string[]) =>
  invoke<RenamePlan>("preview_date_rename", { roots, paths });
export const moveNamedGroups = (rootPath: string, rootB: string, batches: { group: FileGroup; tag: string }[], keepPolicy: KeepPolicy, verifyLarge: boolean) =>
  invoke<number>("move_named_groups", { rootPath, rootB, batches, keepPolicy, verifyLarge });

export function previewItem(file: FolderFile): FileItem {
  const dot = file.name.lastIndexOf(".");
  return { ...file, stem: dot > 0 ? file.name.slice(0, dot) : file.name,
    extension: dot > 0 ? file.name.slice(dot + 1).toLowerCase() : "",
    category: file.category ?? "Other", mtime: 0, hashId: "" };
}
export const creationDate = (seconds: number | null) => seconds === null
  ? "Tarih okunamadı" : new Date(seconds * 1000).toLocaleDateString("tr-TR");

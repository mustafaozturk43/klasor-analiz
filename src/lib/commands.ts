/**
 * Tauri komutlarini turlu sarmalayan ince katman.
 * App/hook dosyalari dogrudan invoke kurallariyla ugrasmaz.
 */
import { Channel, invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { confirmAlert } from "./alerts";
import type {
  FileGroup,
  FolderFile,
  FolderStatsResult,
  ScanProgress,
  ScanResult,
  UndoRecord,
} from "../types/file";

export interface ScanOptions {
  rootPath: string;
  includeSubdirs: boolean;
  /** 0-100 arasi yuzde (orn. 82). */
  thresholdPercent: number;
  excludeFolders: string[];
  /** Kategoriye girse bile taranmayacak uzantilar (F-B). */
  bannedExtensions?: string[];
  /** Kategoriye dusmeyen dosyalari da tara (FAZ 2: "Diğer"). */
  includeOther?: boolean;
  onProgress: (p: ScanProgress) => void;
}

export async function scanAndGroup(opts: ScanOptions): Promise<ScanResult> {
  const channel = new Channel<ScanProgress>();
  channel.onmessage = opts.onProgress;
  return invoke<ScanResult>("scan_and_group", {
    rootPath: opts.rootPath,
    includeSubdirs: opts.includeSubdirs,
    similarityThreshold: opts.thresholdPercent / 100,
    excludeFolders: opts.excludeFolders,
    bannedExtensions: opts.bannedExtensions ?? [],
    includeOther: opts.includeOther ?? false,
    onProgress: channel,
  });
}

export const getFolderStats = (rootPath: string) =>
  invoke<FolderStatsResult>("get_folder_stats", { rootPath });

export interface CompareOptions extends ScanOptions {
  /** Karsilastirilacak ikinci klasor (A = rootPath). */
  rootB: string;
}

/** Iki klasoru karsilastirir (A = ana kok, tasima hedefi). */
export async function compareFolders(opts: CompareOptions): Promise<ScanResult> {
  const channel = new Channel<ScanProgress>();
  channel.onmessage = opts.onProgress;
  return invoke<ScanResult>("compare_folders", {
    rootA: opts.rootPath,
    rootB: opts.rootB,
    includeSubdirs: opts.includeSubdirs,
    similarityThreshold: opts.thresholdPercent / 100,
    excludeFolders: opts.excludeFolders,
    bannedExtensions: opts.bannedExtensions ?? [],
    includeOther: opts.includeOther ?? false,
    onProgress: channel,
  });
}

export const cancelScan = () => invoke("cancel_scan");
export const cancelOrganize = () => invoke("cancel_organize");

export interface ListFilesOptions {
  rootPath: string;
  includeSubdirs: boolean;
  excludeFolders: string[];
  bannedExtensions: string[];
  /** Kategoriye dusmeyen dosyalari da dahil et (Tum dosyalar modu). */
  includeUncategorized: boolean;
}

/** Klasordeki dosyalari düz listeler (hash yok; kategori/secime gore filtrelenir). */
export async function listFiles(opts: ListFilesOptions): Promise<FolderFile[]> {
  return invoke<FolderFile[]>("list_files", {
    rootPath: opts.rootPath,
    includeSubdirs: opts.includeSubdirs,
    excludeFolders: opts.excludeFolders,
    bannedExtensions: opts.bannedExtensions,
    includeUncategorized: opts.includeUncategorized,
  });
}

export const getMoveHistory = () => invoke<UndoRecord[]>("get_move_history");

export const clearMoveHistory = () => invoke("clear_move_history");

export const undoLastMove = () =>
  invoke<UndoRecord | null>("undo_last_move");

export const clearQuarantine = (rootPath: string) =>
  invoke("clear_quarantine", { rootPath });

export const revealInExplorer = (path: string) =>
  invoke("reveal_in_explorer", { path });

export const openFileDefault = (path: string) =>
  invoke("open_file_default", { path });

/** Kaydet diyalogu acar; iptal edilirse null doner. */
export async function pickCsvTarget(defaultName: string): Promise<string | null> {
  const target = await save({
    title: "CSV Rapor Kaydet",
    defaultPath: defaultName,
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  return target ?? null;
}

export const exportHistoryCsv = (filePath: string) => invoke<number>("export_history_csv", { filePath });

export const writeGroupsCsv = (filePath: string, groups: FileGroup[]) =>
  invoke<number>("export_groups_csv", { filePath, groups });

export async function pickFolder(): Promise<string | null> {
  const picked = await open({
    directory: true,
    multiple: false,
    title: "Ana Klasörü Seçin",
  });
  return typeof picked === "string" && picked ? picked : null;
}

export async function confirmDialog(
  message: string,
  title: string,
  okLabel = "Onayla",
): Promise<boolean> {
  return confirmAlert(message, title, okLabel);
}

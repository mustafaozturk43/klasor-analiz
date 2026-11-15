/**
 * Rust modelleriyle (serde camelCase) birebir eslesen TypeScript tipleri.
 * Kaynak: src-tauri/src/models.rs
 */

export type FileCategory = "Document" | "Image" | "Archive" | "Other";
export type GroupType = "Exact" | "Similar";

/** Filtre secenekleri (UI tarafinda kullanilir, Rust'a gonderilmez). */
export type CategoryFilter = "All" | FileCategory;
export type GroupTypeFilter = "All" | GroupType;

/** Grup listesi siralama modlari. */
export type SortMode = "sizeDesc" | "sizeAsc" | "countDesc" | "nameAsc";

export interface FileItem {
  path: string;
  name: string;
  stem: string;
  extension: string;
  size: number;
  /** Son degisiklik zamani (unix saniye). */
  mtime: number;
  category: FileCategory;
  hashId: string;
}

export interface FileGroup {
  id: string;
  groupType: GroupType;
  category: FileCategory;
  files: FileItem[];
  wastedBytes: number;
}

export interface MoveRecord {
  from: string;
  to: string;
}

/** Exact grupta hangi dosyanin tutulacagi politikasi (F2). */
export type KeepPolicy = "first" | "newest" | "oldest" | "shortestPath";

/** Dosya listesi panel modu: "category" veya "all". */
export type FileListMode = "category" | "all";

/** Bir tasma isleminin geri alma kaydi (JSON gecmiste saklanir). */
export interface UndoRecord {
  id: string;
  timestamp: number;
  tag: string;
  moves: MoveRecord[];
}

export interface ScanResult {
  rootPath: string;
  totalFiles: number;
  groups: FileGroup[];
  comparison: { rootB: string; files: ComparedFile[] } | null;
  /** Okunamadigi icin atlanan klasor/dosyalar. */
  warnings?: string[];
}

export interface ComparedFile {
  file: FolderFile;
  source: "A" | "B";
  groupId: string | null;
}

/** Canli tarama ilerleme olayi (Rust Channel). */
export interface ScanProgress {
  phase: "scanning" | "hashing" | "done";
  filesDone: number;
  totalFiles: number;
}

/** Bir alt klasordeki tek dosya (dosya listeleme icin). */
export interface FolderFile {
  name: string;
  path: string;
  size: number;
  category: FileCategory | null;
  createdAt: number | null;
  /** Son degisiklik zamani (unix saniye). */
  modifiedAt: number | null;
}

/** Bir alt klasorun boyut ozeti. */
export interface FolderStat {
  name: string;
  path: string;
  totalBytes: number;
  fileCount: number;
  documentBytes: number;
  imageBytes: number;
  archiveBytes: number;
  otherBytes: number;
  files: FolderFile[];
}

/** Ana klasor + alt klasorlerin boyut ozeti. */
export interface FolderStatsResult {
  rootPath: string;
  rootTotalBytes: number;
  rootFileCount: number;
  folders: FolderStat[];
  categoryBytes: [number, number, number, number];
  warnings: string[];
}

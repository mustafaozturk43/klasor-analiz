/** Benzer gorseller: gorsel parmak izi (dHash) ile yeniden boyutlandirilmis/sikistirilmis kopyalar. */
import { Channel, invoke } from "@tauri-apps/api/core";
import type { ScanProgress } from "../../types/file";

export interface VisualFile {
  path: string;
  size: number;
  width: number;
  height: number;
  mtime: number;
  /** Grubun ilk (en iyi) dosyasina Hamming mesafesi, 0..64. */
  distance: number;
}
export interface VisualGroup { id: string; files: VisualFile[] }
export interface VisualReport { rootPath: string; scanned: number; unreadable: number; groups: VisualGroup[] }

/** Karsilastirma hassasiyeti: en fazla kac bit fark olabilir (64 bitlik parmak izi). */
export const SENSITIVITY = [
  { id: 3, label: "Çok benzer", hint: "Neredeyse aynı: yalnızca yeniden boyutlandırılmış ya da sıkıştırılmış kopyalar" },
  { id: 6, label: "Benzer", hint: "Hafif kırpma, parlaklık ve renk düzenlemeleri de eşleşir" },
  { id: 10, label: "Gevşek", hint: "Daha çok eşleşme; yanlış eşleşme olabilir, kontrol edin" },
] as const;

export const DEFAULT_DISTANCE = 6;

/** Diske yazmaz. Cozulemeyen resimler `unreadable` ile sayilir. */
export async function findSimilarImages(
  rootPath: string, includeSubdirs: boolean, excludeFolders: string[], maxDistance: number, onProgress: (p: ScanProgress) => void,
): Promise<VisualReport> {
  const channel = new Channel<ScanProgress>();
  channel.onmessage = onProgress;
  return invoke<VisualReport>("find_similar_images", { rootPath, includeSubdirs, excludeFolders, maxDistance, onProgress: channel });
}

/** 0 bit fark = %100; her bit ~%1,6. */
export const similarityPct = (distance: number) => Math.round((64 - distance) / 64 * 100);

/** Her grupta ilk (en iyi) dosyayi tutup digerlerini isaretler. */
export const defaultSelection = (groups: VisualGroup[]): Set<string> => new Set(groups.flatMap(g => g.files.slice(1).map(f => f.path)));

/** `path` secilirse grupta hic dosya tutulmayacaksa (hepsi secili olurdu) false. */
export function canSelect(group: VisualGroup, selected: Set<string>, path: string): boolean {
  return group.files.some(f => f.path !== path && !selected.has(f.path));
}

import type { FileItem, KeepPolicy } from "../types/file";

/**
 * Tutulacak dosyanin indeksini politikaya gore hesaplar.
 * Backend'deki `mover::kept_index` ile ayni kurallar (rozet tutarliligi icin).
 */
export function keptIndex(files: FileItem[], policy: KeepPolicy): number {
  if (files.length === 0) return 0;
  let best = 0;
  switch (policy) {
    case "first":
      return 0;
    case "newest":
      for (let i = 1; i < files.length; i++) {
        if (files[i].mtime >= files[best].mtime) best = i;
      }
      return best;
    case "oldest":
      for (let i = 1; i < files.length; i++) {
        if (files[i].mtime < files[best].mtime) best = i;
      }
      return best;
    case "shortestPath":
      for (let i = 1; i < files.length; i++) {
        if (Array.from(files[i].path).length < Array.from(files[best].path).length) best = i;
      }
      return best;
  }
}

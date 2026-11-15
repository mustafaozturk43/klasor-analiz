import { useEffect, useState } from "react";
import * as api from "../lib/commands";
import type { FolderFile } from "../types/file";

export type FileListMode = "category" | "all";

/**
 * Klasor dosya listesi durumu (Hash yok, sadece metadata).
 * `enabled` switch'i acikken mevcut kok klasordaki dosyalari listeler.
 * Mod: "category" (sadece tarama kategorileri) veya "all" (tum dosyalar).
 */
export function useFileList(
  rootPath: string,
  includeSubdirs: boolean,
  excludeFolders: string[],
  bannedExtensions: string[],
  enabled: boolean,
  includeUncategorized: boolean,
  refreshToken = 0,
) {
  const [files, setFiles] = useState<FolderFile[]>([]);
  const [isListing, setIsListing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    setFiles([]);
    setError(null);
    if (!enabled || !rootPath) {
      setIsListing(false);
      return;
    }
    setIsListing(true);
    api.listFiles({
        rootPath,
        includeSubdirs,
        excludeFolders,
        bannedExtensions,
        includeUncategorized,
      }).then(result => { if (active) setFiles(result); })
      .catch(e => { if (active) setError(String(e)); })
      .finally(() => { if (active) setIsListing(false); });
    return () => { active = false; };
  }, [
    enabled,
    rootPath,
    includeSubdirs,
    excludeFolders,
    bannedExtensions,
    includeUncategorized,
    refreshToken,
  ]);

  return { files, isListing, error };
}

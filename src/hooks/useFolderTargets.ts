import { useEffect, useState } from "react";
import { listSubfolders } from "../lib/selectionCommands";

/**
 * Hedef klasor secicisi icin kok(ler)in altindaki klasorler. Yalnizca `enabled`
 * iken (secim varken) yuklenir; `token` degisince (tasima sonrasi) yenilenir.
 */
export function useFolderTargets(roots: string[], excludes: string[], token: number, enabled: boolean) {
  const key = JSON.stringify([roots, excludes, token, enabled]);
  const [state, setState] = useState<{ key: string; folders: string[]; truncated: boolean }>({ key: "", folders: [], truncated: false });
  useEffect(() => {
    if (!enabled) return;
    let alive = true;
    const [rs, ex] = JSON.parse(key) as [string[], string[]];
    Promise.all(rs.map(r => listSubfolders(r, ex).catch(() => ({ paths: [] as string[], truncated: false }))))
      .then(res => { if (alive) setState({ key, folders: res.flatMap(r => r.paths), truncated: res.some(r => r.truncated) }); });
    return () => { alive = false; };
  }, [key, enabled]);
  return { folders: state.folders, truncated: state.truncated, loading: enabled && state.key !== key };
}

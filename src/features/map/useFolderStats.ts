import { useEffect, useState } from "react";
import * as api from "../../lib/commands";
import type { FolderStatsResult } from "../../types/file";

// Analysis and comparison share the same metadata request for a root/revision.
const requests = new Map<string, Promise<FolderStatsResult>>();
function request(root: string, token: number) {
  const key = JSON.stringify([root, token]);
  if (!requests.has(key)) {
    if (requests.size >= 12) requests.delete(requests.keys().next().value!);
    const pending = api.getFolderStats(root).catch(e => {
      if (requests.get(key) === pending) requests.delete(key);
      throw e;
    });
    requests.set(key, pending);
  }
  return requests.get(key)!;
}

/**
 * Ana klasorun (ve alt klasorlerinin) boyut istatistiklerini ceker.
 * `refreshToken` her arttiginda (tarama/tasma/undo sonrasi) yenilenir.
 */
export function useFolderStats(rootPath: string, refreshToken: number) {
  const [stats, setStats] = useState<FolderStatsResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!rootPath) {
      setStats(null);
      setLoading(false);
      setError(null);
      return;
    }
    let cancelled = false;
    setLoading(true);
    setStats(null);
    setError(null);
    request(rootPath, refreshToken)
      .then((s) => {
        if (!cancelled) setStats(s);
      })
      .catch((e) => {
        if (!cancelled) { setStats(null); setError(String(e)); }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [rootPath, refreshToken]);

  return { stats, loading, error };
}

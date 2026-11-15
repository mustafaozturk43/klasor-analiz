import { useCallback, useEffect, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { getShellThumbnails } from "../lib/selectionCommands";

const BATCH = 30;

/**
 * Windows kucuk resimleri: gorunen satirlar `request` ile istenir, 30'luk paketler
 * sirayla islenir, ayni yol iki kez istenmez. `thumbOf`: undefined = bekliyor,
 * null = Windows resim veremedi, string = gosterilebilir adres.
 */
export function useShellThumbs(size: number) {
  const cache = useRef(new Map<string, string | null>());
  const queue = useRef<string[]>([]);
  const queued = useRef(new Set<string>());
  const running = useRef(false);
  const alive = useRef(true);
  const [, setVersion] = useState(0);

  useEffect(() => { alive.current = true; return () => { alive.current = false; }; }, []);

  const pump = useCallback(async () => {
    if (running.current) return;
    running.current = true;
    while (queue.current.length && alive.current) {
      const chunk = queue.current.splice(0, BATCH);
      const result = await getShellThumbnails(chunk, size).catch(() => chunk.map(() => null));
      chunk.forEach((p, i) => {
        const file = result[i];
        cache.current.set(p, file ? convertFileSrc(file) : null);
        queued.current.delete(p);
      });
      if (alive.current) setVersion(v => v + 1);
    }
    running.current = false;
  }, [size]);

  const request = useCallback((paths: string[]) => {
    const fresh = paths.filter(p => !cache.current.has(p) && !queued.current.has(p));
    if (!fresh.length) return;
    fresh.forEach(p => queued.current.add(p));
    queue.current.push(...fresh);
    void pump();
  }, [pump]);

  const thumbOf = useCallback((path: string) => cache.current.get(path), []);
  return { thumbOf, request };
}

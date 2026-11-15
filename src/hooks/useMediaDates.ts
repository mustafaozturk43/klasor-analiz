import { useEffect, useMemo, useRef, useState } from "react";
import { readMediaDates } from "../lib/selectionCommands";

const BATCH = 50;

/**
 * Cekim tarihlerini arka planda 50'lik paketlerle okur. Sonuclar yola gore
 * onbellekte tutulur; klasor degisince (`key`) suren okuma durur ve eski klasorun
 * sonuclari yeni listeye karismaz (yalnizca istenen yollar sayilir).
 */
export function useMediaDates(paths: string[], key: string) {
  const cache = useRef(new Map<string, number | null>());
  const [, setVersion] = useState(0);
  const pathsKey = paths.join("\n");

  useEffect(() => {
    let alive = true;
    const todo = pathsKey ? pathsKey.split("\n").filter(p => !cache.current.has(p)) : [];
    (async () => {
      for (let i = 0; i < todo.length && alive; i += BATCH) {
        const chunk = todo.slice(i, i + BATCH);
        const result = await readMediaDates(chunk).catch(() => chunk.map(() => null));
        chunk.forEach((p, j) => cache.current.set(p, result[j] ?? null));
        if (alive) setVersion(v => v + 1);
      }
    })();
    return () => { alive = false; };
  }, [pathsKey, key]);

  const done = paths.filter(p => cache.current.has(p)).length;
  const taken = useMemo(() => new Map(paths.filter(p => cache.current.has(p)).map(p => [p, cache.current.get(p) ?? null])),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [pathsKey, done]);
  return { taken, done, total: paths.length, reading: done < paths.length };
}

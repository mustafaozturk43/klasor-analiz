import { useCallback, useState } from "react";
import {
  DEFAULT_BANNED_EXTENSIONS,
  loadBannedExtensions,
  normalizeExt,
  saveBannedExtensions,
} from "../lib/bannedExtensions";

/**
 * Yasakli uzanti listesi durumu (settings.json icinde kalici).
 * Tarama bu listeyi backend'e gonderir; uzanti kategoriye girse bile atlanir.
 */
export function useBannedExtensions() {
  const [bannedExtensions, setBannedExtensions] = useState<string[]>(loadBannedExtensions);

  const persist = useCallback((next: string[]) => {
    setBannedExtensions(next);
    saveBannedExtensions(next);
  }, []);

  const addExtension = useCallback(
    (raw: string) => {
      const ext = normalizeExt(raw);
      if (!ext) return;
      setBannedExtensions((prev) => {
        if (prev.includes(ext)) return prev;
        const next = [...prev, ext].sort((a, b) => a.localeCompare(b));
        saveBannedExtensions(next);
        return next;
      });
    },
    [],
  );

  const removeExtension = useCallback((ext: string) => {
    setBannedExtensions((prev) => {
      const next = prev.filter((e) => e !== ext);
      saveBannedExtensions(next);
      return next;
    });
  }, []);

  const resetExtensions = useCallback(() => {
    persist([...DEFAULT_BANNED_EXTENSIONS]);
  }, [persist]);

  return {
    bannedExtensions,
    addExtension,
    removeExtension,
    resetExtensions,
  };
}

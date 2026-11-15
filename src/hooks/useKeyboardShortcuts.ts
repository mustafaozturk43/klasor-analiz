//! Klavye kisayollari (F1): Ctrl+Z=geri al, F5=yeniden tara, Ctrl+A=tum gruplari sec.

import { useEffect } from "react";
import type { FileGroup } from "../types/file";

interface KeyboardShortcutsDeps {
  /** Guncel gorunen (filtrelenmis) gruplar — Ctrl+A hedefi. */
  visibleGroups: FileGroup[];
  scan: () => void;
  undoLast: () => void;
  toggleAllVisible: (groups: FileGroup[]) => void;
}

/**
 * Global klavye kisayollarini baglar. Girdi alanlarinda (input/textarea)
 * varsayilan davranis korunur; kullanicinin metin secimi bozulmaz.
 */
export function useKeyboardShortcuts({
  visibleGroups,
  scan,
  undoLast,
  toggleAllVisible,
}: KeyboardShortcutsDeps) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.defaultPrevented || document.querySelector("dialog[open]")) return;
      const target = e.target as HTMLElement | null;
      const inEditor =
        target?.tagName === "INPUT" ||
        target?.tagName === "TEXTAREA" ||
        target?.isContentEditable;

      if (e.key === "F5" && !inEditor) {
        e.preventDefault();
        scan();
        return;
      }
      if ((e.ctrlKey || e.metaKey) && !e.shiftKey) {
        if (e.key.toLowerCase() === "z" && !inEditor) {
          e.preventDefault();
          undoLast();
        } else if (e.key.toLowerCase() === "a" && !inEditor && visibleGroups.length > 0) {
          e.preventDefault();
          toggleAllVisible(visibleGroups);
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [visibleGroups, scan, undoLast, toggleAllVisible]);
}

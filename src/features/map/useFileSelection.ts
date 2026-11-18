import { useCallback, useMemo, useState } from "react";

interface SelectionState { key: string; selected: Set<string>; last: string | null }

/**
 * Klasordeki dosyalar ekraninda secim: tek tik, Shift ile aralik, toplu sec/birak.
 * `order` gorunen (suzulmus, sirali) yollardir; `resetKey` degisince (klasor degisti) secim temizlenir.
 */
export function useFileSelection(order: string[], resetKey: string) {
  const [raw, setRaw] = useState<SelectionState>({ key: resetKey, selected: new Set(), last: null });
  const state = raw.key === resetKey ? raw : { key: resetKey, selected: new Set<string>(), last: null };
  const update = useCallback((fn: (s: SelectionState) => SelectionState) =>
    setRaw(prev => fn(prev.key === resetKey ? prev : { key: resetKey, selected: new Set(), last: null })), [resetKey]);

  const toggle = useCallback((path: string, shift: boolean) => update(s => {
    const next = new Set(s.selected);
    const on = !next.has(path);
    const [from, to] = [s.last ? order.indexOf(s.last) : -1, order.indexOf(path)];
    if (shift && from >= 0 && to >= 0) {
      order.slice(Math.min(from, to), Math.max(from, to) + 1).forEach(p => (on ? next.add(p) : next.delete(p)));
    } else if (on) next.add(path);
    else next.delete(path);
    return { ...s, selected: next, last: path };
  }), [order, update]);

  const setMany = useCallback((paths: string[], on: boolean) => update(s => {
    const next = new Set(s.selected);
    paths.forEach(p => (on ? next.add(p) : next.delete(p)));
    return { ...s, selected: next };
  }), [update]);

  /** Adlandirmadan sonra eski yollari yenileriyle degistirir. */
  const remap = useCallback((renamed: Map<string, string>) => update(s => ({
    ...s, selected: new Set([...s.selected].map(p => renamed.get(p) ?? p)), last: s.last && (renamed.get(s.last) ?? s.last),
  })), [update]);

  const clear = useCallback(() => update(s => ({ ...s, selected: new Set(), last: null })), [update]);
  const hiddenCount = useMemo(() => {
    const visible = new Set(order);
    return [...state.selected].filter(p => !visible.has(p)).length;
  }, [order, state.selected]);

  return { selected: state.selected, isSelected: (p: string) => state.selected.has(p), toggle, setMany, remap, clear, hiddenCount };
}

export type FileSelection = ReturnType<typeof useFileSelection>;

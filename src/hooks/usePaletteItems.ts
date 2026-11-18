import { useMemo } from "react";
import { baseName } from "../lib/format";
import { DESTINATIONS } from "../lib/navigation";
import type { PaletteItem } from "../lib/palette";
import { loadRecent } from "../lib/recent";
import type { Module, OrganizerState } from "../types/app";
import type { Theme } from "./useTheme";

interface Deps {
  app: OrganizerState;
  go: (module: Module) => void;
  setTheme: (theme: Theme) => void;
  /** Palet her acildiginda son klasorler yeniden okunur. */
  open: boolean;
  checkUpdate: () => Promise<void>;
}

const THEMES: [Theme, string][] = [["light", "Açık"], ["dark", "Koyu"], ["auto", "Otomatik"]];

/** Palet satirlari: tum ekranlar (alt sekmeleriyle), son klasorler ve sik kullanilan islemler. */
export function usePaletteItems({ app, go, setTheme, open, checkUpdate }: Deps): PaletteItem[] {
  const { rootPath, openRecent, scan, undoLast, history, isScanning, isMoving } = app;
  return useMemo(() => {
    const busy = isScanning || isMoving;
    const screens: PaletteItem[] = DESTINATIONS.flatMap(d => d.views.map(v => ({
      id: `screen:${v.id}`, group: "Ekran" as const, label: v.label === d.label ? d.label : `${d.label} › ${v.label}`, run: () => go(v.id),
    })));
    const folders: PaletteItem[] = open ? loadRecent().filter(p => p !== rootPath).map(p => ({
      id: `folder:${p}`, group: "Klasör" as const, label: `Klasörü aç: ${baseName(p)}`, hint: p, disabled: busy, run: () => void openRecent(p),
    })) : [];
    const actions: PaletteItem[] = [
      { id: "act:scan", group: "İşlem", label: "Taramayı başlat", hint: "F5", keywords: "yeniden tara", disabled: busy || !rootPath, run: () => scan() },
      { id: "act:undo", group: "İşlem", label: "Son işlemi geri al", hint: "Ctrl+Z", disabled: busy || !history.length, run: () => void undoLast() },
      { id: "act:update", group: "İşlem", label: "Güncellemeleri denetle", keywords: "yeni sürüm yükselt", disabled: busy, run: () => void checkUpdate() },
      ...THEMES.map(([t, name]): PaletteItem => ({ id: `theme:${t}`, group: "İşlem", label: `Tema: ${name}`, keywords: "görünüm renk", run: () => setTheme(t) })),
    ];
    return [...screens, ...folders, ...actions];
  }, [open, rootPath, openRecent, scan, undoLast, history.length, isScanning, isMoving, go, setTheme, checkUpdate]);
}

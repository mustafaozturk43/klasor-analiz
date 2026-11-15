import { useEffect, useRef, useState } from "react";
import { Monitor, Moon, Settings, Sun } from "lucide-react";
import BannedExtEditor from "./BannedExtEditor";
import Segmented from "./Segmented";
import type { Theme } from "../hooks/useTheme";

interface SettingsMenuProps {
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
  /** Yasakli uzanti listesi (F-B). */
  bannedExtensions: string[];
  onAddBanned: (raw: string) => void;
  onRemoveBanned: (ext: string) => void;
  onResetBanned: () => void;
}

const THEME_TABS: { value: Theme; label: string; icon: typeof Sun }[] = [
  { value: "light", label: "Açık", icon: Sun },
  { value: "dark", label: "Koyu", icon: Moon },
  { value: "auto", label: "Otomatik", icon: Monitor },
];

/** Disari tiklama / Escape ile kapanan ayar menusu (tema + yasakli uzantilar). */
export default function SettingsMenu({
  theme,
  onThemeChange,
  bannedExtensions,
  onAddBanned,
  onRemoveBanned,
  onResetBanned,
}: SettingsMenuProps) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (rootRef.current && !rootRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("mousedown", onDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  return (
    <div className="relative" ref={rootRef}>
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        title="Ayarlar"
        className={`inline-flex h-8 w-8 items-center justify-center rounded-lg border transition-colors ${
          open
            ? "border-blue-500/50 bg-blue-50 text-blue-600 dark:bg-blue-500/10 dark:text-blue-400"
            : "border-zinc-300 bg-white text-zinc-600 hover:bg-zinc-50 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800"
        }`}
      >
        <Settings className="h-4 w-4" />
      </button>

      {open && (
        <div className="absolute right-0 top-full z-50 mt-2 w-80 rounded-xl border border-zinc-200 bg-white p-4 shadow-xl dark:border-zinc-700 dark:bg-zinc-900">
          <p className="mb-2 text-xs font-semibold text-zinc-500 dark:text-zinc-400">
            Görünüm
          </p>
          <Segmented value={theme} options={THEME_TABS} onChange={onThemeChange} />

          <div className="mt-4 border-t border-zinc-100 pt-3 dark:border-zinc-800">
            <BannedExtEditor
              extensions={bannedExtensions}
              onAdd={onAddBanned}
              onRemove={onRemoveBanned}
              onReset={onResetBanned}
            />
          </div>

          <div className="mt-4 border-t border-zinc-100 pt-3 dark:border-zinc-800">
            <p className="text-[11px] text-zinc-400 dark:text-zinc-500">
              Smart File Organizer · v0.1.0
            </p>
            <p className="text-[11px] text-zinc-400 dark:text-zinc-500">
              Tema ve yasaklı uzantılar bu cihazda saklanır.
            </p>
          </div>
        </div>
      )}
    </div>
  );
}

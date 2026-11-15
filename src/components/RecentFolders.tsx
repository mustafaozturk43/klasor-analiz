import { useEffect, useRef, useState } from "react";
import { ChevronDown, FolderClock, FolderOpen, X } from "lucide-react";
import { loadRecent, removeRecent } from "../lib/recent";

interface RecentFoldersProps {
  /** Suan secili klasor (varsa listeden vurgulanmaz). */
  currentPath: string;
  /** Klasor secilince (acilir ve taranir) cagrilir. */
  onPick: (path: string) => void;
}

/** Son taranan klasorler dropdown'i (F1). */
export default function RecentFolders({ currentPath, onPick }: RecentFoldersProps) {
  const [open, setOpen] = useState(false);
  const [folderItem, setFolderItem] = useState<string[]>([]);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setFolderItem(loadRecent().filter((p) => p !== currentPath));
  }, [currentPath, open]);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", onDown);
    return () => document.removeEventListener("mousedown", onDown);
  }, [open]);

  const remove = (path: string) => {
    removeRecent(path);
    setFolderItem(loadRecent().filter((p) => p !== currentPath));
  };

  const pick = (path: string) => {
    setOpen(false);
    onPick(path);
  };

  return (
    <div ref={ref} className="relative">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        disabled={folderItem.length === 0 && !open}
        title="Son taranan klasörler"
        className="inline-flex h-[30px] w-8 items-center justify-center rounded-lg border border-zinc-300 bg-white text-zinc-600 transition-colors hover:bg-zinc-50 disabled:cursor-not-allowed disabled:opacity-40 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800"
      >
        <FolderClock className="h-4 w-4" />
        <ChevronDown className="h-2.5 w-2.5 -ml-0.5" />
      </button>

      {open && (
        <div className="absolute left-0 top-9 z-20 w-80 overflow-hidden rounded-xl border border-zinc-200 bg-white shadow-lg dark:border-zinc-700 dark:bg-zinc-900">
          <p className="border-b border-zinc-100 px-3 py-2 text-[12px] font-semibold text-zinc-500 dark:border-zinc-800 dark:text-zinc-500">
            Son Taranan Klasörler
          </p>
          {folderItem.length === 0 ? (
            <p className="px-3 py-3 text-xs text-zinc-400 dark:text-zinc-500">
              Kayıt yok. Bir klasör tarayın.
            </p>
          ) : (
            <ul className="max-h-64 overflow-y-auto py-1">
              {folderItem.map((path) => (
                <li key={path} className="group flex items-center gap-1 px-1.5 py-0.5">
                  <button
                    type="button"
                    onClick={() => pick(path)}
                    title={path}
                    className="flex min-w-0 flex-1 items-center gap-2 rounded-lg px-2 py-1.5 text-left text-xs text-zinc-700 transition-colors hover:bg-blue-50 hover:text-blue-700 dark:text-zinc-300 dark:hover:bg-zinc-800 dark:hover:text-blue-400"
                  >
                    <FolderOpen className="h-3.5 w-3.5 shrink-0 text-zinc-400" />
                    <span className="truncate">{path}</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => remove(path)}
                    title="Listeden çıkar"
                    className="shrink-0 rounded-md p-1 text-zinc-300 opacity-0 transition-opacity hover:text-rose-500 group-hover:opacity-100 dark:text-zinc-600"
                  >
                    <X className="h-3 w-3" />
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}

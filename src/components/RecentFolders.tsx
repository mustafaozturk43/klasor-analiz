import { useEffect, useState } from "react";
import { ChevronDown, FolderClock, FolderOpen, X } from "lucide-react";
import { loadRecent, removeRecent } from "../lib/recent";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";

interface RecentFoldersProps {
  /** Suan secili klasor (varsa listeden vurgulanmaz). */
  currentPath: string;
  /** Klasor secilince (acilir ve taranir) cagrilir. */
  onPick: (path: string) => void;
}

/** Son taranan klasorler acilir menusu (F1). Radix Popover: Esc / disari tiklama kapatir, odak geri doner. */
export default function RecentFolders({ currentPath, onPick }: RecentFoldersProps) {
  const [open, setOpen] = useState(false);
  const [folderItem, setFolderItem] = useState<string[]>([]);

  useEffect(() => {
    setFolderItem(loadRecent().filter((p) => p !== currentPath));
  }, [currentPath, open]);

  const remove = (path: string) => {
    removeRecent(path);
    setFolderItem(loadRecent().filter((p) => p !== currentPath));
  };

  const pick = (path: string) => {
    setOpen(false);
    onPick(path);
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger
        className="btn"
        disabled={folderItem.length === 0 && !open}
        title="Son taranan klasörler"
        aria-label="Son taranan klasörler"
      >
        <FolderClock size={16} aria-hidden />
        <ChevronDown size={10} className="-ml-1" aria-hidden />
      </PopoverTrigger>
      <PopoverContent className="menu-pop flush" aria-labelledby="recent-title">
        <p id="recent-title" className="menu-label border-b border-[var(--color-rule)] px-3 py-2">
          Son Taranan Klasörler
        </p>
        {folderItem.length === 0 ? (
          <p className="menu-note px-3 py-3">Kayıt yok. Bir klasör tarayın.</p>
        ) : (
          <ul className="recent-list">
            {folderItem.map((path) => (
              <li key={path} className="recent-item">
                <button type="button" className="recent-pick" onClick={() => pick(path)} title={path}>
                  <FolderOpen size={14} aria-hidden />
                  <span>{path}</span>
                </button>
                <button
                  type="button"
                  className="recent-remove"
                  onClick={() => remove(path)}
                  title="Listeden çıkar"
                  aria-label={`${path} klasörünü listeden çıkar`}
                >
                  <X size={14} aria-hidden />
                </button>
              </li>
            ))}
          </ul>
        )}
      </PopoverContent>
    </Popover>
  );
}

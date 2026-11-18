import { useState } from "react";
import { Ban, Plus, RotateCcw, X } from "lucide-react";
import { DEFAULT_BANNED_EXTENSIONS } from "./bannedExtensions";

interface BannedExtEditorProps {
  extensions: string[];
  onAdd: (raw: string) => void;
  onRemove: (ext: string) => void;
  onReset: () => void;
}

/**
 * Yasakli uzanti listesi duzenleyici: chip listesi + ekleme kutusu.
 * Bu uzantilar taranmaz (kategoriye girse bile).
 */
export default function BannedExtEditor({
  extensions,
  onAdd,
  onRemove,
  onReset,
}: BannedExtEditorProps) {
  const [draft, setDraft] = useState("");

  const commit = () => {
    if (!draft.trim()) return;
    onAdd(draft);
    setDraft("");
  };

  return (
    <div>
      <div className="mb-1.5 flex items-center justify-between">
        <p className="flex items-center gap-1.5 text-xs font-semibold text-zinc-500 dark:text-zinc-400">
          <Ban className="h-3.5 w-3.5" />
          Yasaklı Uzantılar
        </p>
        <button
          type="button"
          onClick={onReset}
          title="Varsayılan listeye dön"
          className="inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[10px] text-zinc-400 transition-colors hover:text-blue-600 dark:text-zinc-500 dark:hover:text-blue-400"
        >
          <RotateCcw className="h-3 w-3" />
          Sıfırla
        </button>
      </div>

      <p className="mb-2 text-[10px] leading-relaxed text-zinc-400 dark:text-zinc-500">
        Sistem/çalıştırılabilir dosyalar taranmaz. Varsayılan:{" "}
        {DEFAULT_BANNED_EXTENSIONS.slice(0, 6).join(", ")}…
      </p>

      <div className="flex flex-wrap gap-1">
        {extensions.length === 0 ? (
          <span className="text-[11px] text-zinc-400 dark:text-zinc-500">
            Liste boş: hiçbir uzantı engellenmiyor.
          </span>
        ) : (
          extensions.map((ext) => (
            <span
              key={ext}
              className="inline-flex items-center gap-1 rounded-md bg-rose-50 px-1.5 py-0.5 text-[11px] font-medium text-rose-700 dark:bg-rose-500/10 dark:text-rose-400"
            >
              .{ext}
              <button
                type="button"
                onClick={() => onRemove(ext)}
                title={`${ext} listesinden çıkar`}
                className="rounded-sm p-0.5 transition-colors hover:bg-rose-100 dark:hover:bg-rose-500/20"
              >
                <X className="h-2.5 w-2.5" />
              </button>
            </span>
          ))
        )}
      </div>

      <div className="mt-2 flex items-center gap-1.5">
        <input
          type="text"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              commit();
            }
          }}
          placeholder="örn: tmp veya .log"
          spellCheck={false}
          className="min-w-0 flex-1 rounded-lg border border-zinc-300 bg-white px-2 py-1 text-xs outline-none transition-colors placeholder:text-zinc-400 focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 dark:border-zinc-700 dark:bg-zinc-950 dark:placeholder:text-zinc-500"
        />
        <button
          type="button"
          onClick={commit}
          disabled={!draft.trim()}
          title="Uzantıyı yasaklı listeye ekle"
          className={"inline-flex shrink-0 items-center gap-1 rounded-lg border border-zinc-300 bg-white px-2 py-1 text-xs font-medium text-zinc-700 transition-colors hover:bg-zinc-50 disabled:cursor-not-allowed disabled:opacity-40 " +
            "dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800"}
        >
          <Plus className="h-3 w-3" />
          Ekle
        </button>
      </div>
    </div>
  );
}

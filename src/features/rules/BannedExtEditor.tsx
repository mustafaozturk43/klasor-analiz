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
 * Bu uzantilar taranmaz (kategoriye girse bile). Renkler menus.css token'larindan.
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
      <div className="flex items-center justify-between gap-2">
        <p className="menu-label" id="banned-label">
          <Ban size={14} aria-hidden />
          Yasaklı Uzantılar
        </p>
        <button type="button" onClick={onReset} title="Varsayılan listeye dön" className="menu-link">
          <RotateCcw size={12} aria-hidden />
          Sıfırla
        </button>
      </div>

      <p className="menu-note mb-2">
        Sistem/çalıştırılabilir dosyalar taranmaz. Varsayılan:{" "}
        {DEFAULT_BANNED_EXTENSIONS.slice(0, 6).join(", ")}…
      </p>

      <div className="ban-chips" role="list" aria-labelledby="banned-label">
        {extensions.length === 0 ? (
          <span className="menu-note">Liste boş: hiçbir uzantı engellenmiyor.</span>
        ) : (
          extensions.map((ext) => (
            <span key={ext} className="ban-chip" role="listitem">
              .{ext}
              <button
                type="button"
                onClick={() => onRemove(ext)}
                title={`${ext} listesinden çıkar`}
                aria-label={`.${ext} uzantısını listeden çıkar`}
              >
                <X size={12} aria-hidden />
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
          aria-label="Yasaklanacak uzantı"
          spellCheck={false}
          className="menu-input"
        />
        <button
          type="button"
          onClick={commit}
          disabled={!draft.trim()}
          title="Uzantıyı yasaklı listeye ekle"
          className="btn"
        >
          <Plus size={14} aria-hidden />
          Ekle
        </button>
      </div>
    </div>
  );
}

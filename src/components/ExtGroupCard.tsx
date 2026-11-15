import { useState } from "react";
import { Trash2, X } from "lucide-react";
import { splitExtensions } from "../lib/settingsCommands";
import type { ExtGroup } from "../types/settings";

interface ExtGroupCardProps {
  group: ExtGroup;
  /** Bu gruptaki uzanti -> zaten bagli oldugu diger grup. */
  conflicts: Map<string, string>;
  onChange: (group: ExtGroup) => void;
  onRemove: () => void;
  canRemove: boolean;
}

/** Bir uzanti grubu: ad ve uzantilar (× ile sil, "Uzanti ekle" ile yaz; Enter ya da virgul ekler). */
export default function ExtGroupCard({ group, conflicts, onChange, onRemove, canRemove }: ExtGroupCardProps) {
  const [text, setText] = useState("");
  const [adding, setAdding] = useState(false);
  const add = (raw: string) => {
    const fresh = splitExtensions(raw).filter(e => !group.extensions.includes(e));
    if (fresh.length) onChange({ ...group, extensions: [...group.extensions, ...fresh] });
    setText("");
  };
  const clashes = group.extensions.filter(e => conflicts.has(`${group.name}:${e}`));
  return <div className="rules-group">
    <div className="rules-group-name">
      <input value={group.name} aria-label="Grup adı" onChange={e => onChange({ ...group, name: e.target.value })} />
      <small>klasörüne gider</small>
    </div>
    <div className="rules-group-exts">
      <div className="rules-chips">
        {group.extensions.map(e => <span key={e} className={`rules-chip ${conflicts.has(`${group.name}:${e}`) ? "clash" : ""}`}>
          {e}<button type="button" aria-label={`.${e} uzantısını kaldır`} onClick={() => onChange({ ...group, extensions: group.extensions.filter(x => x !== e) })}><X size={13} /></button>
        </span>)}
        {adding
          ? <input id={`ext-${group.name}`} className="rules-chip-input" autoFocus value={text} placeholder="uzantı…" aria-label={`${group.name} grubuna uzantı ekle`}
            onChange={e => (/[,;\s]$/.test(e.target.value) ? add(e.target.value) : setText(e.target.value))}
            onKeyDown={e => {
              if (e.key === "Enter") { e.preventDefault(); add(text); }
              else if (e.key === "Escape") { setText(""); setAdding(false); }
              else if (e.key === "Backspace" && !text && group.extensions.length) onChange({ ...group, extensions: group.extensions.slice(0, -1) });
            }}
            onBlur={() => { if (text) add(text); setAdding(false); }} />
          : <button type="button" className="rules-add" onClick={() => setAdding(true)} aria-label={`${group.name} grubuna uzantı ekle`}>Uzantı ekle</button>}
      </div>
      {clashes.map(e => <p key={e} className="settings-error">“.{e}” zaten {conflicts.get(`${group.name}:${e}`)} grubunda. Bir uzantı yalnızca bir gruba bağlanabilir.</p>)}
      {!group.extensions.length && <p className="settings-error">Grupta en az bir uzantı olmalı.</p>}
    </div>
    <button type="button" className="btn" onClick={onRemove} disabled={!canRemove} aria-label={`${group.name} grubunu sil`}><Trash2 size={14} /></button>
  </div>;
}

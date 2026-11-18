import { useState, type KeyboardEvent } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Check, ChevronDown } from "lucide-react";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { leafName } from "../features/duplicates/groupPath";

export interface RootOption { label: string; path: string }

interface Props {
  /** null: acik klasor ("Bu klasör"). */
  value: string | null;
  currentDir: string | null;
  roots: RootOption[];
  folders: string[];
  external: string[];
  onChange: (path: string | null) => void;
  onPickExternal: (path: string) => void;
  /** Tetik kimligi (etiketin `htmlFor`u). */
  id?: string;
  /** Verilirse null secenegi bu adla her zaman secilebilir; "Bu klasör" ayri secenek olur (`currentDir` yolu). */
  nullLabel?: string;
  /** Kok seceneginin adi; varsayilan "Klasör A kökü · ad". */
  rootLabel?: (r: RootOption) => string;
  disabled?: boolean;
}

interface Item { value: string | null; label: string; disabled?: boolean }
interface Group { label: string | null; items: Item[] }

const lower = (s: string) => s.toLocaleLowerCase("tr");
const rel = (root: string, path: string) => path.slice(root.replace(/[\\/]+$/, "").length + 1);

/** Ok tuslariyla secenekler arasinda gezinme (liste kutusu). */
function moveFocus(e: KeyboardEvent<HTMLDivElement>) {
  if (e.key !== "ArrowDown" && e.key !== "ArrowUp" && e.key !== "Home" && e.key !== "End") return;
  e.preventDefault();
  const opts = [...e.currentTarget.querySelectorAll<HTMLButtonElement>("[role=option]:not(:disabled)")];
  const i = opts.indexOf(document.activeElement as HTMLButtonElement);
  const next = e.key === "Home" ? 0 : e.key === "End" ? opts.length - 1 : Math.min(opts.length - 1, Math.max(0, i + (e.key === "ArrowDown" ? 1 : -1)));
  opts[next]?.focus();
}

/** Ilk grup: `nullLabel` varsa null secenegi + (varsa) "Bu klasör"; yoksa null = "Bu klasör". */
function firstItems(currentDir: string | null, nullLabel?: string): Item[] {
  const here = currentDir ? `Bu klasör · ${leafName(currentDir)}` : null;
  if (nullLabel) return [{ value: null, label: nullLabel }, ...(currentDir && here ? [{ value: currentDir, label: here }] : [])];
  return [here ? { value: null, label: here } : { value: null, label: "Hedef klasör seçin…", disabled: true }];
}

/** Hedef klasor: bu klasor, kokler, kok icindeki klasorler, diger konumlar, "Başka bir klasör seç…".
 *  Yerel `<select>` yerine acilir liste: ekranin altinda kalinca yukari acilir. */
export default function TargetFolderSelect({ value, currentDir, roots, folders, external, onChange, onPickExternal, id = "selection-target", nullLabel, rootLabel, disabled }: Props) {
  const [isOpen, setOpen] = useState(false);
  const isCurrent = (p: string) => !!currentDir && lower(p) === lower(currentDir);
  const groups: Group[] = [
    { label: null, items: firstItems(currentDir, nullLabel) },
    { label: "Kökler", items: roots.filter(r => !isCurrent(r.path)).map(r => ({ value: r.path, label: rootLabel ? rootLabel(r) : `${r.label} kökü · ${leafName(r.path)}` })) },
    ...roots.map(r => ({
      label: `${leafName(r.path)} içindeki klasörler`,
      items: folders.filter(f => lower(f).startsWith(lower(r.path.replace(/[\\/]+$/, "")) + "\\") && !isCurrent(f)).map(f => ({ value: f, label: rel(r.path, f) })),
    })),
    { label: "Diğer konumlar", items: external.map(p => ({ value: p, label: p })) },
  ].filter(g => g.items.length);
  const selectedLabel = groups.flatMap(g => g.items).find(i => i.value === value)?.label ?? (value ?? "");

  const choose = (v: string | null) => { setOpen(false); onChange(v); };
  const pick = async () => {
    setOpen(false);
    const picked = await open({ directory: true, multiple: false, title: "Hedef klasörü seçin" });
    if (typeof picked === "string" && picked) onPickExternal(picked);
  };

  return <Popover open={isOpen} onOpenChange={setOpen}>
    <PopoverTrigger id={id} className="target-trigger" aria-label="Hedef klasör" aria-haspopup="listbox" disabled={disabled}>
      <span className="truncate">{selectedLabel}</span><ChevronDown size={15} aria-hidden />
    </PopoverTrigger>
    <PopoverContent collisionPadding={8} className="target-list" onKeyDown={moveFocus}
      onOpenAutoFocus={e => { e.preventDefault(); (e.currentTarget as HTMLElement).querySelector<HTMLButtonElement>("[aria-selected=true], [role=option]:not(:disabled)")?.focus(); }}>
      <div role="listbox" aria-label="Hedef klasör">
        {groups.map((g, gi) => <div key={g.label ?? gi} role="group" aria-label={g.label ?? undefined}>
          {g.label && <div className="target-group">{g.label}</div>}
          {g.items.map(i => <button key={i.value ?? ""} type="button" role="option" aria-selected={i.value === value} disabled={i.disabled}
            className={`target-option${g.label ? " nested" : ""}`} onClick={() => choose(i.value)} title={i.label}>
            <span className="truncate">{i.label}</span>{i.value === value && <Check size={14} aria-hidden />}
          </button>)}
        </div>)}
        <button type="button" role="option" aria-selected={false} className="target-option pick" onClick={() => void pick()}>Başka bir klasör seç…</button>
      </div>
    </PopoverContent>
  </Popover>;
}

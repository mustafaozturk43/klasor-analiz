import { useEffect, useId, useMemo, useRef, useState } from "react";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import { Search } from "lucide-react";
import { Dialog } from "./ui/dialog";
import { filterItems, nextIndex, type PaletteItem } from "../lib/palette";

interface Props { open: boolean; onOpenChange: (open: boolean) => void; items: PaletteItem[] }

/** Komut paleti (Ctrl+K): ekranlara, son klasorlere ve sik islemlere yazarak git; ok tuslari + Enter, Esc kapatir. */
export default function CommandPalette({ open, onOpenChange, items }: Props) {
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const listId = useId();
  const listRef = useRef<HTMLUListElement>(null);
  const shown = useMemo(() => filterItems(items, query), [items, query]);

  useEffect(() => { if (open) setQuery(""); }, [open]);
  // Sorgu degisince ilk secilebilir satira don.
  useEffect(() => { setActive(Math.max(0, shown.findIndex(i => !i.disabled))); }, [shown]);
  useEffect(() => { listRef.current?.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: "nearest" }); }, [active]);

  const pick = (item: PaletteItem | undefined) => {
    if (!item || item.disabled) return;
    onOpenChange(false);
    item.run();
  };
  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const i = nextIndex(shown, active, e.key === "ArrowDown" ? 1 : -1);
      if (i >= 0) setActive(i);
    } else if (e.key === "Enter") {
      e.preventDefault();
      pick(shown[active]);
    }
  };

  return <Dialog open={open} onOpenChange={onOpenChange}>
    <DialogPrimitive.Portal>
      <DialogPrimitive.Overlay className="fixed inset-0 z-50 bg-[var(--color-scrim)]" />
      <DialogPrimitive.Content aria-describedby={undefined} className="palette-box" onOpenAutoFocus={e => { e.preventDefault(); document.getElementById(`${listId}-input`)?.focus(); }}>
        <DialogPrimitive.Title className="sr-only">Komut paleti</DialogPrimitive.Title>
        <div className="palette-search"><Search size={16} aria-hidden="true" />
          <input id={`${listId}-input`} type="text" role="combobox" aria-expanded="true" aria-controls={listId} aria-activedescendant={shown[active] ? `${listId}-${active}` : undefined}
            aria-label="Komut ara" placeholder="Ekran, klasör ya da işlem ara…" autoComplete="off" value={query} onChange={e => setQuery(e.target.value)} onKeyDown={onKeyDown} /></div>
        <ul id={listId} ref={listRef} role="listbox" className="palette-list" aria-label="Sonuçlar">
          {shown.map((item, i) => <li key={item.id} id={`${listId}-${i}`} role="option" aria-selected={i === active} aria-disabled={item.disabled || undefined}
            className="palette-row" data-active={i === active || undefined} data-disabled={item.disabled || undefined}
            onMouseMove={() => { if (!item.disabled) setActive(i); }} onClick={() => pick(item)}>
            <span className="palette-group">{item.group}</span><span className="palette-label">{item.label}</span>{item.hint && <small className="palette-hint">{item.hint}</small>}
          </li>)}
          {!shown.length && <li className="palette-empty" role="presentation">“{query}” için sonuç yok.</li>}
        </ul>
        <p className="palette-foot">↑ ↓ gez · Enter seç · Esc kapat</p>
      </DialogPrimitive.Content>
    </DialogPrimitive.Portal>
  </Dialog>;
}

import { Fragment, useEffect, useState } from "react";
import { CalendarDays, ChevronDown, Copy, Folders, History, Inbox, LayoutGrid, PenLine, SlidersHorizontal, type LucideIcon } from "lucide-react";
import type { Destination } from "../types/app";
import type { AutomationStatus } from "../lib/automationCommands";
import { DESTINATIONS, OPS_LABEL, type DestinationDef } from "../lib/navigation";

const ICONS: Record<Destination, LucideIcon> = { today: Inbox, duplicates: Copy, map: LayoutGrid, rename: PenLine, dates: CalendarDays, rules: SlidersHorizontal, history: History };
const OPS = DESTINATIONS.filter(d => d.group === "ops");

interface Props {
  dest: Destination;
  setDest: (d: Destination) => void;
  /** Hedef basina rozet sayisi (varsa). */
  counts: Partial<Record<Destination, number>>;
  automation: AutomationStatus | null;
  onTogglePause: () => void;
  /** Ikon seridi: adlar gizlenir, grup hep acik kalir. */
  rail: boolean;
}

/** Koyu kenar cubugu: hedefler, acilir "Klasör-Dosya İşlemleri" grubu ve otomasyon durumu. */
export default function AppNav({ dest, setDest, counts, automation, onTogglePause, rail }: Props) {
  const [open, setOpen] = useState(true);
  const inOps = OPS.some(d => d.id === dest);
  // Baska bir yoldan (tepsi menusu, kisayol) grup icindeki bir ekrana gidilirse grup acilir.
  useEffect(() => { if (inOps) setOpen(true); }, [inOps]);
  const item = ({ id, label }: DestinationDef, size: number) => {
    const Icon = ICONS[id];
    const count = counts[id];
    return <button key={id} type="button" title={label} aria-current={dest === id ? "page" : undefined} onClick={() => setDest(id)}>
      <Icon size={size} aria-hidden="true" /><span className="nav-label">{label}</span>
      {count ? <span className="app-nav-count">{count}</span> : null}
    </button>;
  };
  return <nav className="app-nav" aria-label="Ana gezinme">
    <div className="app-wordmark">Smart File<br />Organizer</div>
    <div className="app-nav-list">
      {DESTINATIONS.map(d => d.group !== "ops" ? item(d, 20) : d !== OPS[0] ? null : <Fragment key="ops">
        <button type="button" className="app-nav-group" title={OPS_LABEL} aria-expanded={open || rail} onClick={() => setOpen(o => !o)}>
          <Folders size={20} aria-hidden="true" /><span className="nav-label">{OPS_LABEL}</span><ChevronDown size={16} className="app-nav-chev" aria-hidden="true" />
        </button>
        {(open || rail) && <div className="app-nav-sub">{OPS.map(o => item(o, 18))}</div>}
      </Fragment>)}
    </div>
    {automation?.enabled && <div className="app-nav-status">
      <strong>{automation.paused ? "Otomasyon duraklatıldı" : "Otomasyon açık"}</strong>
      <span>{automation.sources} kaynak izleniyor</span>
      <button type="button" className="btn" onClick={onTogglePause}>{automation.paused ? "Devam et" : "Duraklat"}</button>
    </div>}
  </nav>;
}

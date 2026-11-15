import { Check, Loader2 } from "lucide-react";
import type { Module } from "../types/app";
import type { DestinationDef } from "../lib/navigation";
import SubTabs from "./SubTabs";

interface Props {
  dest: DestinationDef;
  module: Module;
  onPick: (view: Module) => void;
  title: string;
  sub: string | null;
  /** Tarama durumu; null ise rozet gosterilmez. */
  scan: "idle" | "busy" | "done" | null;
  /** Sayi seridi: [etiket, deger, ipucu]; null ise serit gosterilmez. */
  stats: string[][] | null;
}

const BADGE = { idle: "Hazır", busy: "Tarama sürüyor", done: "Tarama tamamlandı" };

/** Ekran basligi karti: menu cubugu, baslik ve durum rozeti, sayi seridi. */
export default function ScreenHeader({ dest, module, onPick, title, sub, scan, stats }: Props) {
  return <section className="screen-card">
    <SubTabs dest={dest} active={module} onPick={onPick} />
    <div className="screen-head">
      <div><h1>{title}</h1>{sub && <p className="screen-sub">{sub}</p>}</div>
      {scan && <span className="screen-badge" role="status" data-kind={scan === "done" ? "ok" : scan === "busy" ? "busy" : undefined}>
        {scan === "done" ? <Check size={14} aria-hidden="true" /> : scan === "busy" ? <Loader2 size={14} className="animate-spin" aria-hidden="true" /> : null}{BADGE[scan]}
      </span>}
    </div>
    {stats && <dl className="stat-strip">{stats.map(([label, value, hint]) => <div key={label}><dt>{label}</dt><dd>{value}</dd><small>{hint}</small></div>)}</dl>}
  </section>;
}

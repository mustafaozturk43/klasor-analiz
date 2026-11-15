import { Activity, Columns2, Copy, File, LayoutGrid, ListChecks, ShieldAlert, SlidersHorizontal, type LucideIcon } from "lucide-react";
import type { Module } from "../types/app";
import type { DestinationDef } from "../lib/navigation";

const ICONS: Partial<Record<Module, LucideIcon>> = {
  groups: Copy, compare: Columns2, quarantine: ShieldAlert, analysis: LayoutGrid, files: File, health: Activity, settings: SlidersHorizontal, automation: ListChecks,
};

interface Props { dest: DestinationDef; active: Module; onPick: (view: Module) => void }

/** Menu cubugu: bir hedefin icindeki ekranlar arasinda gecis; tek ekranli hedeflerde gorunmez. */
export default function SubTabs({ dest, active, onPick }: Props) {
  if (dest.views.length < 2) return null;
  return <div className="sub-tabs" role="group" aria-label={`${dest.label} ekranları`}>
    {dest.views.map(v => {
      const Icon = ICONS[v.id];
      return <button key={v.id} type="button" aria-pressed={active === v.id} onClick={() => onPick(v.id)}>{Icon && <Icon size={16} aria-hidden="true" />}{v.label}</button>;
    })}
  </div>;
}

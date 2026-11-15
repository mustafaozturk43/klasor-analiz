import type { BrowserFilters, BrowserSort } from "../lib/browserView";
import { KIND_LABELS, KIND_ORDER, MONTHS, type KindChip } from "../lib/fileKinds";

interface Props {
  filters: BrowserFilters;
  onChange: (next: BrowserFilters) => void;
  years: string[];
  months: string[];
  counts: Record<KindChip, number>;
  /** Cekim tarihleri okunurken yil/ay kapali. */
  datesLocked: boolean;
}

/** Arama, yil, ay, siralama ve tur cipleri. */
export default function FileBrowserFilters({ filters, onChange, years, months, counts, datesLocked }: Props) {
  const set = (patch: Partial<BrowserFilters>) => onChange({ ...filters, ...patch });
  const year = years.includes(filters.year) ? filters.year : "all";
  const month = months.includes(filters.month) ? filters.month : "all";
  return <div className="browser-filters">
    <div className="inline-actions">
      <input type="search" className="browser-search" value={filters.q} onChange={e => set({ q: e.target.value })} placeholder="Dosya adında ara…" aria-label="Dosya adında ara" />
      <label className="browser-label">Yıl
        <select value={year} disabled={datesLocked} onChange={e => set({ year: e.target.value, month: "all" })} aria-label="Yıl">
          <option value="all">Tümü</option>{years.map(y => <option key={y} value={y}>{y}</option>)}
        </select></label>
      <label className="browser-label">Ay
        <select value={month} disabled={datesLocked} onChange={e => set({ month: e.target.value })} aria-label="Ay">
          <option value="all">Tümü</option>{months.map(m => <option key={m} value={m}>{MONTHS[Number(m) - 1]}</option>)}
        </select></label>
      <label className="browser-label">Sıra
        <select value={filters.sort} onChange={e => set({ sort: e.target.value as BrowserSort })} aria-label="Dosya sırası">
          <option value="date">Tarih (yeni → eski)</option><option value="name">Dosya adı</option><option value="size">Boyut ↓</option>
        </select></label>
    </div>
    <div className="inline-actions" role="group" aria-label="Dosya türü">
      {KIND_ORDER.map(k => <button key={k} type="button" className="browser-chip" aria-pressed={filters.kind === k} onClick={() => set({ kind: k })}>
        {KIND_LABELS[k]} <small>{counts[k]}</small></button>)}
    </div>
  </div>;
}

import { formatCount } from "../lib/format";

/** Tasima sirasinda "Taşınıyor · 120 / 400 (%30)" yazisi ve ilerleme cubugu. */
export default function MoveProgress({ done, total, label = "Taşınıyor" }: { done: number; total: number; label?: string }) {
  const pct = total ? Math.min(100, Math.round(done / total * 100)) : 0;
  return <div role="status" className="rules-organize-progress">
    <span>{label} · {formatCount(done)} / {formatCount(total)} (%{pct})</span>
    <div className="organize-progress"><span style={{ transform: `scaleX(${pct / 100})` }} /></div>
  </div>;
}

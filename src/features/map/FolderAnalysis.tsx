import { useEffect, type ReactNode } from "react";
import { ArrowUp } from "lucide-react";
import { useFolderStats } from "./useFolderStats";
import { baseName, formatBytes } from "../../lib/format";
import type { FolderStatsResult } from "../../types/file";
import CategoryBars from "./CategoryBars";
import FolderTreemap from "./FolderTreemap";
const COLORS = ["var(--color-chart-1)", "var(--color-chart-2)", "var(--color-chart-3)", "var(--color-chart-4)"];
const LABELS = ["Doküman", "Resim", "Arşiv", "Diğer"];

export function FolderSummary({ stats }: { stats: FolderStatsResult }) {
  const total = stats.categoryBytes.reduce((a, b) => a + b, 0);
  let end = 0;
  const gradient = stats.categoryBytes.map((bytes, i) => {
    const start = end; end += total ? bytes / total * 100 : 0;
    return `${COLORS[i]} ${start}% ${end}%`;
  }).join(",");
  return <div className="donut-layout">
    <div className="donut-chart" role="img" aria-label={`Toplam ${formatBytes(total)}; kategori değerleri yanındaki listede.`} style={{ background: total ? `conic-gradient(${gradient})` : "var(--color-paper-3)" }}><span>{formatBytes(total)}</span></div>
    <div className="chart-legend">{stats.categoryBytes.map((bytes, i) => <div className="legend-row" key={LABELS[i]}><span className="legend-dot" style={{ background: COLORS[i] }} />{LABELS[i]}<strong>{formatBytes(bytes)} · %{total ? Math.round(bytes / total * 100) : 0}</strong></div>)}</div>
  </div>;
}

interface Props {
  root: string; token: number; label: string; onOpen?: (path: string) => void; onUp?: () => void;
  /** Kök klasörün istatistiği hazır olunca (ekran başlığı için). */
  onStats?: (stats: FolderStatsResult | null) => void;
  /** Türe göre çubuklarının yanındaki ek bölüm (Temizlenecekler). */
  aside?: ReactNode;
}

/** Klasör haritası: kutu haritası, türe göre alan ve (varsa) temizlik özeti. */
export default function FolderAnalysis({ root, token, label, onOpen, onUp, onStats, aside }: Props) {
  const { stats, loading, error } = useFolderStats(root, token);
  useEffect(() => { onStats?.(stats); }, [stats, onStats]);
  return <section className="map-screen" aria-label={label}>
    <div className="map-bar">
      <p title={root}>{root ? <><b>{label}</b> {root}</> : "Üst çubuktan klasör seçerek başlayın."}</p>
      {onUp && <button className="btn" onClick={onUp}><ArrowUp size={15} /> Üst klasör</button>}
    </div>
    {loading ? <div className="empty-state" role="status">Klasör boyutları hesaplanıyor…</div> : error ? <div className="error-state" role="alert">{error}</div> : stats ? <>
      <FolderTreemap stats={stats} onOpen={onOpen} />
      <div className="map-lower"><CategoryBars stats={stats} />{aside}</div>
      <p className="hint">{baseName(root)}: boyut analizi tüm dosya türlerini kapsar; tarama filtrelerinden bağımsızdır, toplam {formatBytes(stats.rootTotalBytes)}.</p>
      {!!stats.warnings?.length && <div className="error-state" role="alert">{stats.warnings.map(w => <p key={w}>{w}</p>)}</div>}
    </> : <div className="empty-state">Analiz için bir klasör seçin.</div>}
  </section>;
}

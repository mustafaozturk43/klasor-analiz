import { formatBytes } from "../../lib/format";
import type { FolderStatsResult } from "../../types/file";

const LABELS = ["Doküman", "Resim", "Arşiv", "Diğer"];

/** Türe göre alan: her tür için orantılı bir çubuk ve boyut. */
export default function CategoryBars({ stats }: { stats: FolderStatsResult }) {
  const total = stats.categoryBytes.reduce((a, b) => a + b, 0);
  const rows = stats.categoryBytes.map((bytes, i) => ({ i, bytes })).sort((a, b) => b.bytes - a.bytes);
  return <section className="map-kinds" aria-label="Türe göre">
    <h2>Türe göre</h2>
    <ul>{rows.map(({ i, bytes }) => <li key={i}>
      <span>{LABELS[i]}</span>
      <span className="map-kind-track" role="img" aria-label={`${LABELS[i]}: yüzde ${total ? Math.round(bytes / total * 100) : 0}`}><span data-kind={i} style={{ width: `${total ? (bytes / total) * 100 : 0}%` }} /></span>
      <span className="map-kind-size">{formatBytes(bytes)}</span>
    </li>)}</ul>
  </section>;
}

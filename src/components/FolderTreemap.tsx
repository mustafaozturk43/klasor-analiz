import { useMemo } from "react";
import type { FolderStat, FolderStatsResult } from "../types/file";
import { formatBytes, formatCount } from "../lib/format";
import { squarify } from "../lib/treemap";

const LABELS = ["Doküman", "Resim", "Arşiv", "Diğer"] as const;
const W = 26;
const H = 10;
/** Toplamın bu oranının altındaki klasörler haritada okunmaz; altta şerit olarak listelenir. */
const MIN_SHARE = 0.03;

const kindOf = (f: FolderStat) => {
  const v = [f.documentBytes, f.imageBytes, f.archiveBytes, f.otherBytes];
  return v.indexOf(Math.max(...v));
};

interface Props { stats: FolderStatsResult; onOpen?: (path: string) => void }

/** Klasör haritası: kutu alanı klasör boyutuyla orantılı, rengi baskın dosya türüdür; kutuya tıklayınca içine girilir. */
export default function FolderTreemap({ stats, onOpen }: Props) {
  const total = stats.rootTotalBytes || 1;
  const { tiles, small, kinds } = useMemo(() => {
    const big = stats.folders.filter(f => f.totalBytes / total >= MIN_SHARE);
    const placed = squarify(big.map(f => ({ value: f.totalBytes, folder: f })), { x: 0, y: 0, w: W, h: H });
    return {
      tiles: placed,
      small: stats.folders.filter(f => f.totalBytes / total < MIN_SHARE && f.totalBytes > 0),
      kinds: new Set(big.concat(stats.folders).map(kindOf)),
    };
  }, [stats, total]);

  if (!stats.folders.length) return <p className="hint">Alt klasör bulunamadı.</p>;
  return <section className="map" aria-label="Klasör haritası">
    <ul className="map-legend">{LABELS.map((l, i) => kinds.has(i) && <li key={l}><span data-kind={i} />{l}</li>)}</ul>
    <div className="map-board">
      {tiles.map(t => {
        const share = t.folder.totalBytes / total;
        const size = share >= 0.15 ? "l" : share >= 0.07 ? "m" : "s";
        return <button key={t.folder.path} type="button" className="map-tile" data-kind={kindOf(t.folder)} data-size={size} disabled={!onOpen}
          title={`${t.folder.path}\n${formatBytes(t.folder.totalBytes)}, ${formatCount(t.folder.fileCount)} dosya`} onClick={() => onOpen?.(t.folder.path)}
          style={{ left: `${(t.x / W) * 100}%`, top: `${(t.y / H) * 100}%`, width: `${(t.w / W) * 100}%`, height: `${(t.h / H) * 100}%` }}>
          <span className="map-tile-name">{t.folder.name}</span>
          <span className="map-tile-size"><b>{formatBytes(t.folder.totalBytes)}</b>{size !== "s" && <small>{formatCount(t.folder.fileCount)} dosya, yüzde {Math.round(share * 100)}</small>}</span>
        </button>;
      })}
    </div>
    {small.length > 0 && <ul className="map-small" aria-label="Küçük klasörler">{small.map(f => <li key={f.path}>
      <button type="button" data-kind={kindOf(f)} disabled={!onOpen} onClick={() => onOpen?.(f.path)} title={f.path}><span>{f.name}</span><span>{formatBytes(f.totalBytes)}</span></button></li>)}</ul>}
    <p className="hint">Alanlar boyutla orantılı. Toplamın yüzde 3’ünden küçük klasörler okunabilsin diye şerit olarak gösterilir.</p>
  </section>;
}

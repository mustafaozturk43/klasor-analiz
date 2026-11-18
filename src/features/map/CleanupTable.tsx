import { Play } from "lucide-react";
import type { OrganizerState, Module } from "../../types/app";
import type { HealthState } from "./useHealth";
import { baseName, formatAge, formatBytes, formatCount } from "../../lib/format";

interface Props { app: OrganizerState; health: HealthState; onGo: (m: Module) => void }

/** Haritanın yanındaki "Temizlenecekler": klasör sağlığı raporunun özeti; ayrıntı ve işlem Klasör sağlığı ekranında. */
export default function CleanupTable({ app, health, onGo }: Props) {
  const r = health.report;
  const open = (tab: "largest" | "oldest" | "emptyFiles" | "emptyDirs") => { health.setTab(tab); onGo("health"); };
  const oldest = r?.oldest[0];
  const largest = r?.largest[0];
  return <section className="map-cleanup" aria-label="Klasör sağlığı">
    <h2>Temizlenecekler</h2>
    {!r ? <div className="map-cleanup-empty">
      <p>{app.rootPath ? "Boş, eski ve çok yer kaplayan öğeleri bulmak için raporu oluştur. Yalnızca dosya bilgileri okunur." : "Önce üstten bir ana klasör seç."}</p>
      <button type="button" className="btn" disabled={!app.rootPath || health.loading || app.isScanning || app.isMoving} onClick={() => void health.run()}><Play size={14} /> {health.loading ? "Taranıyor…" : "Raporu oluştur"}</button>
    </div> : <table>
      <thead><tr><th scope="col">Bulgu</th><th scope="col">Sayı</th><th scope="col"><span className="sr-only">İşlem</span></th></tr></thead>
      <tbody>
        <tr><td>Boş klasör</td><td>{formatCount(r.emptyDirs.length)}</td><td><button type="button" className="btn" disabled={!r.emptyDirs.length} onClick={() => open("emptyDirs")}>İncele</button></td></tr>
        <tr><td>Boş dosya</td><td>{formatCount(r.emptyFiles.length)}</td><td><button type="button" className="btn" disabled={!r.emptyFiles.length} onClick={() => open("emptyFiles")}>İncele</button></td></tr>
        <tr><td>{oldest ? `En eski dosya, ${formatAge(oldest.mtime)}` : "En eski dosya"}</td><td>{oldest ? "1" : "0"}</td><td><button type="button" className="btn" disabled={!oldest} onClick={() => oldest && void app.reveal(oldest.path)}>Göster</button></td></tr>
        <tr><td>{largest ? `En büyük dosya, ${baseName(largest.path)}` : "En büyük dosya"}</td><td className="mono">{largest ? formatBytes(largest.size) : "—"}</td>
          <td><button type="button" className="btn" disabled={!largest} onClick={() => largest && void app.reveal(largest.path)}>Göster</button></td></tr>
      </tbody>
    </table>}
  </section>;
}

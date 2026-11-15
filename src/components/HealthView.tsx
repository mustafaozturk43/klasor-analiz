import { ExternalLink, FolderX, HeartPulse, Play, ShieldAlert } from "lucide-react";
import type { OrganizerState } from "../types/app";
import type { HealthState, HealthTab } from "../hooks/useHealth";
import { baseName, formatAge, formatBytes, formatCount, formatDate } from "../lib/format";
import { relative } from "../lib/organizeView";
import VirtualRows from "./VirtualRows";

const TABS: { id: HealthTab; label: string; hint: string }[] = [
  { id: "largest", label: "En büyük dosyalar", hint: "En çok yer kaplayan ilk 200 dosya" },
  { id: "oldest", label: "En uzun süredir değişmeyen", hint: "Değişiklik tarihi en eski ilk 200 dosya" },
  { id: "emptyFiles", label: "0 baytlık dosyalar", hint: "İçi boş dosyalar (sistem dosyaları hariç)" },
  { id: "emptyDirs", label: "Boş klasörler", hint: "Hiç dosya ya da klasör içermeyenler" },
];

/** Klasor sagligi: buyuk/eski/bos dosyalar ve bos klasorler; secilenler karantinaya alinir veya kaldirilir. */
export default function HealthView({ app, health: h }: { app: OrganizerState; health: HealthState }) {
  const busy = app.isScanning || app.isMoving || h.loading;
  const r = h.report;
  const counts: Record<HealthTab, number> = { largest: r?.largest.length ?? 0, oldest: r?.oldest.length ?? 0, emptyFiles: r?.emptyFiles.length ?? 0, emptyDirs: r?.emptyDirs.length ?? 0 };
  const current = TABS.find(t => t.id === h.tab)!;
  const isDirs = h.tab === "emptyDirs";
  return <div className="organize-stack">
    <section className="panel">
      <div className="panel-head">
        <div><h2><HeartPulse size={18} /> Klasör sağlığı</h2><p>Yer kaplayan, unutulmuş ve boş öğeleri bulur. Yalnızca dosya bilgileri okunur; içerik okunmaz, hiçbir şey kendiliğinden değişmez.</p></div>
        <button className="btn primary" onClick={() => void h.run()} disabled={!app.rootPath || busy}><Play size={14} /> {h.loading ? "Taranıyor…" : r ? "Yeniden tara" : "Raporu oluştur"}</button>
      </div>
      {r ? <div className="organize-kpis">
        <div><span>Toplam dosya</span><strong>{formatCount(r.totalFiles)}</strong></div>
        <div><span>Toplam boyut</span><strong>{formatBytes(r.totalBytes)}</strong></div>
        <div><span>İlk 200'ün boyutu</span><strong>{formatBytes(r.largest.reduce((s, f) => s + f.size, 0))}</strong></div>
        <div><span>0 baytlık dosya</span><strong>{formatCount(r.emptyFiles.length)}</strong></div>
        <div><span>Boş klasör</span><strong>{formatCount(r.emptyDirs.length)}</strong></div>
      </div> : <div className="empty-state">{app.rootPath ? "Raporu oluşturmak için düğmeye basın. Alt klasör ve hariç klasör ayarları üstteki tarama kapsamından gelir." : "Önce üstten bir ana klasör seçin."}</div>}
    </section>

    {r && <section className="panel">
      <nav className="module-tabs m-4 mb-0" aria-label="Rapor bölümü">{TABS.map(t => <button key={t.id} className={h.tab === t.id ? "active" : ""} aria-current={h.tab === t.id ? "page" : undefined} onClick={() => h.setTab(t.id)}>{t.label} <span className="badge">{formatCount(counts[t.id])}</span>
        </button>)}</nav>
      <div className="panel-body inline-actions">
        <label className="inline-actions text-xs"><input type="checkbox" checked={!!h.rows.length && h.selected.size === h.rows.length} onChange={h.toggleAll} disabled={!h.rows.length || busy} /> Tümünü seç</label>
        <span className="hint mr-auto">{current.hint} · {formatCount(h.selected.size)} seçili</span>
        <button className="btn" disabled={!h.selected.size || busy} onClick={() => void h.act()}>{isDirs ? <><FolderX size={14} /> Seçili klasörleri kaldır</> : <><ShieldAlert size={14} /> Seçilenleri karantinaya al</>}</button>
      </div>
      {h.rows.length ? <VirtualRows items={h.rows} estimate={56} itemKey={f => f.path} render={f => <div className={`organize-row health-row ${h.selected.has(f.path) ? "selected" : ""}`}>
        <input type="checkbox" checked={h.selected.has(f.path)} onChange={() => h.toggle(f.path)} aria-label={`${baseName(f.path)} seç`} disabled={busy} />
        <div className="min-w-0"><span className="name block" title={f.path}>{baseName(f.path)}</span><small title={f.path}>{relative(r.rootPath, f.path)}</small></div>
        <div className="target text-right">{isDirs ? "boş klasör" : h.tab === "oldest" ? `${formatDate(f.mtime)} · ${formatAge(f.mtime)}` : formatBytes(f.size)}</div>
        <button className="btn" onClick={() => void app.reveal(f.path)} aria-label={`${baseName(f.path)} klasörde göster`}><ExternalLink size={14} /></button>
      </div>} /> : <div className="empty-state">Bu bölümde öğe yok.</div>}
      {!!r.warnings.length && <details className="panel-body text-xs"><summary>Okunamayan öğeler ({r.warnings.length})</summary>{r.warnings.slice(0, 50).map((w, i) => <p className="hint break-all" key={i}>{w}</p>)}</details>}
    </section>}
  </div>;
}

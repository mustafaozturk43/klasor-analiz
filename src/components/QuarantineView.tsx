import { ArchiveRestore, ExternalLink, RefreshCw, ShieldCheck, Trash2 } from "lucide-react";
import type { OrganizerState } from "../types/app";
import { useQuarantine } from "../hooks/useQuarantine";
import { baseName, formatBytes, formatCount, formatDate } from "../lib/format";
import { relative } from "../lib/organizeView";
import VirtualRows from "./VirtualRows";

/** Karantina yonetimi: kopyalari gor, asil yerine geri yukle veya Geri Donusum Kutusu'na gonder. */
export default function QuarantineView({ app, active }: { app: OrganizerState; active: boolean }) {
  const q = useQuarantine(app, active);
  const busy = app.isScanning || app.isMoving || q.loading;
  const l = q.listing;
  return <section className="panel">
    <div className="panel-head">
      <div><h2><ShieldCheck size={18} /> Karantina</h2>
        <p>{l ? `${formatCount(l.items.length)} dosya · ${formatBytes(l.totalBytes)} · ${l.quarantinePath}` : "Taşınan kopyalar burada bekler; siz göndermedikçe yerinde kalır."}</p></div>
      <div className="inline-actions">
        <button className="btn" onClick={() => void q.load()} disabled={!app.rootPath || busy} aria-label="Karantinayı yenile"><RefreshCw size={14} /></button>
        <button className="btn" onClick={() => void app.clearQuarantineFolder()} disabled={!l?.items.length || busy}><Trash2 size={14} /> Tümünü temizle…</button>
      </div>
    </div>
    {!app.rootPath ? <div className="empty-state">Önce üstten bir ana klasör seçin.</div>
      : !l?.items.length ? <div className="empty-state">{q.loading ? "Karantina okunuyor…" : "Karantina boş."}</div>
      : <>
        <div className="panel-body inline-actions">
          <label className="inline-actions text-xs"><input type="checkbox" checked={!!q.items.length && q.selected.size === q.items.length} onChange={q.toggleAll} disabled={busy} /> Tümünü seç</label>
          <select value={q.group} onChange={e => q.setGroup(e.target.value)} aria-label="Karantina klasörü">
            <option value="all">Tüm klasörler</option>
            {q.groups.map(g => <option key={g} value={g}>{g || "(karantina kökü)"}</option>)}
          </select>
          <button className="btn" disabled={!q.oldCount || busy} onClick={q.selectOld} title="30 günden önce karantinaya alınan kopyaları seçer; göndermek için “Kutuya gönder”e basın.">
            30 günden eskileri seç ({formatCount(q.oldCount)})</button>
          <span className="hint mr-auto">{formatCount(q.selected.size)} seçili · {formatBytes(q.selectedBytes)}</span>
          <button className="btn" disabled={!q.selected.size || busy} onClick={() => void q.restore()}><ArchiveRestore size={14} /> Geri yükle</button>
          <button className="btn" disabled={!q.selected.size || busy} onClick={() => void q.remove()}><Trash2 size={14} /> Kutuya gönder</button>
        </div>
        <VirtualRows items={q.items} estimate={60} itemKey={i => i.path} render={i => <div className={`organize-row health-row ${q.selected.has(i.path) ? "selected" : ""}`}>
          <input type="checkbox" checked={q.selected.has(i.path)} onChange={() => q.toggle(i.path)} aria-label={`${baseName(i.path)} seç`} disabled={busy} />
          <div className="min-w-0"><span className="name block" title={i.path}>{baseName(i.path)}</span>
            <small title={i.original ?? undefined}>{i.group || "karantina kökü"} · {i.original ? `asıl yeri: ${relative(app.rootPath, i.original)}` : "asıl yer kaydı yok; klasör adıyla ana klasöre döner"}</small></div>
          <div className="target text-right">{formatBytes(i.size)} · {formatDate(i.mtime)}</div>
          <button className="btn" onClick={() => void app.reveal(i.path)} aria-label={`${baseName(i.path)} klasörde göster`}><ExternalLink size={14} /></button>
        </div>} />
      </>}
  </section>;
}

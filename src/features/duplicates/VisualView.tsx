import { useEffect, useState } from "react";
import { ExternalLink, Images, Play, ShieldAlert } from "lucide-react";
import { baseName, formatBytes, formatCount } from "../../lib/format";
import type { OrganizerState } from "../../types/app";
import { useShellThumbs } from "../map/useShellThumbs";
import { SENSITIVITY, similarityPct, type VisualGroup } from "./visualCommands";
import { useVisual } from "./useVisual";

const PAGE = 40;

/** Benzer gorseller: ayni gorselin yeniden boyutlandirilmis/sikistirilmis kopyalari gruplanir; secilenler karantinaya alinir. */
export default function VisualView({ app }: { app: OrganizerState }) {
  const v = useVisual(app);
  const thumbs = useShellThumbs(128);
  const [shown, setShown] = useState(PAGE);
  const busy = app.isScanning || app.isMoving || v.loading;
  const r = v.report;
  const groups: VisualGroup[] = r?.groups.slice(0, shown) ?? [];
  const { request } = thumbs;
  useEffect(() => { setShown(PAGE); }, [r]);
  useEffect(() => { request(groups.flatMap(g => g.files.map(f => f.path))); }, [groups, request]);
  const hint = SENSITIVITY.find(s => s.id === v.distance)?.hint;
  return <div className="organize-stack">
    <section className="panel">
      <div className="panel-head">
        <div><h2><Images size={18} /> Benzer görseller</h2>
          <p>Görsel parmak iziyle aynı fotoğrafın yeniden boyutlandırılmış, sıkıştırılmış ya da hafifçe düzenlenmiş kopyalarını bulur. İçerik okunur ama hiçbir dosya değişmez.</p></div>
        <button className="btn primary" onClick={() => void v.run()} disabled={!app.rootPath || busy}><Play size={14} /> {v.loading ? "Taranıyor…" : r ? "Yeniden tara" : "Benzerleri bul"}</button>
      </div>
      <div className="panel-body">
        <div className="naming-seg" role="group" aria-label="Hassasiyet">
          {SENSITIVITY.map(s => <button key={s.id} type="button" aria-pressed={v.distance === s.id} title={s.hint} disabled={busy} onClick={() => v.setDistance(s.id)}>{s.label}</button>)}
        </div>
        <p className="hint">{hint} Desteklenen biçimler: jpg, png, webp, gif, bmp, tiff (heic desteklenmez).</p>
        {v.progress && <p className="hint" role="status">Görseller okunuyor{v.progress.total ? ` · ${formatCount(v.progress.done)} / ${formatCount(v.progress.total)}` : "…"}</p>}
      </div>
      {!r ? <div className="empty-state">{app.rootPath ? "Taramak için düğmeye basın. Alt klasör ve hariç klasör ayarları üstteki tarama kapsamından gelir." : "Önce üstten bir klasör seçin."}</div>
        : <div className="organize-kpis">
          <div><span>Taranan görsel</span><strong>{formatCount(r.scanned)}</strong></div>
          <div><span>Benzer grup</span><strong>{formatCount(r.groups.length)}</strong></div>
          <div><span>Okunamayan</span><strong>{formatCount(r.unreadable)}</strong></div>
          <div><span>Seçili</span><strong>{formatCount(v.selected.size)} · {formatBytes(v.selectedBytes)}</strong></div>
        </div>}
    </section>
    {r && (r.groups.length ? <section className="panel">
      <div className="panel-body inline-actions">
        <span className="hint mr-auto">Her grupta en yüksek çözünürlüklü görsel önerilen olarak kalır; kutuyu işaretlediklerin karantinaya gider. Her grupta en az bir görsel yerinde kalmalı.</span>
        <button className="btn" disabled={!v.selected.size || busy} onClick={() => void v.act()}><ShieldAlert size={14} /> Seçilenleri karantinaya al</button>
      </div>
      <ul className="visual-groups">
        {groups.map(g => <li key={g.id} className="visual-group">
          <h3>{formatCount(g.files.length)} benzer görsel</h3>
          <div className="visual-files">
            {g.files.map((f, i) => {
              const thumb = thumbs.thumbOf(f.path);
              return <div key={f.path} className={`visual-file ${v.selected.has(f.path) ? "selected" : ""}`}>
                <div className="browser-thumb">{thumb ? <img src={thumb} alt="" /> : <Images size={18} aria-hidden="true" />}</div>
                <label className="visual-pick"><input type="checkbox" checked={v.selected.has(f.path)} disabled={busy} onChange={() => v.toggle(g.id, f.path)} aria-label={`${baseName(f.path)} karantinaya al`} />
                  {i === 0 ? "Önerilen: tut" : "Karantinaya al"}</label>
                <span className="name" title={f.path}>{baseName(f.path)}</span>
                <small>{f.width}×{f.height} · {formatBytes(f.size)}{i ? ` · %${similarityPct(f.distance)} benzer` : ""}</small>
                <button className="btn" onClick={() => void app.reveal(f.path)} aria-label={`${baseName(f.path)} klasörde göster`}><ExternalLink size={14} /></button>
              </div>;
            })}
          </div>
        </li>)}
      </ul>
      {r.groups.length > shown && <div className="panel-body"><button className="btn" onClick={() => setShown(n => n + PAGE)}>Daha fazla göster ({formatCount(r.groups.length - shown)} grup daha)</button></div>}
    </section> : <section className="panel"><div className="empty-state">Bu hassasiyette benzer görsel bulunamadı.</div></section>)}
  </div>;
}

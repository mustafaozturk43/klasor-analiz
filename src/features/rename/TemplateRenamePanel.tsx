import { ArrowRight, PenLine, Play } from "lucide-react";
import type { OrganizerState } from "../../types/app";
import { TOKENS, useTemplateRename } from "./useTemplateRename";
import { baseName, formatCount } from "../../lib/format";
import { relative } from "../rules/organizeView";
import VirtualRows from "../../components/VirtualRows";

/** Sablonla toplu adlandirma: alanlar, onizleme (eski -> yeni) ve uygulama. */
export default function TemplateRenamePanel({ app }: { app: OrganizerState }) {
  const t = useTemplateRename(app);
  const locked = app.isScanning || (app.isMoving && !t.busy) || t.busy;
  const changing = t.plan?.entries.filter(e => e.to).length ?? 0;
  const skipped = (t.plan?.entries.length ?? 0) - changing;
  return <section className="panel">
    <div className="panel-head">
      <div><h2><PenLine size={18} /> Şablonla toplu adlandırma</h2>
        <p>Ana klasördeki dosyalara şablona göre yeni ad verir; uzantı korunur, dosyalar yerinde kalır. Alt klasör ve hariç klasör ayarları üstteki tarama kapsamından gelir. Tarih, dosyanın son değişiklik tarihidir.</p></div>
      <button className="btn primary" onClick={() => void t.preview()} disabled={!app.rootPath || locked || !t.template.trim()}><Play size={14} /> Önizle</button>
    </div>
    <fieldset className="panel-body flex flex-col gap-3" disabled={locked}>
      <label className="flex flex-col gap-1.5 text-xs">Şablon
        <input type="text" value={t.template} onChange={e => t.setTemplate(e.target.value)} aria-label="Ad şablonu" spellCheck={false} placeholder="örn. Fatura_{yıl}-{ay}_{sıra}" />
      </label>
      <div className="inline-actions">{TOKENS.map(([token, hint]) => <button type="button" key={token} className="btn" title={hint} onClick={() => t.insertToken(token)}>{token}</button>)}</div>
      <div className="inline-actions text-xs">
        <label className="inline-actions">Uzantılar <input type="text" value={t.extText} onChange={e => t.setExtText(e.target.value)} aria-label="Adlandırılacak uzantılar" placeholder="boş = tümü" /></label>
        <label className="inline-actions">Başlangıç <input type="number" min={0} className="w-20" value={t.start} onChange={e => t.setStart(Math.max(0, Number(e.target.value) || 0))} aria-label="Sıra başlangıcı" /></label>
        <label className="inline-actions">Basamak <select value={t.pad} onChange={e => t.setPad(Number(e.target.value))} aria-label="Sıra basamak sayısı">{[1, 2, 3, 4].map(n => <option key={n} value={n}>{"0".repeat(n - 1)}1</option>)}</select></label>
        <label className="inline-actions">Sıralama <select value={t.sort} onChange={e => t.setSort(e.target.value as "name" | "date")} aria-label="Sıra numarası düzeni"><option value="date">Tarihe göre (eskiden yeniye)</option><option value="name">Ada göre</option></select></label>
      </div>
    </fieldset>
    {t.plan && <>
      <div className="panel-body inline-actions pt-0">
        <span className="hint mr-auto">{formatCount(changing)} dosyanın adı değişecek{skipped ? ` · ${formatCount(skipped)} dosya atlanacak` : ""}.</span>
        <button className="btn" onClick={t.reset} disabled={t.busy}>Vazgeç</button>
        <button className="btn primary" onClick={() => void t.apply()} disabled={!changing || locked}>{formatCount(changing)} dosyayı adlandır</button>
      </div>
      <VirtualRows items={t.plan.entries} estimate={56} itemKey={e => e.from} render={e => <div className="organize-row">
        <div className="min-w-0"><span className="name block" title={e.from}>{baseName(e.from)}</span><small title={e.from}>{relative(app.rootPath, e.from)}</small></div>
        <div className="target inline-actions"><ArrowRight size={13} aria-hidden="true" /> {e.to ? baseName(e.to) : "aynı kalır"}</div>
        {e.warning ? <span className={`badge ${e.to ? "warn" : "bad"}`} title={e.warning}>{e.to ? "Numaralandı" : "Atlanacak"}</span> : <span className="badge ok">Yeni ad</span>}
      </div>} />
      {skipped > 0 && <details className="panel-body text-xs"><summary>Atlanma nedenleri</summary>{t.plan.entries.filter(e => !e.to).slice(0, 50).map(e => <p className="hint break-all" key={e.from}>{baseName(e.from)}: {e.warning}</p>)}</details>}
    </>}
  </section>;
}

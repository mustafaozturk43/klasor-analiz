import { useState } from "react";
import { ListTree, Plus, RotateCcw, Trash2 } from "lucide-react";
import type { OrganizeSettings } from "./useOrganizeSettings";
import type { RuleDef } from "../../types/organize";
import { parseExtensions } from "./organizeSettings";

/**
 * Tur klasoru kurallari: hangi uzanti hangi kategori/tur klasorune gider.
 * Uzanti kutusu odak kaybedince kaydedilir (yazarken virgul silinmesin diye).
 */
export default function OrganizeRulesEditor({ settings, disabled }: { settings: OrganizeSettings; disabled: boolean }) {
  const { rules, isCustomRules, rulesError, setRules, resetRules } = settings;
  const [open, setOpen] = useState(false);
  const update = (i: number, patch: Partial<RuleDef>) => setRules(rules.map((r, j) => (j === i ? { ...r, ...patch } : r)));
  const remove = (i: number) => setRules(rules.filter((_, j) => j !== i));
  const add = () => setRules([...rules, { category: "Diğer", kind: "Yeni klasör", extensions: [] }]);
  const categories = [...new Set(rules.map(r => r.category))];

  return <section className="panel">
    <button type="button" className="panel-head w-full text-left" aria-expanded={open} aria-controls="organize-rules" onClick={() => setOpen(v => !v)}>
      <div><h2><ListTree size={18} /> Tür klasörü kuralları</h2>
        <p>{rules.length} kural · {isCustomRules ? "özelleştirilmiş" : "varsayılan"} · Her uzantı yalnızca bir klasöre bağlanabilir.</p></div>
      <span className="btn">{open ? "Gizle" : "Düzenle"}</span>
    </button>
    {rulesError && <p className="error-state" role="alert">{rulesError} Düzeltilene kadar önizleme yapılamaz.</p>}
    <fieldset id="organize-rules" hidden={!open} disabled={disabled}>
      <div className="organize-rule organize-rule-head" aria-hidden="true"><span>Kategori klasörü</span><span>Tür klasörü</span><span>Uzantılar (virgülle)</span><span /></div>
      <datalist id="organize-categories">{categories.map(c => <option key={c} value={c} />)}</datalist>
      {rules.map((r, i) => <div className="organize-rule" key={i}>
        <input type="text" list="organize-categories" aria-label={`${i + 1}. kural kategori klasörü`} value={r.category} onChange={e => update(i, { category: e.target.value })} />
        <input type="text" aria-label={`${i + 1}. kural tür klasörü`} value={r.kind} onChange={e => update(i, { kind: e.target.value })} />
        <input type="text" aria-label={`${r.kind} uzantıları`} key={`${i}:${r.extensions.join(",")}`} defaultValue={r.extensions.join(", ")} placeholder="örn: mp4, mov" spellCheck={false}
          onBlur={e => { const next = parseExtensions(e.target.value); if (next.join() !== r.extensions.join()) update(i, { extensions: next }); }}
          onKeyDown={e => { if (e.key === "Enter") e.currentTarget.blur(); }} />
        <button type="button" className="btn" onClick={() => remove(i)} aria-label={`${r.kind} kuralını sil`}><Trash2 size={14} /></button>
      </div>)}
      <div className="panel-body inline-actions">
        <button type="button" className="btn" onClick={add}><Plus size={14} /> Kural ekle</button>
        <button type="button" className="btn" onClick={resetRules} disabled={!isCustomRules}><RotateCcw size={14} /> Varsayılana dön</button>
        <span className="hint">Kurallarda olmayan uzantılar “Diğer dosyalar” açık değilse yerinde kalır.</span>
      </div>
    </fieldset>
  </section>;
}

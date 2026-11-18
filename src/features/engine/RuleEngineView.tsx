import { Plus, Workflow } from "lucide-react";
import type { OrganizerState } from "../../types/app";
import Switch from "../../components/Switch";
import RuleCard from "./RuleCard";
import RulePreview from "./RulePreview";
import { describeConditions } from "./ruleModel";
import { useRuleEngine } from "./useRuleEngine";

/** Kural motoru: seçili klasörde kullanıcı kurallarını önizler, onayla uygular. Gelen Kutusu'na dokunmaz. */
export default function RuleEngineView({ app, active }: { app: OrganizerState; active: boolean }) {
  const eng = useRuleEngine(app, active);
  const locked = app.isScanning || app.isMoving || eng.busy;
  return <div className="naming-layout">
    <section className="panel naming-options rule-engine-options">
      <div className="panel-head"><h2><Workflow size={18} /> Kurallar</h2></div>
      <fieldset disabled={locked}>
        <div className="naming-opt"><b>Kapsam</b>
          <label><Switch checked={eng.recursive} onChange={eng.setRecursive} label="Alt klasörlerdeki dosyalara da uygula" /> Alt klasörlerdeki dosyalara da</label>
          <p className="hint">Kapalıyken yalnızca seçili klasörün kendi dosyalarına bakılır. Yıl\Ay eylemi dosyanın değişiklik tarihini kullanır.</p></div>
        <div className="rule-engine-body">
        {eng.rules.length === 0 && <p className="hint rule-empty">Henüz kural yok. “Kural ekle” ile başlayın: her kuralın en az bir koşulu olmalı.</p>}
        <ul className="rule-list">
          {eng.rules.map((r, i) => <RuleCard key={i} rule={r} index={i} count={eng.rules.length}
            onChange={patch => eng.patchRule(i, patch)} onMove={by => eng.moveRule(i, by)} onRemove={() => eng.removeRule(i)} />)}
        </ul>
        <button type="button" className="btn primary rule-add" onClick={eng.addRule}><Plus size={14} /> Kural ekle</button>
        {eng.rules.some(r => r.enabled) && <p className="hint">{eng.rules.filter(r => r.enabled).map(r => `${r.name}: ${describeConditions(r) || "koşul yok"}`).join(" → ")}</p>}
        </div>
      </fieldset>
    </section>
    <RulePreview app={app} eng={eng} rootPath={app.rootPath} locked={locked} />
  </div>;
}

import { ListChecks, Undo2 } from "lucide-react";
import MoveProgress from "../../components/MoveProgress";
import { formatCount } from "../../lib/format";
import type { RuleEngineState } from "./useRuleEngine";
import { ACTION_LABELS } from "./ruleModel";

interface Props { eng: RuleEngineState; rootPath: string; locked: boolean }

/** Onizleme: kural basina dosya sayisi ve ornek adlar; altta uygulama, ilerleme ve geri alma. */
export default function RulePreview({ eng, rootPath, locked }: Props) {
  const { plan, error, loading } = eng;
  const n = plan?.entries.length ?? 0;
  const hasRules = eng.rules.some(r => r.enabled);
  return <section className="panel naming-preview">
    <div className="panel-head"><h2><ListChecks size={18} /> Önizleme</h2><span className="hint">Kurallar yukarıdan aşağıya denenir; ilk eşleşen kural kazanır.</span></div>
    {!rootPath ? <div className="empty-state">Kuralları denemek için yukarıdan bir klasör seçin.</div>
      : !hasRules ? <div className="empty-state">Bir kural ekleyin ve etkinleştirin.</div>
      : error ? <div className="error-state" role="alert">{error}</div>
      : !plan ? <div className="empty-state" role="status">Dosyalar kurallarla karşılaştırılıyor…</div>
      : <ul className="date-group-list">
        {plan.counts.map(c => <li key={c.rule} className="rule-count-row" data-trash={c.action === "trash" || undefined}>
          <span className="naming-name">{c.rule}</span>
          <span className="hint">{ACTION_LABELS[c.action]}</span>
          <span className="date-group-samples" title={c.samples.join(", ")}>{c.samples.join(", ")}{c.count > c.samples.length ? ` +${formatCount(c.count - c.samples.length)}` : ""}</span>
          <span className="badge">{formatCount(c.count)} dosya</span>
        </li>)}
      </ul>}
    {eng.moving && <MoveProgress done={eng.moving.done} total={eng.moving.total} label="Uygulanıyor" />}
    <div className="naming-foot">
      <span role="status">{plan ? `${formatCount(plan.scanned)} dosya bakıldı · ${formatCount(n)} işlem · ${formatCount(plan.unmatched)} dosya hiçbir kurala uymadı`
        + `${plan.inPlace ? ` · ${formatCount(plan.inPlace)} zaten yerinde` : ""}.` : ""}{loading && plan ? " Güncelleniyor…" : ""}</span>
      {eng.canUndo && <button type="button" className="btn" onClick={() => void eng.undo()} disabled={locked}><Undo2 size={14} /> Geri al</button>}
      <button type="button" className="btn primary naming-go" onClick={() => void eng.apply()} disabled={locked || loading || !n}>
        {n ? `${formatCount(n)} işlemi uygula` : "Uygulanacak işlem yok"}</button>
    </div>
  </section>;
}

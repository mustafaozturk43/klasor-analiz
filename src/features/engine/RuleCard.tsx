import { ArrowDown, ArrowUp, Trash2 } from "lucide-react";
import Switch from "../../components/Switch";
import { splitExtensions } from "../rules/settingsCommands";
import type { Rule, RuleAction } from "./ruleEngineCommands";
import { ACTION_LABELS, parseOptionalNumber } from "./ruleModel";

interface Props {
  rule: Rule;
  index: number;
  count: number;
  onChange: (patch: Partial<Rule>) => void;
  onMove: (by: -1 | 1) => void;
  onRemove: () => void;
}

const ACTIONS = Object.keys(ACTION_LABELS) as RuleAction[];

/** Tek kural: kosullar (uzanti, ad, boyut, yas) ve eylem. Kurallar yukaridan asagiya denenir, ilk eslesen kazanir. */
export default function RuleCard({ rule: r, index, count, onChange, onMove, onRemove }: Props) {
  const num = (key: "minMb" | "maxMb" | "minAgeDays") => (e: React.ChangeEvent<HTMLInputElement>) => onChange({ [key]: parseOptionalNumber(e.target.value) });
  return <li className="rule-card" data-off={!r.enabled || undefined}>
    <div className="rule-card-head">
      <span className="badge">{index + 1}</span>
      <input type="text" value={r.name} onChange={e => onChange({ name: e.target.value })} aria-label="Kural adı" autoComplete="off" />
      <Switch checked={r.enabled} onChange={v => onChange({ enabled: v })} label={`${r.name} kuralı etkin`} />
      <button type="button" className="btn icon" onClick={() => onMove(-1)} disabled={index === 0} aria-label="Yukarı taşı"><ArrowUp size={14} /></button>
      <button type="button" className="btn icon" onClick={() => onMove(1)} disabled={index === count - 1} aria-label="Aşağı taşı"><ArrowDown size={14} /></button>
      <button type="button" className="btn icon" onClick={onRemove} aria-label="Kuralı sil"><Trash2 size={14} /></button>
    </div>
    <div className="rule-card-grid">
      <label>Uzantılar
        <input type="text" defaultValue={r.exts.join(", ")} key={r.exts.join(",")} placeholder="pdf, docx" autoComplete="off"
          onBlur={e => onChange({ exts: splitExtensions(e.target.value) })} /></label>
      <label>Adı içerir
        <input type="text" value={r.nameContains} onChange={e => onChange({ nameContains: e.target.value })} placeholder="fatura" autoComplete="off" /></label>
      <label>En az MB
        <input type="number" min={0} step="any" value={r.minMb ?? ""} onChange={num("minMb")} /></label>
      <label>En çok MB
        <input type="number" min={0} step="any" value={r.maxMb ?? ""} onChange={num("maxMb")} /></label>
      <label>En az kaç gün eski
        <input type="number" min={0} step={1} value={r.minAgeDays ?? ""} onChange={num("minAgeDays")} /></label>
      <label>Eylem
        <select value={r.action} onChange={e => onChange({ action: e.target.value as RuleAction })}>
          {ACTIONS.map(a => <option key={a} value={a}>{ACTION_LABELS[a]}</option>)}
        </select></label>
      {r.action !== "trash" && <label className="rule-target">Hedef klasör (seçili klasörün altında)
        <input type="text" value={r.target} onChange={e => onChange({ target: e.target.value })} placeholder="Faturalar" autoComplete="off" /></label>}
    </div>
    {r.action === "trash" && <p className="hint rule-warn">Dosyalar Geri Dönüşüm Kutusu'na gider. İşlem Geçmişi'nden geri alınamaz; ayrıca onay istenir.</p>}
  </li>;
}

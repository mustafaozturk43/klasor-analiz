import { FolderTree } from "lucide-react";
import type { OrganizePlan } from "../types/organize";
import { formatBytes, formatCount } from "../lib/format";
import { byCategory, countPlan, sourceFolders } from "../lib/organizeView";

const MAX_SOURCES = 10;

/** Onizleme ozeti: sayaclar, tur cubuklari ve once/sonra klasor yapisi. */
export default function OrganizeSummary({ plan, flat }: { plan: OrganizePlan; flat: boolean }) {
  const counts = countPlan(plan);
  const groups = byCategory(plan.kinds);
  const max = Math.max(1, ...plan.kinds.map(k => k.count));
  const sources = sourceFolders(plan);
  const kpis: [string, number][] = [
    ["Taşınacak", counts.move], ["Zaten yerinde", counts.inPlace], ["Karantinaya", counts.quarantine],
    ["Numaralanacak", counts.renamed], ["Atlanacak", counts.skip],
  ];
  return <section className="panel">
    <div className="panel-head"><div><h2><FolderTree size={18} /> Önizleme</h2>
      <p>{formatCount(plan.totalFiles)} dosya tarandı{plan.protectedDirs.length ? ` · ${plan.protectedDirs.length} proje klasörü korundu` : ""}. Sayılar uygulanacak planın kendisidir.</p></div></div>
    <div className="organize-kpis">{kpis.map(([label, value]) => <div key={label}><span>{label}</span><strong>{formatCount(value)}</strong></div>)}</div>
    <div className="two-column panel-body pt-0">
      <div className="chart-surface">
        <h3>Bulunan türler</h3>
        {groups.length ? groups.map(g => <div key={g.category}>
          <p className="organize-group-label">{g.category} · {formatCount(g.total)}</p>
          {g.kinds.map(k => <div className="bar-item" key={k.kind} title={formatBytes(k.bytes)}>
            <span className="bar-label">{k.kind}</span>
            <div className="bar-track"><span style={{ width: `${(k.count / max) * 100}%` }} /></div>
            <span className="text-right tabular-nums">{formatCount(k.count)}</span>
          </div>)}
        </div>) : <p className="hint">Düzenlenecek tanınan dosya bulunamadı.</p>}
      </div>
      <div className="flex flex-col gap-3.5 min-w-0">
        <div className="chart-surface">
          <h3>Şimdi · taşınacak dosyaların klasörleri</h3>
          {sources.length ? <pre className="organize-tree">{sources.slice(0, MAX_SOURCES).map(s => `${s.folder}  (${formatCount(s.count)})`).join("\n")}</pre> : <p className="hint">Taşınacak dosya yok; klasör zaten düzenli.</p>}
          {sources.length > MAX_SOURCES && <p className="hint">ve {formatCount(sources.length - MAX_SOURCES)} klasör daha</p>}
        </div>
        <div className="chart-surface">
          <h3>Sonra · oluşacak yapı</h3>
          <pre className="organize-tree">{groups.map(g => flat
            ? g.kinds.map(k => `${k.kind}/  (${formatCount(k.count)})`).join("\n")
            : `${g.category}/\n${g.kinds.map((k, i) => `${i === g.kinds.length - 1 ? "└─" : "├─"} ${k.kind}/  (${formatCount(k.count)})`).join("\n")}`).join("\n")}
            {counts.quarantine > 0 && `\n_Kopyalar_Karantina/  (${formatCount(counts.quarantine)})`}
            {plan.protectedDirs.length > 0 && `\n${plan.protectedDirs.length} proje klasörü olduğu gibi kalır`}</pre>
        </div>
      </div>
    </div>
    {!!plan.warnings.length && <details className="panel-body text-xs pt-0"><summary>Okunamayan öğeler ({plan.warnings.length})</summary>{plan.warnings.slice(0, 50).map((w, i) => <p className="hint break-all" key={i}>{w}</p>)}</details>}
  </section>;
}

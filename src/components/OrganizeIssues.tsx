import { useMemo, useState } from "react";
import { ExternalLink, ListChecks } from "lucide-react";
import type { OrganizeEntry, OrganizePlan } from "../types/organize";
import { baseName, formatBytes, formatCount } from "../lib/format";
import { ISSUE_FILTERS, matchesIssue, relative, type IssueFilter } from "../lib/organizeView";
import VirtualRows from "./VirtualRows";

function badgeOf(e: OrganizeEntry): [string, string] {
  if (e.action === "Skip") return ["bad", "Atlandı"];
  if (e.action === "Quarantine") return ["exact", "Birebir kopya"];
  if (e.renamed) return ["warn", "Numaralandı"];
  if (e.similarTo) return ["ok", "Benzer ad"];
  if (e.action === "InPlace") return ["", "Zaten yerinde"];
  return ["", "Taşınacak"];
}

function detailOf(e: OrganizeEntry): string {
  if (e.note) return e.note;
  if (e.renamed) return "Hedefte aynı adlı dosya var; daha yeni olan numara alır";
  if (e.similarTo) return `“${e.similarTo}” ile adı benzer · ikisi de kalır`;
  return formatBytes(e.size);
}

/** Numaralanan, karantinaya giden, benzer adli ve atlanan dosyalarin listesi. */
export default function OrganizeIssues({ plan, onReveal }: { plan: OrganizePlan; onReveal: (path: string) => void }) {
  const [filter, setFilter] = useState<IssueFilter>("attention");
  const rows = useMemo(() => plan.entries.filter(e => matchesIssue(e, filter)), [plan, filter]);
  return <section className="panel">
    <div className="panel-head">
      <div><h2><ListChecks size={18} /> Dosya ayrıntıları</h2><p>{formatCount(rows.length)} dosya · her satır nedeniyle birlikte</p></div>
      <select value={filter} onChange={e => setFilter(e.target.value as IssueFilter)} aria-label="Ayrıntı filtresi">
        {ISSUE_FILTERS.map(f => <option key={f.id} value={f.id}>{f.label}</option>)}
      </select>
    </div>
    {rows.length ? <VirtualRows items={rows} estimate={60} itemKey={e => e.from} render={e => {
      const [tone, label] = badgeOf(e);
      return <div className="organize-row">
        <div className="min-w-0"><button className="name" title={e.from} onClick={() => onReveal(e.from)}>{baseName(e.from)}</button><small title={e.from}>{relative(plan.rootPath, e.from)} · {detailOf(e)}</small></div>
        <div className="target">{e.to ? relative(plan.rootPath, e.to) : "yerinde kalır"}</div>
        <div className="inline-actions"><span className={`badge ${tone}`}>{label}</span><button className="btn" onClick={() => onReveal(e.from)} aria-label={`${baseName(e.from)} klasörde göster`}><ExternalLink size={14} /></button></div>
      </div>;
    }} /> : <div className="empty-state">Bu filtrede dosya yok.</div>}
  </section>;
}

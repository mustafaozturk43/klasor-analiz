import { FileSpreadsheet, FolderInput } from "lucide-react";
import type { OrganizerState } from "../../types/app";
import type { OrganizeSettings } from "./useOrganizeSettings";
import type { OrganizerHook } from "../../hooks/useOrganizer";
import { formatCount } from "../../lib/format";
import { countPlan } from "./organizeView";
import OrganizeIssues from "./OrganizeIssues";
import OrganizeResult from "./OrganizeResult";
import OrganizeRulesEditor from "./OrganizeRulesEditor";
import OrganizeSummary from "./OrganizeSummary";

interface Props { app: OrganizerState; settings: OrganizeSettings; org: OrganizerHook; locked: boolean }

/** "Türüne göre düzenle" çıktısı: klasör kuralları, önizleme özeti, sorunlar, uygulama çubuğu ve sonuç. */
export default function OrganizeOutput({ app, settings, org, locked }: Props) {
  const counts = org.plan ? countPlan(org.plan) : null;
  const actionable = counts ? counts.move + counts.quarantine : 0;
  const off = !!org.busy || locked;
  return <div className="organize-stack">
    <OrganizeRulesEditor settings={settings} disabled={off} />
    {org.outcome && <OrganizeResult outcome={org.outcome} disabled={off}
      onOpen={() => void app.reveal(app.rootPath)}
      onUndo={() => { org.reset(); void app.undoLast(); }} />}
    {org.plan && <>
      <OrganizeSummary plan={org.plan} flat={settings.toggles.flat} />
      <OrganizeIssues plan={org.plan} onReveal={path => void app.reveal(path)} />
      <section className="panel panel-body inline-actions justify-end">
        <span className="hint mr-auto">{actionable ? `${formatCount(counts!.move)} dosya taşınacak, ${formatCount(counts!.quarantine)} kopya karantinaya alınacak.` : "Taşınacak dosya yok; klasör zaten düzenli."}</span>
        <button className="btn" onClick={() => void org.exportCsv()} disabled={!!org.busy}><FileSpreadsheet size={14} /> CSV olarak kaydet</button>
        <button className="btn" onClick={org.reset} disabled={!!org.busy}>Vazgeç</button>
        <button className="btn primary" onClick={() => void org.apply()} disabled={!actionable || !!org.busy || locked}><FolderInput size={14} /> {org.busy === "apply" ? "Düzenleniyor…" : `${formatCount(actionable)} dosyayı düzenle`}</button>
      </section>
    </>}
  </div>;
}

import { useEffect, useMemo, useState } from "react";
import { ArrowLeftRight, Eye, FolderInput, FolderOpen, ListChecks, PencilLine } from "lucide-react";
import type { OrganizerState } from "../../types/app";
import type { FileItem } from "../../types/file";
import { useFolderStats } from "../map/useFolderStats";
import { usePagination } from "../../hooks/usePagination";
import { baseName, formatBytes } from "../../lib/format";
import { creationDate, previewDateRename, previewItem, type RenamePlan } from "../../lib/fileOperations";
import { FolderSummary } from "../map/FolderAnalysis";
import Switch from "../../components/Switch";
import Pagination from "../../components/Pagination";
import VirtualRows from "../../components/VirtualRows";
import RenameDialog from "../rename/RenameDialog";
import SelectionSummary from "../map/SelectionSummary";

function CompareCard({ root, token, label, pick, disabled }: { root: string; token: number; label: string; pick: () => void; disabled: boolean }) {
  const { stats, loading, error } = useFolderStats(root, token);
  return <section className="panel"><div className="panel-head"><h2><FolderOpen size={18} /> {label}</h2><button className="btn" disabled={disabled} onClick={pick}>{root ? "Değiştir" : "Klasör seç"}</button></div>
    <div className="panel-body"><p className="hint mb-4 truncate" title={root}>{root || "Klasör seçilmedi"}</p>
      {loading ? <p role="status">Hesaplanıyor…</p> : error ? <p role="alert">{error}</p> : stats ? <><p className="mb-4 text-sm">{stats.rootFileCount.toLocaleString("tr-TR")} dosya · {formatBytes(stats.rootTotalBytes)}</p>
        <FolderSummary stats={stats} /></> : <p className="hint">Karşılaştırmak için iki farklı klasör seçin.</p>}
    </div></section>;
}

export default function CompareView({ app, onPreview }: { app: OrganizerState; onPreview: (file: FileItem) => void }) {
  const [details, setDetails] = useState(false);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [plan, setPlan] = useState<RenamePlan | null>(null);
  const [filter, setFilter] = useState("all");
  const inventory = app.scanResult?.comparison?.files ?? [];
  const groups = useMemo(() => new Map(app.groups.map(g => [g.id, g])), [app.groups]);
  useEffect(() => { setSelected(new Set()); }, [app.scanResult]);
  const filtered = useMemo(() => inventory.filter(entry => filter === "all" || (entry.groupId ? groups.get(entry.groupId)?.groupType : "unmatched") === filter), [inventory, filter, groups]);
  const pagination = usePagination(filtered);
  const selectedFiles = inventory.filter(entry => selected.has(entry.file.path));
  const busy = app.isMoving || app.isScanning;
  function selectType(type: "Exact" | "Similar") {
    setSelected(new Set(inventory.filter(e => e.groupId && groups.get(e.groupId)?.groupType === type).map(e => e.file.path)));
    app.selectGroups(type);
  }
  async function rename() {
    app.setFileOperationBusy(true);
    try { setPlan(await previewDateRename([app.rootPath, app.rootB].filter(Boolean), selectedFiles.map(e => e.file.path))); }
    catch (e) { app.setStatus({ type: "error", text: String(e) }); app.setFileOperationBusy(false); }
  }
  async function refresh() { app.clearSelection(); await app.rescan(app.rootPath); app.bumpStatsToken(); await app.refreshHistory(); }
  return <div className="analysis-stack">
    <div className="two-column"><CompareCard root={app.rootPath} token={app.statsToken} label="Klasör A · Ana klasör" pick={app.pickFolder} disabled={busy} />
      <CompareCard root={app.rootB} token={app.statsToken} label="Klasör B · Karşılaştırılan" pick={app.pickFolderB} disabled={busy} /></div>
    <section className="panel"><div className="panel-head"><div><h2><ArrowLeftRight size={18} /> Karşılaştırma sonuçları</h2>
      <p>{app.scanResult?.comparison ? `${app.groups.length} klasörler arası grup · ${inventory.length} taranan dosya` : "A ve B seçildikten sonra karşılaştırmayı başlatın."}</p></div>
      <div className="inline-actions"><button className="btn primary" disabled={!app.rootPath || !app.rootB || busy} onClick={app.compareScan}>Karşılaştır</button>
        <label className="inline-actions text-xs"><Switch checked={details} onChange={setDetails} label="Dosya eşleşme detaylarını göster" /> Detayları göster</label></div></div>
      {details && <><div className="panel-body inline-actions">
        <button className="btn" disabled={busy || !inventory.length} onClick={() => selectType("Exact")}><ListChecks size={15} /> Birebir kopyaları seç</button>
        <button className="btn" disabled={busy || !inventory.length} onClick={() => selectType("Similar")}>İsim benzerlerini seç</button>
        <button className="btn" disabled={busy} onClick={() => { setSelected(new Set()); app.clearSelection(); }}>Seçimi temizle</button>
        <select aria-label="Eşleşme filtresi" value={filter} onChange={e => setFilter(e.target.value)}><option value="all">Tüm dosyalar</option><option value="Exact">Birebir</option><option value="Similar">İsim benzeri</option><option value="unmatched">Eşleşmeyenler</option></select>
        <span className="hint">{selectedFiles.length} dosya seçili · Seçim tüm sayfalara uygulanır</span>
      </div>
      {!filtered.length ? <div className="empty-state">Bu kapsamda dosya bulunamadı.</div> : <VirtualRows items={pagination.items} itemKey={e => e.file.path} estimate={100} render={entry => {
        const group = entry.groupId ? groups.get(entry.groupId) : undefined;
        return <div className={`data-row ${selected.has(entry.file.path) ? "selected" : ""}`}>
          <input type="checkbox" aria-label={`${entry.file.name} dosyasını seç`} checked={selected.has(entry.file.path)} disabled={busy}
            onChange={() => { app.clearSelection(); setSelected(prev => { const next = new Set(prev); if (next.has(entry.file.path)) next.delete(entry.file.path); else next.add(entry.file.path); return next; }); }} />
          <div className="file-description"><button onClick={() => onPreview(previewItem(entry.file))}>{entry.file.name}</button>
            <small title={entry.file.path}>{entry.source}: {entry.file.path}</small><small>Oluşturulma (UTC): {creationDate(entry.file.createdAt)} · {formatBytes(entry.file.size)}</small></div>
          <div className="row-badges">
            <span className="badge">{entry.source}</span>{group && <span className="badge">{group.id}</span>}
            <span className={`badge ${group ? group.groupType === "Exact" ? "exact" : "similar" : "unmatched"}`}>{group ? group.groupType === "Exact" ? "Birebir" : "İsim benzeri" : "Eşleşme yok"}</span>
            {group?.groupType === "Exact" && entry.file.size >= 524288000 && <span className="badge">Örnek hash</span>}</div>
        </div>;
      }} />}
      <Pagination {...pagination} />
      <div className="panel-body inline-actions"><button className="btn" disabled={!selectedFiles.length || busy} onClick={() => onPreview(previewItem(selectedFiles[0].file))}><Eye size={15} /> Ön izle</button>
        <button className="btn" disabled={!selectedFiles.length || busy} onClick={() => void rename()}><PencilLine size={15} /> Tarihle isimlendir</button>
        <button className="btn" disabled={!selectedFiles.length || busy} onClick={() => { const ids = new Set(selectedFiles.flatMap(e => e.groupId ? [e.groupId] : [])); app.clearSelection();
          app.groups.filter(g => ids.has(g.id)).forEach(g => app.toggleGroup(g.id));
          app.setStatus({ type: "info", text: "Seçilen dosyaların tüm grupları taşıma için seçildi. Aşağıdaki “Seçili grupları taşı” düğmesiyle taşımayı inceleyin." }); }}>Grupları taşımaya hazırla</button>
        <p className="hint">Taşıma tüm grup üyelerini kapsar. Benzer isimler aynı içerik anlamına gelmez.</p>
      </div></>}
      {app.selectedGroups.length > 0 && <div className="panel-body flex flex-col gap-3">
        <SelectionSummary app={app} />
        <div><button className="btn primary" disabled={busy} onClick={() => void app.applyMove()}><FolderInput size={16} /> Seçili {app.selectedGroups.length} grubu taşı</button></div>
      </div>}
    </section>
    {plan && <RenameDialog plan={plan} onClose={() => { setPlan(null); app.setFileOperationBusy(false); }} onApplied={refresh} />}
    <p className="hint">{baseName(app.rootPath) || "A"} ana hedeftir. Büyük dosyalardaki örnek hash eşleşmeleri tam içerik kanıtı değildir.</p>
  </div>;
}

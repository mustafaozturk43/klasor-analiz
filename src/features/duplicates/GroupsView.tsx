import { useEffect, useMemo, useState } from "react";
import { Sparkles } from "lucide-react";
import type { OrganizerState } from "../../types/app";
import type { FileItem } from "../../types/file";
import { baseName } from "../../lib/format";
import { suggestTagName } from "./tagSuggest";
import { usePagination } from "../../hooks/usePagination";
import Pagination from "../../components/Pagination";
import GroupListItem from "./GroupListItem";
import GroupCompare from "./GroupCompare";
import SelectionSummary from "../map/SelectionSummary";

/** Kopya grupları: solda süzülebilir liste, sağda seçili grubun karşılaştırması ve tek grubu taşıma. */
export default function GroupsView({ app, onPreview, active }: { app: OrganizerState; onPreview: (file: FileItem) => void; active: boolean }) {
  const [search, setSearch] = useState("");
  const [type, setType] = useState("all");
  const [category, setCategory] = useState("all");
  const [sort, setSort] = useState("size");
  const [minMB, setMinMB] = useState(0);
  const [picked, setPicked] = useState<string | null>(null);
  const count = (t: string) => app.groups.filter(g => g.groupType === t).length;
  const filtered = useMemo(() => app.groups.filter(g => (type === "all" || g.groupType === type) && (category === "all" || g.category === category)
    && g.files.some(f => f.name.toLocaleLowerCase("tr").includes(search.toLocaleLowerCase("tr")))
    && g.files.some(f => f.size >= minMB * 1024 ** 2))
    .map(g => ({ group: g, size: g.files.reduce((sum, f) => sum + f.size, 0) }))
    .sort((a, b) => sort === "name" ? a.group.files[0].name.localeCompare(b.group.files[0].name, "tr") : sort === "count" ? b.group.files.length - a.group.files.length : b.size - a.size)
    .map(entry => entry.group), [app.groups, type, category, search, minMB, sort]);
  const pagination = usePagination(filtered);
  const shown = filtered.find(g => g.id === picked) ?? pagination.items[0] ?? null;
  useEffect(() => {
    if (!active) return;
    const onKey = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      if (e.defaultPrevented || app.isMoving || app.isScanning || document.querySelector("dialog[open]") || target.matches("input,textarea,select,[contenteditable=true]")) return;
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "a") { e.preventDefault(); app.toggleAllVisible(pagination.items); }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [active, app.isMoving, app.isScanning, app.toggleAllVisible, pagination.items]);
  const types = [["all", "Tümü", app.groups.length], ["Exact", "Birebir", count("Exact")], ["Similar", "Benzer", count("Similar")]] as const;
  const busy = app.isMoving || app.isScanning;

  return <div className="dup-layout">
    <section className="dup-list" aria-label="Gruplar">
      <div className="dup-list-tools">
      <div className="dup-chips" role="group" aria-label="Grup türü">{types.map(([v, label, n]) =>
        <button key={v} type="button" aria-pressed={type === v} onClick={() => setType(v)}>{label} {n}</button>)}</div>
      <div className="dup-filters">
        <input type="search" aria-label="Gruplarda ara" placeholder="Dosya adında ara…" value={search} onChange={e => setSearch(e.target.value)} />
        <select value={category} onChange={e => setCategory(e.target.value)} aria-label="Kategori"><option value="all">Tüm kategoriler</option>
          <option value="Document">Doküman</option><option value="Image">Resim</option><option value="Archive">Arşiv</option><option value="Other">Diğer</option></select>
        <select value={sort} onChange={e => setSort(e.target.value)} aria-label="Grup sırası"><option value="size">Boyut ↓</option><option value="name">Dosya adı</option><option value="count">Dosya sayısı ↓</option></select>
        <label>Min MB <input type="number" min="0" value={minMB} onChange={e => setMinMB(Math.max(0, Number(e.target.value) || 0))} /></label>
        <button className="btn" disabled={!filtered.length} onClick={() => app.toggleAllVisible(pagination.items)}>Bu sayfadakileri seç / kaldır</button>
        <button className="btn" disabled={!app.selectedGroups.length} onClick={() => app.selectedGroups.forEach(g => app.setGroupName(g.id, suggestTagName(g.files)))}><Sparkles size={14} /> Seçililere otomatik ad</button>
      </div>
      <p className="hint">Kutularla birden çok grup seçip sağdaki düğmeyle birlikte taşıyabilirsin. Ctrl+A bu sayfadakileri seçer.</p>
      </div>
      <ul className="dup-items">{pagination.items.map(g => <GroupListItem key={g.id} group={g} active={shown?.id === g.id} selected={app.selectedIds.has(g.id)} onPick={() => setPicked(g.id)} onToggle={() => app.toggleGroup(g.id)} />)}</ul>
      {!filtered.length && <div className="empty-state">{app.scanResult ? "Bu filtrelerle eşleşen grup yok." : "Klasör seçip taramayı başlatın."}</div>}
      <Pagination {...pagination} />
    </section>
    {shown ? <GroupCompare group={shown} policy={app.keepPolicy} setPolicy={app.setKeepPolicy} rootName={baseName(app.rootPath) || "Klasör"} busy={busy} onPreview={onPreview}
      name={app.groupNames[shown.id] ?? suggestTagName(shown.files)} setName={name => app.setGroupName(shown.id, name)} suggest={() => app.setGroupName(shown.id, suggestTagName(shown.files))}
      selectedCount={app.selectedGroups.length} summary={<SelectionSummary app={app} />}
      onMove={() => { if (!busy) void app.applyMove(app.selectedGroups.length > 1 ? undefined : shown); }} />
      : <section className="dup-compare dup-empty"><p>Karşılaştırmak için soldan bir grup seç.</p></section>}
  </div>;
}

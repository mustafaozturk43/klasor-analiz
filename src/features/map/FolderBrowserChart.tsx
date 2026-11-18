import { ChartBar, ChevronRight, FolderOpen } from "lucide-react";
import { formatBytes } from "../../lib/format";
import { usePagination } from "../../hooks/usePagination";
import type { FolderSource } from "./useFolderBrowser";
import type { FolderStat } from "../../types/file";
import Pagination from "../../components/Pagination";
import VirtualRows from "../../components/VirtualRows";

export default function FolderBrowserChart({ folders, loading, enter }: {
  folders: { folder: FolderStat; source: FolderSource }[];
  loading: boolean; enter: (path: string, source: FolderSource) => void;
}) {
  const pagination = usePagination(folders);
  const max = folders.reduce((max, entry) => Math.max(max, entry.folder.totalBytes), 1);
  if (loading) return <div className="empty-state" role="status">Alt klasör boyutları hesaplanıyor…</div>;
  if (!folders.length) return null;
  return <section aria-label="Alt klasör grafikleri" className="border-y border-[var(--color-rule)]">
    <div className="panel-head"><div><h3><ChartBar size={16} /> Alt klasörler</h3><p>İçeriğini listelemek için klasöre tıklayın. Grafikler tüm dosya türlerinin toplam boyutunu gösterir.</p></div><span className="badge">{folders.length} klasör</span></div>
    <VirtualRows items={pagination.items} itemKey={e => `${e.source}:${e.folder.path}`} estimate={64} maxHeight={320} render={({ folder, source }) =>
      <button type="button" className="folder-chart-row" onClick={() => enter(folder.path, source)} title={folder.path}>
        <FolderOpen size={18} /><span className="truncate text-left">{folder.name}</span><span className="badge">Klasör {source}</span>
        <span className="bar-track"><span style={{ width: `${folder.totalBytes / max * 100}%` }} /></span><span className="text-xs">{formatBytes(folder.totalBytes)}</span><ChevronRight size={15} />
      </button>} />
    {folders.length > 25 && <Pagination {...pagination} />}
  </section>;
}

import { useCallback, useEffect, useMemo, useState } from "react";
import { ArrowLeft, Camera, Files, Home } from "lucide-react";
import type { OrganizerState } from "../types/app";
import { usePagination } from "../hooks/usePagination";
import { useFolderBrowser } from "../hooks/useFolderBrowser";
import { useFileSelection } from "../hooks/useFileSelection";
import { useMediaDates } from "../hooks/useMediaDates";
import { useShellThumbs } from "../hooks/useShellThumbs";
import { DEFAULT_FILTERS, viewFiles, type BrowserFilters } from "../lib/browserView";
import Switch from "./Switch";
import Pagination from "./Pagination";
import VirtualRows from "./VirtualRows";
import FolderBrowserChart from "./FolderBrowserChart";
import FileBrowserFilters from "./FileBrowserFilters";
import FileBrowserRow from "./FileBrowserRow";
import BrowserPreview from "./BrowserPreview";
import SelectionPanel from "./SelectionPanel";

/** Harita > Klasordeki dosyalar: on izlemeli liste, suzme, secim ve (secim cubuguyla) gruplama. */
export default function FileBrowser({ app }: { app: OrganizerState }) {
  const browser = useFolderBrowser(app);
  const [filters, setFilters] = useState<BrowserFilters>(DEFAULT_FILTERS);
  const folderKey = `${app.rootPath}|${app.rootB}|${browser.scope}|${browser.current?.path ?? ""}`;
  const paths = useMemo(() => browser.files.map(e => e.file.path), [browser.files]);
  const dates = useMediaDates(paths, folderKey);
  const view = useMemo(() => viewFiles(browser.files, dates.taken, filters, !dates.reading), [browser.files, dates.taken, filters, dates.reading]);
  const order = useMemo(() => view.items.map(e => e.file.path), [view.items]);
  const selection = useFileSelection(order, folderKey);
  const thumbs = useShellThumbs(128);
  const big = useShellThumbs(768);
  const pagination = usePagination(view.items, `${folderKey}|${JSON.stringify(filters)}`);
  const [previewAt, setPreviewAt] = useState<number | null>(null);
  const { request } = thumbs;
  useEffect(() => { request(pagination.items.map(e => e.file.path)); }, [pagination.items, request]);
  const takenOf = useCallback((p: string) => (dates.taken.has(p) ? dates.taken.get(p) ?? null : undefined), [dates.taken]);
  const openPreview = useCallback((p: string) => setPreviewAt(order.indexOf(p)), [order]);
  const reveal = useCallback((p: string) => void app.reveal(p), [app]);
  const shown = order.filter(p => selection.selected.has(p)).length;

  return <section className="panel"><div className="panel-head"><div><h2><Files size={18} /> Klasördeki dosyalar</h2>
    <p>Küçük resme tıklayarak büyütün. Dosyaları işaretleyip bir grup adıyla bir klasöre taşıyın. Yasaklı uzantılar ve karantina listelenmez.</p></div>
    <label className="inline-actions text-xs"><Switch checked={app.fileListEnabled} onChange={app.setFileListEnabled} label="Dosyaları listele" /> Listele</label></div>
    {app.fileListEnabled ? <><div className="panel-body inline-actions pb-0">
      <select value={browser.scope} onChange={e => browser.setScope(e.target.value as "A" | "B" | "all")} aria-label="Listelenecek klasör">
        <option value="A">Klasör A</option><option value="B" disabled={!app.rootB}>Klasör B</option><option value="all" disabled={!app.rootB}>Tümü · A ve B</option></select>
      <select value={app.fileListMode} onChange={e => app.setFileListMode(e.target.value as "all" | "category")} aria-label="Liste kapsamı"><option value="category">Kategori dosyaları</option><option value="all">Tüm dosyalar</option></select>
      {browser.current && <><button className="btn" onClick={browser.back}><ArrowLeft size={14} /> Geri</button>
        <button className="btn" onClick={browser.home}><Home size={14} /> Ana klasörler</button><span className="hint break-all">Klasör {browser.current.source}: {browser.current.path}</span></>}
    </div>
    {browser.errors.map((error, i) => <p className="error-state" role="alert" key={i}>{error}</p>)}
    <FolderBrowserChart folders={browser.folders} loading={browser.folderLoading} enter={browser.enter} />
    {!!browser.warnings.length && <details className="panel-body text-xs"><summary>Analiz uyarıları ({browser.warnings.length})</summary>{browser.warnings.map((w, i) => <p className="hint break-all" key={i}>{w}</p>)}</details>}
    <FileBrowserFilters filters={filters} onChange={setFilters} years={view.years} months={view.months} counts={view.counts} datesLocked={dates.reading} />
    {!!dates.total && <p className={`browser-status${dates.reading ? " reading" : ""}`} role="status">
      {dates.reading ? <><span className="spinner" aria-hidden="true" />Çekim tarihleri okunuyor… {dates.done}/{dates.total}. Yıl ve ay süzgeci bitince açılır.</>
        : <><Camera size={14} aria-hidden="true" />Tarihler hazır: {[...dates.taken.values()].filter(v => v !== null).length} dosyada çekim tarihi, diğerlerinde değiştirme tarihi kullanılıyor.</>}
    </p>}
    {browser.loading ? <div className="empty-state" role="status">Dosyalar listeleniyor…</div> : view.items.length ? <>
      <div className="browser-head">
        <input type="checkbox" checked={shown > 0 && shown === order.length} ref={el => { if (el) el.indeterminate = shown > 0 && shown < order.length; }}
          onChange={e => selection.setMany(order, e.target.checked)} aria-label={`Süzülen ${order.length} dosyanın tümünü seç`} />
        <span>Ön izleme</span><span>Ad</span><span className="wide">Tarih</span><span className="wide text-right">Boyut</span><span />
      </div>
      <VirtualRows items={pagination.items} estimate={72} itemKey={e => `${e.source}:${e.file.path}`} render={({ file, source }) =>
        <FileBrowserRow file={file} source={source} selected={selection.selected.has(file.path)} taken={takenOf(file.path)} thumb={thumbs.thumbOf(file.path)}
          onToggle={selection.toggle} onPreview={openPreview} onReveal={reveal} />} />
      <Pagination {...pagination} />
    </> : <div className="empty-state">{!app.rootPath ? "Önce bir klasör seçin." : browser.files.length ? "Bu süzgeçlerle eşleşen dosya yok." : "Bu kapsamda dosya bulunamadı."}</div>}
    <SelectionPanel app={app} browser={browser} selection={selection} entries={browser.files} datesReady={!dates.reading}
      takenOf={takenOf} thumbOf={thumbs.thumbOf} requestThumbs={request} />
    <BrowserPreview items={view.items} index={previewAt} onIndex={setPreviewAt} isSelected={p => selection.selected.has(p)} onToggle={p => selection.toggle(p, false)}
      takenOf={takenOf} bigThumb={big.thumbOf} requestBig={big.request} />
    </> : <div className="empty-state">Dosya listesini görüntülemek için anahtarı açın.</div>}
  </section>;
}

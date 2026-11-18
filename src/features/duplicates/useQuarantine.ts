import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { confirmDialog } from "../../lib/commands";
import { deleteQuarantined, listQuarantine, restoreQuarantined } from "../map/healthCommands";
import { formatBytes, formatCount } from "../../lib/format";
import type { OrganizerState } from "../../types/app";
import type { QuarantineListing } from "../../types/health";

/** Karantina icerigi, grup filtresi, secim; geri yukleme ve Geri Donusum Kutusu'na gonderme. */
export function useQuarantine(app: OrganizerState, active: boolean) {
  const [listing, setListing] = useState<QuarantineListing | null>(null);
  const [loading, setLoading] = useState(false);
  const [group, setGroup] = useState("all");
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const lock = useRef(false);
  const { rootPath, setStatus, setFileOperationBusy, history, statsToken } = app;

  const load = useCallback(async () => {
    if (!rootPath) { setListing(null); return; }
    setLoading(true);
    try {
      setListing(await listQuarantine(rootPath));
    } catch (e) {
      setStatus({ type: "error", text: String(e) });
    } finally {
      setLoading(false);
    }
  }, [rootPath, setStatus]);

  // Sekme acildiginda ve tasima/geri alma/temizlik sonrasi yenile.
  useEffect(() => { if (active) void load(); }, [active, load, history, statsToken]);
  useEffect(() => { setSelected(new Set()); }, [listing, group]);

  const groups = useMemo(() => [...new Set(listing?.items.map(i => i.group) ?? [])], [listing]);
  const items = useMemo(() => (listing?.items ?? []).filter(i => group === "all" || i.group === group), [listing, group]);
  const selectedBytes = items.filter(i => selected.has(i.path)).reduce((s, i) => s + i.size, 0);

  const toggle = useCallback((path: string) => setSelected(prev => {
    const next = new Set(prev);
    if (next.has(path)) next.delete(path); else next.add(path);
    return next;
  }), []);
  // Saklama suresi onerisi: 30 gunden once karantinaya alinanlar (zamani gecmiste kayitli olanlar).
  const old = useMemo(() => {
    const limit = Date.now() - 30 * 86_400_000;
    return items.filter(i => i.quarantinedMs !== null && i.quarantinedMs < limit).map(i => i.path);
  }, [items]);
  const selectOld = useCallback(() => setSelected(new Set(old)), [old]);
  const toggleAll = useCallback(() => setSelected(prev => (prev.size === items.length ? new Set() : new Set(items.map(i => i.path)))), [items]);

  const run = useCallback(async (action: () => Promise<string>) => {
    if (lock.current) return;
    lock.current = true;
    setFileOperationBusy(true);
    try {
      setStatus({ type: "success", text: await action() });
    } catch (e) {
      setStatus({ type: "error", text: String(e) });
    } finally {
      lock.current = false;
      setFileOperationBusy(false);
      await app.refreshHistory();
      app.bumpStatsToken();
      await load();
    }
  }, [app, load, setStatus, setFileOperationBusy]);

  const restore = useCallback(() => run(async () => {
    const out = await restoreQuarantined(rootPath, [...selected]);
    return `${formatCount(out.done)} dosya geri yüklendi. İşlem Geçmişi'nden geri alınabilir.${out.failed.length ? ` ${out.failed.length} dosya taşınamadı: ${out.failed[0].error}` : ""}`;
  }), [run, rootPath, selected]);

  const remove = useCallback(async () => {
    const ok = await confirmDialog(
      `${formatCount(selected.size)} dosya (${formatBytes(selectedBytes)}) Windows Geri Dönüşüm Kutusu’na gönderilecek. Uygulamadan geri alınamaz; dosyalar Geri Dönüşüm Kutusu’ndan geri yüklenebilir.\n\nBu kopyaları karantinaya taşıyan eski işlemler de artık uygulamadan geri alınamaz.`,
      "Geri Dönüşüm Kutusu’na gönder", "Kutuya gönder",
    );
    if (ok) await run(async () => `${formatCount(await deleteQuarantined(rootPath, [...selected]))} dosya Geri Dönüşüm Kutusu’na gönderildi.`);
  }, [run, rootPath, selected, selectedBytes]);

  return { listing, loading, group, setGroup, groups, items, selected, selectedBytes, toggle, toggleAll, restore, remove, load, oldCount: old.length, selectOld };
}

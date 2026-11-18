import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { confirmDialog } from "../../lib/commands";
import { folderHealth, quarantineFiles, removeEmptyFolders, trashHealthFiles } from "./healthCommands";
import { selectedBytes } from "./healthSuggest";
import { formatBytes, formatCount } from "../../lib/format";
import type { OrganizerState } from "../../types/app";
import type { HealthReport } from "../../types/health";

export type HealthTab = "largest" | "oldest" | "emptyFiles" | "emptyDirs";

/** Karantinaya alinan dosyalarin karantinadaki klasor adi. */
const LABELS: Record<Exclude<HealthTab, "emptyDirs">, string> = {
  largest: "Büyük dosyalar", oldest: "Eski dosyalar", emptyFiles: "Boş dosyalar",
};

/** Klasor sagligi raporu, secim ve toplu islemler (karantinaya al, bos klasor kaldir). */
export function useHealth(app: OrganizerState) {
  const [report, setReport] = useState<HealthReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [tab, setTab] = useState<HealthTab>("largest");
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const lock = useRef(false);
  const { rootPath, setStatus, setFileOperationBusy } = app;

  useEffect(() => { setReport(null); setSelected(new Set()); }, [rootPath]);
  useEffect(() => { setSelected(new Set()); }, [tab, report]);

  const rows = useMemo(() => {
    if (!report) return [];
    if (tab === "emptyDirs") return report.emptyDirs.map(path => ({ path, size: 0, mtime: 0 }));
    return report[tab];
  }, [report, tab]);

  const run = useCallback(async () => {
    if (!rootPath || lock.current) return;
    lock.current = true;
    setLoading(true);
    try {
      setReport(await folderHealth(rootPath, app.includeSubdirs, app.excludeText.split(",").map(s => s.trim()).filter(Boolean)));
    } catch (e) {
      setStatus({ type: "error", text: String(e) });
    } finally {
      lock.current = false;
      setLoading(false);
    }
  }, [rootPath, app.includeSubdirs, app.excludeText, setStatus]);

  const toggle = useCallback((path: string) => setSelected(prev => {
    const next = new Set(prev);
    if (next.has(path)) next.delete(path); else next.add(path);
    return next;
  }), []);
  const toggleAll = useCallback(() => setSelected(prev => (prev.size === rows.length ? new Set() : new Set(rows.map(r => r.path)))), [rows]);

  /** Islem sonrasi gecmis, analiz ve rapor yenilenir. */
  const runAction = useCallback(async (title: string, message: string, ok: string, action: () => Promise<string>) => {
    if (!selected.size || lock.current || !await confirmDialog(message, title, ok)) return;
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
    }
    await run();
  }, [selected, app, run, setStatus, setFileOperationBusy]);

  const act = useCallback(async () => {
    const paths = [...selected];
    if (tab === "emptyDirs") {
      await runAction("Boş klasörleri kaldır", `${formatCount(paths.length)} boş klasör ve bu yüzden boşalan üst klasörleri silinecek. İçinde dosya olan klasörlere dokunulmaz. Klasör silme geri alınamaz.`, "Kaldır",
        async () => `${formatCount(await removeEmptyFolders(rootPath, paths))} boş klasör kaldırıldı.`);
      return;
    }
    const label = LABELS[tab];
    await runAction("Karantinaya al", `${formatCount(paths.length)} dosya “_Kopyalar_Karantina/${label}” klasörüne taşınacak. Silinmez; Karantina sekmesinden veya İşlem Geçmişi'nden geri alınabilir.`, "Karantinaya al",
      async () => {
        const out = await quarantineFiles(rootPath, paths, label);
        return `${formatCount(out.done)} dosya karantinaya alındı.${out.failed.length ? ` ${out.failed.length} dosya taşınamadı: ${out.failed[0].error}` : ""}`;
      });
  }, [selected, tab, rootPath, runAction]);

  /** Bir oneriye uyan satirlari secer (mevcut secimin yerine). */
  const selectPaths = useCallback((paths: string[]) => setSelected(new Set(paths)), []);
  const bytes = useMemo(() => selectedBytes(rows, selected), [rows, selected]);

  /** Secili dosyalari Geri Donusum Kutusu'na gonderir; Gecmis'ten geri alinamaz, bu yuzden net onay metni gosterilir. */
  const trash = useCallback(async () => {
    if (tab === "emptyDirs") return;
    const items = rows.filter(r => selected.has(r.path)).map(r => ({ path: r.path, size: r.size, mtime: r.mtime }));
    await runAction("Geri Dönüşüm Kutusu'na gönder",
      `${formatCount(items.length)} dosya (${formatBytes(bytes)}) Windows Geri Dönüşüm Kutusu'na gönderilecek. Bu işlem İşlem Geçmişi'nden geri alınamaz; dosyaları Geri Dönüşüm Kutusu'ndan geri yükleyebilirsin. Rapordan sonra değişen bir dosya varsa hiçbir şey gönderilmez.`,
      "Gönder", async () => {
        const out = await trashHealthFiles(rootPath, items);
        return `${formatCount(out.done)} dosya Geri Dönüşüm Kutusu'na gönderildi.${out.failed.length ? ` ${out.failed.length} dosya gönderilemedi: ${out.failed[0].error}` : ""}`;
      });
  }, [tab, rows, selected, bytes, rootPath, runAction]);

  return { report, loading, tab, setTab, rows, selected, selectedBytes: bytes, selectPaths, toggle, toggleAll, run, act, trash };
}

export type HealthState = ReturnType<typeof useHealth>;

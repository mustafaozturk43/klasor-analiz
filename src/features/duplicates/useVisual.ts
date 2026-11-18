import { useCallback, useEffect, useRef, useState } from "react";
import { confirmDialog } from "../../lib/commands";
import { formatBytes, formatCount } from "../../lib/format";
import type { OrganizerState } from "../../types/app";
import { quarantineFiles } from "../map/healthCommands";
import { DEFAULT_DISTANCE, canSelect, defaultSelection, findSimilarImages, type VisualReport } from "./visualCommands";

const LABEL = "Benzer görseller";

/** Benzer gorseller: tarama (tarama kapsami ustteki ayarlardan), grup basina tutulacak dosya secimi, karantinaya alma. */
export function useVisual(app: OrganizerState) {
  const [report, setReport] = useState<VisualReport | null>(null);
  const [distance, setDistance] = useState(DEFAULT_DISTANCE);
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState<{ done: number; total: number } | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const lock = useRef(false);
  const { rootPath, setStatus, setFileOperationBusy } = app;

  useEffect(() => { setReport(null); setSelected(new Set()); }, [rootPath]);

  const run = useCallback(async () => {
    if (!rootPath || lock.current) return;
    lock.current = true;
    setLoading(true);
    setProgress({ done: 0, total: 0 });
    try {
      const r = await findSimilarImages(rootPath, app.includeSubdirs, app.excludeText.split(",").map(s => s.trim()).filter(Boolean), distance,
        p => setProgress({ done: p.filesDone, total: p.totalFiles }));
      setReport(r);
      setSelected(defaultSelection(r.groups));
    } catch (e) {
      setStatus({ type: "error", text: String(e) });
    } finally {
      lock.current = false;
      setLoading(false);
      setProgress(null);
    }
  }, [rootPath, app.includeSubdirs, app.excludeText, distance, setStatus]);

  const toggle = useCallback((groupId: string, path: string) => {
    const group = report?.groups.find(g => g.id === groupId);
    if (!group) return;
    setSelected(prev => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else if (canSelect(group, prev, path)) next.add(path);
      return next;
    });
  }, [report]);

  const selectedBytes = report ? report.groups.flatMap(g => g.files).filter(f => selected.has(f.path)).reduce((s, f) => s + f.size, 0) : 0;

  /** Secilenleri karantinaya alir (silmez, Gecmis'ten geri alinir); sonra yeniden tarar. */
  const act = useCallback(async () => {
    const paths = [...selected];
    if (!paths.length || lock.current) return;
    const text = `${formatCount(paths.length)} görsel (${formatBytes(selectedBytes)}) “_Kopyalar_Karantina/${LABEL}” klasörüne taşınacak. Her grupta en az bir görsel yerinde kalır. Silinmez; Karantina sekmesinden veya İşlem Geçmişi'nden geri alınır.`;
    if (!await confirmDialog(text, "Karantinaya al", "Karantinaya al")) return;
    lock.current = true;
    setFileOperationBusy(true);
    try {
      const out = await quarantineFiles(rootPath, paths, LABEL);
      setStatus({ type: out.failed.length ? "error" : "success", text: `${formatCount(out.done)} görsel karantinaya alındı.${out.failed.length ? ` ${out.failed.length} dosya taşınamadı: ${out.failed[0].error}` : ""}` });
    } catch (e) {
      setStatus({ type: "error", text: String(e) });
    } finally {
      lock.current = false;
      setFileOperationBusy(false);
      await app.refreshHistory();
      app.bumpStatsToken();
    }
    await run();
  }, [selected, selectedBytes, rootPath, app, run, setStatus, setFileOperationBusy]);

  return { report, distance, setDistance, loading, progress, selected, selectedBytes, run, toggle, act };
}

export type VisualState = ReturnType<typeof useVisual>;

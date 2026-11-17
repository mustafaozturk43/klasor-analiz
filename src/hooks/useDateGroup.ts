import { useCallback, useEffect, useRef, useState } from "react";
import { confirmDialog } from "../lib/commands";
import { applyDateGroup, previewDateGroup, type DateGroupOptions, type DateGroupPlan } from "../lib/dateGroupCommands";
import { formatCount } from "../lib/format";
import { getPref, setPref } from "../lib/prefs";
import { getSettings } from "../lib/settingsCommands";
import type { OrganizerState } from "../types/app";
import type { ExtGroup } from "../types/settings";

const KEY = "dateGroupOptions";
const DEFAULTS: DateGroupOptions = { extensions: [], recursive: false };

/** Tarihe gore (Yıl\Ay) gruplama: secenekler (`uiPrefs.dateGroupOptions`), canli onizleme, uygulama ve geri alma. */
export function useDateGroup(app: OrganizerState, active: boolean) {
  const [options, setOptionsState] = useState<DateGroupOptions>(() => ({ ...DEFAULTS, ...getPref<Partial<DateGroupOptions>>(KEY) }));
  const [plan, setPlan] = useState<DateGroupPlan | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState<{ done: number; total: number } | null>(null);
  const [busy, setBusy] = useState(false);
  const [groups, setGroups] = useState<ExtGroup[]>([]);
  const [lastId, setLastId] = useState<string | null>(null);
  const request = useRef(0);
  const { rootPath, statsToken, setStatus, setFileOperationBusy, refreshHistory, bumpStatsToken } = app;

  const setOptions = useCallback((next: DateGroupOptions) => {
    setOptionsState(next);
    setPref(KEY, next);
  }, []);

  useEffect(() => {
    if (active) getSettings().then(s => setGroups(s.extGroups)).catch(() => setGroups([]));
  }, [active]);

  // Onizleme: klasor, secenek veya disk degisince yeniden hesaplanir; diske yazmaz.
  useEffect(() => {
    if (!active || !rootPath || busy) return;
    const id = ++request.current;
    const timer = window.setTimeout(() => {
      setProgress({ done: 0, total: 0 });
      previewDateGroup(rootPath, options, p => { if (id === request.current) setProgress({ done: p.filesDone, total: p.totalFiles }); })
        .then(result => { if (id === request.current) { setPlan(result); setError(null); } })
        .catch(e => { if (id === request.current) { setPlan(null); setError(String(e)); } })
        .finally(() => { if (id === request.current) setProgress(null); });
    }, 300);
    return () => window.clearTimeout(timer);
  }, [active, rootPath, options, statsToken, busy]);

  const apply = useCallback(async () => {
    const n = plan?.entries.length ?? 0;
    if (!plan || !n || busy) return;
    const text = `${formatCount(n)} dosya ${formatCount(plan.buckets.length)} Yıl\\Ay klasörüne taşınacak. Hiçbir dosyanın üstüne yazılmaz; aynı ad varsa numaralandırılır. `
      + "İşlem Geçmişi'nden geri alınabilir.";
    if (!await confirmDialog(text, "Tarihe göre grupla", "Grupla")) return;
    setBusy(true);
    setFileOperationBusy(true);
    try {
      const out = await applyDateGroup(plan.id);
      setLastId(out.moved.length ? plan.id : null);
      const failed = out.failed.length ? ` ${formatCount(out.failed.length)} dosya taşınamadı: ${out.failed[0].error}` : "";
      setStatus({ type: out.failed.length ? "error" : "success", text: `${formatCount(out.moved.length)} dosya Yıl\\Ay klasörlerine taşındı. İşlem geri alınabilir.${failed}` });
    } catch (e) {
      setStatus({ type: "error", text: String(e) });
    } finally {
      setPlan(null);
      await refreshHistory();
      bumpStatsToken();
      setFileOperationBusy(false);
      setBusy(false);
    }
  }, [plan, busy, setStatus, setFileOperationBusy, refreshHistory, bumpStatsToken]);

  /** Yalnizca Gecmis'in en ustundeki kayit bu gruplamaysa geri alinir. */
  const canUndo = !!lastId && app.history[0]?.id === lastId;
  const undo = useCallback(async () => {
    if (!lastId) return;
    setLastId(null);
    await app.undoEntry(lastId);
  }, [lastId, app]);

  return { options, setOptions, defaults: DEFAULTS, plan, error, progress, busy, groups, apply, canUndo, undo };
}

export type DateGroupState = ReturnType<typeof useDateGroup>;

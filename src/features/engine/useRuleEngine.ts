import { useCallback, useEffect, useRef, useState } from "react";
import { confirmDialog } from "../../lib/commands";
import { formatCount } from "../../lib/format";
import { getPref, setPref } from "../../lib/prefs";
import type { OrganizerState } from "../../types/app";
import { applyRules, previewRules, type Rule, type RulePlan } from "./ruleEngineCommands";
import { newRule } from "./ruleModel";

const KEY = "ruleEngine";
interface Saved { rules: Rule[]; recursive: boolean }
const load = (): Saved => ({ rules: [], recursive: false, ...getPref<Partial<Saved>>(KEY) });

/** Kural motoru: kurallar (`uiPrefs.ruleEngine`), canli onizleme, uygulama ve geri alma. */
export function useRuleEngine(app: OrganizerState, active: boolean) {
  const [saved, setSaved] = useState<Saved>(load);
  const [plan, setPlan] = useState<RulePlan | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [moving, setMoving] = useState<{ done: number; total: number } | null>(null);
  const [busy, setBusy] = useState(false);
  const [lastId, setLastId] = useState<string | null>(null);
  const request = useRef(0);
  const { rootPath, statsToken, setStatus, setFileOperationBusy, refreshHistory, bumpStatsToken } = app;

  const update = useCallback((next: Saved) => {
    setSaved(next);
    setPref(KEY, next);
  }, []);
  const setRules = useCallback((rules: Rule[]) => update({ ...saved, rules }), [saved, update]);
  const patchRule = useCallback((i: number, patch: Partial<Rule>) => setRules(saved.rules.map((r, k) => (k === i ? { ...r, ...patch } : r))), [saved.rules, setRules]);
  const addRule = useCallback(() => setRules([...saved.rules, newRule(saved.rules.length + 1)]), [saved.rules, setRules]);
  const removeRule = useCallback((i: number) => setRules(saved.rules.filter((_, k) => k !== i)), [saved.rules, setRules]);
  const moveRule = useCallback((i: number, by: -1 | 1) => {
    const j = i + by;
    if (j < 0 || j >= saved.rules.length) return;
    const next = [...saved.rules];
    [next[i], next[j]] = [next[j], next[i]];
    setRules(next);
  }, [saved.rules, setRules]);
  const setRecursive = useCallback((recursive: boolean) => update({ ...saved, recursive }), [saved, update]);

  // Onizleme: klasor, kural veya disk degisince yeniden hesaplanir; diske yazmaz.
  useEffect(() => {
    if (!active || !rootPath || busy) return;
    if (!saved.rules.some(r => r.enabled)) { setPlan(null); setError(null); return; }
    const id = ++request.current;
    const timer = window.setTimeout(() => {
      setLoading(true);
      previewRules(rootPath, saved.rules, saved.recursive)
        .then(result => { if (id === request.current) { setPlan(result); setError(null); } })
        .catch(e => { if (id === request.current) { setPlan(null); setError(String(e)); } })
        .finally(() => { if (id === request.current) setLoading(false); });
    }, 400);
    return () => window.clearTimeout(timer);
  }, [active, rootPath, saved, statsToken, busy]);

  const apply = useCallback(async () => {
    if (!plan || !plan.entries.length || busy) return;
    const moves = plan.entries.length - plan.trashCount;
    const text = `${formatCount(moves)} dosya taşınacak${plan.trashCount ? `, ${formatCount(plan.trashCount)} dosya Geri Dönüşüm Kutusu'na gönderilecek` : ""}. `
      + "Hiçbir dosyanın üstüne yazılmaz; aynı ad varsa numaralandırılır. Taşımalar İşlem Geçmişi'nden geri alınabilir.";
    if (!await confirmDialog(text, "Kuralları uygula", "Uygula")) return;
    if (plan.trashCount && !await confirmDialog(
      `${formatCount(plan.trashCount)} dosya Windows Geri Dönüşüm Kutusu'na gönderilecek. Bu kısım İşlem Geçmişi'nden geri alınamaz; dosyaları Geri Dönüşüm Kutusu'ndan geri yükleyebilirsin.`,
      "Geri Dönüşüm Kutusu", "Gönder")) return;
    setBusy(true);
    setFileOperationBusy(true);
    try {
      setMoving({ done: 0, total: plan.entries.length });
      const out = await applyRules(plan.id, plan.trashCount > 0, p => setMoving({ done: p.filesDone, total: p.totalFiles }));
      setLastId(out.moved.length ? plan.id : null);
      const failed = out.failed.length ? ` ${formatCount(out.failed.length)} dosya işlenemedi: ${out.failed[0].error}` : "";
      const trashed = out.trashed ? `, ${formatCount(out.trashed)} dosya Geri Dönüşüm Kutusu'na gönderildi` : "";
      setStatus({ type: out.failed.length ? "error" : "success", text: `${formatCount(out.moved.length)} dosya taşındı${trashed}.${out.moved.length ? " Taşıma geri alınabilir." : ""}${failed}` });
    } catch (e) {
      setStatus({ type: "error", text: String(e) });
    } finally {
      setPlan(null);
      setMoving(null);
      await refreshHistory();
      bumpStatsToken();
      setFileOperationBusy(false);
      setBusy(false);
    }
  }, [plan, busy, setStatus, setFileOperationBusy, refreshHistory, bumpStatsToken]);

  /** Yalnizca Gecmis'in en ustundeki kayit bu islemse geri alinir. */
  const canUndo = !!lastId && app.history[0]?.id === lastId;
  const undo = useCallback(async () => {
    if (!lastId) return;
    setLastId(null);
    await app.undoEntry(lastId);
  }, [lastId, app]);

  return { ...saved, plan, error, loading, moving, busy, addRule, removeRule, moveRule, patchRule, setRecursive, apply, canUndo, undo };
}

export type RuleEngineState = ReturnType<typeof useRuleEngine>;

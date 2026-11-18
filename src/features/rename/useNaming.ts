import { useCallback, useEffect, useRef, useState } from "react";
import { confirmDialog } from "../../lib/commands";
import { formatCount } from "../../lib/format";
import { applyNameRules, NAMING_DEFAULTS, previewNameRules, type NamePlan, type NameRuleOptions } from "./namingCommands";
import { getPref, setPref } from "../../lib/prefs";
import type { OrganizerState } from "../../types/app";

const KEY = "namingOptions";

/** Kural tabanli adlandirma: secenekler (`uiPrefs.namingOptions`), canli onizleme ve uygulama. */
export function useNaming(app: OrganizerState, active: boolean) {
  const [options, setOptionsState] = useState<NameRuleOptions>(() => ({ ...NAMING_DEFAULTS, ...getPref<Partial<NameRuleOptions>>(KEY) }));
  const [plan, setPlan] = useState<NamePlan | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState(false);
  /** Dosya basina ozel adlar (tam yol -> metin); kalici degil, klasor degisince ve uygulaninca temizlenir. */
  const [customNames, setCustomNames] = useState<Record<string, string>>({});
  const request = useRef(0);
  const { rootPath, excludeText, statsToken, setStatus, setFileOperationBusy } = app;

  const setCustom = useCallback((path: string, text: string) => setCustomNames(prev => {
    const next = { ...prev };
    if (text.trim()) next[path] = text; else delete next[path];
    return next;
  }), []);
  const clearCustom = useCallback(() => setCustomNames({}), []);
  useEffect(() => { setCustomNames({}); }, [rootPath]);

  const setOptions = useCallback((next: NameRuleOptions) => {
    setOptionsState(next);
    setPref(KEY, next);
  }, []);

  // Onizleme: secenek, klasor veya disk degisince yeniden hesaplanir. Yalnizca ekran acikken calisir; diske yazmaz.
  useEffect(() => {
    if (!active || !rootPath || busy) return;
    const id = ++request.current;
    setLoading(true);
    const timer = window.setTimeout(() => {
      const excludeFolders = excludeText.split(",").map(s => s.trim()).filter(Boolean);
      previewNameRules(rootPath, { ...options, excludeFolders, customNames })
        .then(result => { if (id === request.current) { setPlan(result); setError(null); } })
        .catch(e => { if (id === request.current) { setPlan(null); setError(String(e)); } })
        .finally(() => { if (id === request.current) setLoading(false); });
    }, 250);
    return () => window.clearTimeout(timer);
  }, [active, rootPath, excludeText, options, customNames, statsToken, busy]);

  const changing = plan?.entries.filter(e => e.to) ?? [];

  const apply = useCallback(async () => {
    const count = plan?.entries.filter(e => e.to).length ?? 0;
    if (!plan || !count || busy) return;
    const dirs = plan.entries.filter(e => e.to && e.isDir).length;
    const text = `${formatCount(dirs)} klasör ve ${formatCount(count - dirs)} dosyanın adı değişecek. Öğeler yerinde kalır, uzantılar korunur, hiçbir şeyin üstüne yazılmaz. `
      + "İşlem Geçmişi'nden geri alınabilir.";
    if (!await confirmDialog(text, "Adları uygula", "Adlandır")) return;
    setBusy(true);
    setFileOperationBusy(true);
    try {
      const done = await applyNameRules(plan.id);
      setStatus({ type: "success", text: `${formatCount(done)} ad değiştirildi. İşlem geri alınabilir.` });
    } catch (e) {
      setStatus({ type: "error", text: String(e) });
    } finally {
      setPlan(null);
      setCustomNames({});
      await app.refreshHistory();
      app.bumpStatsToken(); // Harita ve onizleme yeni adlari gorur.
      setFileOperationBusy(false);
      setBusy(false);
    }
  }, [plan, busy, app, setStatus, setFileOperationBusy]);

  const customCount = Object.keys(customNames).length;
  return { options, setOptions, plan, error, loading, busy, changing, apply, customNames, customCount, setCustom, clearCustom };
}

export type NamingState = ReturnType<typeof useNaming>;

import { useCallback, useEffect, useRef, useState } from "react";
import { applyRenamePlan, previewTemplateRename } from "../rules/automationCommands";
import { confirmDialog } from "../../lib/commands";
import { formatCount } from "../../lib/format";
import { parseExtensions } from "../rules/organizeSettings";
import type { RenamePlan } from "../../lib/fileOperations";
import type { OrganizerState } from "../../types/app";

export const TOKENS = [
  ["{ad}", "Mevcut ad"], ["{sıra}", "Sıra numarası"], ["{tarih}", "2026-03-15"],
  ["{yıl}", "Yıl"], ["{ay}", "Ay"], ["{gün}", "Gün"], ["{klasör}", "Bulunduğu klasör"],
] as const;

/** Sablonla toplu adlandirma: form durumu, onizleme ve uygulama. */
export function useTemplateRename(app: OrganizerState) {
  const [template, setTemplate] = useState("Belge_{yıl}-{ay}_{sıra}");
  const [extText, setExtText] = useState("pdf");
  const [start, setStart] = useState(1);
  const [pad, setPad] = useState(3);
  const [sort, setSort] = useState<"name" | "date">("date");
  const [plan, setPlan] = useState<RenamePlan | null>(null);
  const [busy, setBusy] = useState(false);
  const lock = useRef(false);
  const { rootPath, setStatus, setFileOperationBusy } = app;

  // Girdi degisince onizleme gecersizdir.
  useEffect(() => { setPlan(null); }, [template, extText, start, pad, sort, rootPath, app.includeSubdirs, app.excludeText]);

  const guard = useCallback(async (work: () => Promise<void>) => {
    if (lock.current) return;
    lock.current = true;
    setBusy(true);
    setFileOperationBusy(true);
    try { await work(); } catch (e) { setStatus({ type: "error", text: String(e) }); } finally {
      lock.current = false;
      setBusy(false);
      setFileOperationBusy(false);
    }
  }, [setStatus, setFileOperationBusy]);

  const preview = useCallback(() => guard(async () => {
    setPlan(await previewTemplateRename(rootPath, {
      template, start, pad, sort, extensions: parseExtensions(extText),
      includeSubdirs: app.includeSubdirs, excludeFolders: app.excludeText.split(",").map(s => s.trim()).filter(Boolean),
    }));
  }), [guard, rootPath, template, start, pad, sort, extText, app.includeSubdirs, app.excludeText]);

  const apply = useCallback(async () => {
    if (!plan) return;
    const count = plan.entries.filter(e => e.to).length;
    if (!await confirmDialog(`${formatCount(count)} dosyanın adı değişecek. Dosyalar klasörlerinde kalır, uzantılar korunur. İşlem Geçmişi'nden geri alınabilir.`, "Toplu adlandır", "Adlandır")) return;
    await guard(async () => {
      try {
        const done = await applyRenamePlan(plan.id);
        setStatus({ type: "success", text: `${formatCount(done)} dosya yeniden adlandırıldı. İşlem geri alınabilir.` });
      } finally {
        setPlan(null);
        await app.refreshHistory();
        app.bumpStatsToken();
      }
    });
  }, [plan, guard, app, setStatus]);

  const insertToken = useCallback((token: string) => setTemplate(t => t + token), []);

  return { template, setTemplate, extText, setExtText, start, setStart, pad, setPad, sort, setSort, plan, busy, preview, apply, insertToken, reset: () => setPlan(null) };
}

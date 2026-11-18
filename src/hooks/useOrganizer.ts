import { useCallback, useEffect, useRef, useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { cancelOrganize, confirmDialog } from "../lib/commands";
import { applyOrganize, exportOrganizeCsv, previewOrganize } from "../features/rules/organizeCommands";
import { formatBytes, formatCount } from "../lib/format";
import type { OrganizerState } from "../types/app";
import type { ScanProgress } from "../types/file";
import type { OrganizeOutcome, OrganizePlan } from "../types/organize";
import type { OrganizeSettings } from "../features/rules/useOrganizeSettings";

/** "Türüne Göre Düzenle" sekmesinin akisi: onizleme, uygulama, CSV, sonuc. */
export function useOrganizer(app: OrganizerState, settings: OrganizeSettings) {
  const [plan, setPlan] = useState<OrganizePlan | null>(null);
  const [outcome, setOutcome] = useState<OrganizeOutcome | null>(null);
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const [busy, setBusy] = useState<"preview" | "apply" | null>(null);
  const lock = useRef(false);
  const { rootPath, setFileOperationBusy, setStatus } = app;
  const { toggles, rulesForBackend } = settings;

  // Klasor veya ayar degisince eski onizleme gecersizdir.
  useEffect(() => { setPlan(null); }, [rootPath, toggles, rulesForBackend]);
  useEffect(() => { setOutcome(null); }, [rootPath]);

  const preview = useCallback(async () => {
    if (!rootPath || lock.current || settings.rulesError) return;
    lock.current = true;
    setBusy("preview");
    setFileOperationBusy(true);
    setOutcome(null);
    setProgress(null);
    try {
      const result = await previewOrganize(rootPath, {
        ...toggles,
        rules: rulesForBackend,
        includeSubdirs: app.includeSubdirs,
        includeOther: app.includeOther,
        keepPolicy: app.keepPolicy,
        excludeFolders: app.excludeText.split(",").map(s => s.trim()).filter(Boolean),
        bannedExtensions: app.bannedExtensions,
        similarityThreshold: app.thresholdPercent / 100,
      }, setProgress);
      setPlan(result);
    } catch (e) {
      const text = String(e);
      setStatus(text.includes("TARAMA_DURDURULDU") ? { type: "info", text: "Önizleme durduruldu." } : { type: "error", text });
    } finally {
      lock.current = false;
      setBusy(null);
      setProgress(null);
      setFileOperationBusy(false);
    }
  }, [rootPath, toggles, rulesForBackend, settings.rulesError, app.includeSubdirs, app.includeOther, app.keepPolicy, app.excludeText, app.bannedExtensions, app.thresholdPercent, setFileOperationBusy, setStatus]);

  const apply = useCallback(async () => {
    if (!plan || lock.current) return;
    const moving = plan.entries.filter(e => e.action === "Move").length;
    const quarantine = plan.entries.filter(e => e.action === "Quarantine").length;
    const ok = await confirmDialog(
      `${formatCount(moving)} dosya tür klasörlerine, ${formatCount(quarantine)} birebir kopya karantinaya taşınacak.${plan.removeEmptyDirs ? " Boşalan klasörler kaldırılacak." : ""}\n\nHiçbir dosya silinmez ve üzerine yazılmaz. İşlem, İşlem Geçmişi'nden tek adımda geri alınabilir.`,
      "Klasörü düzenle", "Düzenle",
    );
    if (!ok) return;
    lock.current = true;
    setBusy("apply");
    setFileOperationBusy(true);
    try {
      const result = await applyOrganize(plan.id);
      setOutcome(result);
      setPlan(null);
      setStatus({
        type: result.failed.length ? "info" : "success",
        text: `Düzenleme tamamlandı: ${formatCount(result.moved)} dosya (${formatBytes(result.movedBytes)}) taşındı, ${formatCount(result.quarantined)} kopya karantinaya alındı.`
          + `${result.removedDirs ? ` ${formatCount(result.removedDirs)} boş klasör kaldırıldı.` : ""}${result.failed.length ? ` ${result.failed.length} dosya taşınamadı.` : ""}`,
      });
    } catch (e) {
      setStatus({ type: "error", text: String(e) });
      setPlan(null);
    } finally {
      lock.current = false;
      setBusy(null);
      setFileOperationBusy(false);
      await app.refreshHistory();
      app.bumpStatsToken();
      if (app.scanResult) await app.rescan(rootPath);
    }
  }, [plan, app, rootPath, setFileOperationBusy, setStatus]);

  const exportCsv = useCallback(async () => {
    if (!plan) return;
    try {
      const target = await save({ title: "Önizleme raporunu kaydet", defaultPath: "duzenleme-onizleme.csv", filters: [{ name: "CSV", extensions: ["csv"] }] });
      if (!target) return;
      const rows = await exportOrganizeCsv(target, plan.entries);
      setStatus({ type: "success", text: `Önizleme raporu kaydedildi · ${formatCount(rows)} satır → ${target}` });
    } catch (e) {
      setStatus({ type: "error", text: String(e) });
    }
  }, [plan, setStatus]);

  const stop = useCallback(() => { void cancelOrganize(); }, []);
  const reset = useCallback(() => { setPlan(null); setOutcome(null); }, []);

  return { plan, outcome, progress, busy, preview, apply, exportCsv, stop, reset };
}

export type OrganizerHook = ReturnType<typeof useOrganizer>;

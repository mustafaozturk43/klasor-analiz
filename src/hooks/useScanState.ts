import { useCallback, useMemo, useRef, useState } from "react";
import * as api from "../lib/commands";
import { formatCount, parentPath, skippedNote } from "../lib/format";
import { pushRecent } from "../lib/recent";
import { useBannedExtensions } from "./useBannedExtensions";
import type { ScanProgress, ScanResult } from "../types/file";
import type { StatusMessage } from "../types/ui";

/**
 * Tarama tarafinin tum durumu: klasor secimi, tarama ayarlari,
 * canli ilerleme (Channel) ve drill-down gezinme.
 */
export function useScanState(
  setStatus: (s: StatusMessage | null) => void,
  clearSelection: () => void,
) {
  const [rootPath, setRootPath] = useState("");
  /** Kullanicinin diyalogdan sectigi ilk klasor (drill-down siniri). */
  const [baseRoot, setBaseRoot] = useState("");
  const [scanResult, setScanResult] = useState<ScanResult | null>(null);
  const [includeSubdirs, setIncludeSubdirs] = useState(true);
  const [thresholdPercent, setThresholdPercent] = useState(82);
  const [excludeText, setExcludeText] = useState("");
  const [isScanning, setIsScanning] = useState(false);
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const [statsToken, setStatsToken] = useState(0);
  /** Iki klasor karsilastirma modu (FAZ 3). */
  const [compareMode, setCompareMode] = useState(false);
  const [rootB, setRootB] = useState("");
  const scanLock = useRef(false);
  /** "Diğer dosyaları da tara" (FAZ 2). */
  const [includeOther, setIncludeOther] = useState(false);
  const banned = useBannedExtensions();
  const excludeFolders = useMemo(
    () => excludeText.split(",").map((s) => s.trim()).filter(Boolean),
    [excludeText],
  );
  const groups = useMemo(() => scanResult?.groups ?? [], [scanResult]);
  const bumpStatsToken = useCallback(() => setStatsToken((t) => t + 1), []);

  /** Ortak tarama cagrisi: ikinci kok verilirse A↔B karsilastirma, yoksa tek klasor. */
  const invokeScan = useCallback(
    async (path: string, secondRoot: string | null, onProgress: (p: ScanProgress) => void) => {
      if (scanLock.current) throw new Error("Bir tarama zaten sürüyor.");
      scanLock.current = true;
      const base = {
        rootPath: path,
        includeSubdirs,
        thresholdPercent,
        excludeFolders,
        bannedExtensions: banned.bannedExtensions,
        includeOther,
        onProgress,
      };
      try { return await (secondRoot ? api.compareFolders({ ...base, rootB: secondRoot }) : api.scanAndGroup(base)); }
      finally { scanLock.current = false; }
    },
    [includeSubdirs, thresholdPercent, excludeFolders, banned.bannedExtensions, includeOther],
  );
  /** Sessiz yeniden tarama (tasma/undo sonrasi); karsilastirma modunu korur. */
  const rescan = useCallback(
    async (path: string) => {
      if (scanLock.current) return;
      setIsScanning(true);
      try {
        setScanResult(null);
        setScanResult(await invokeScan(path, compareMode && rootB ? rootB : null, () => {}));
      } catch (e) {
        setStatus({ type: "error", text: String(e) });
      } finally {
        setIsScanning(false);
      }
    },
    [invokeScan, compareMode, rootB, setStatus],
  );
  /** Tarama hatasi mesajini durdurma/gercek hata olarak ayirir. */
  const reportError = useCallback(
    (e: unknown) => {
      const msg = String(e);
      const text = msg.includes("TARAMA_DURDURULDU") ? "Tarama durduruldu." : msg;
      setStatus({ type: text === msg ? "error" : "info", text });
    },
    [setStatus],
  );
  const scanAt = useCallback(
    async (path: string) => {
      if (scanLock.current) return;
      setIsScanning(true);
      setScanResult(null);
      setProgress(null);
      setStatus({ type: "info", text: "Klasör taranıyor ve gruplanıyor…" });
      try {
        const result = await invokeScan(path, null, (p) =>
          setProgress(p.phase === "done" ? null : p),
        );
        pushRecent(path);
        setScanResult(result);
        clearSelection();
        setStatus({
          type: "success",
          text: `${formatCount(result.totalFiles)} dosya tarandı · ${formatCount(result.groups.length)} grup bulundu.${skippedNote(result.warnings)}`,
        });
      } catch (e) {
        reportError(e);
      } finally {
        setProgress(null);
        setIsScanning(false);
        bumpStatsToken();
      }
    },
    [invokeScan, reportError, setStatus, clearSelection, bumpStatsToken],
  );
  /** Devam eden taramayi durdurur (backend iptal bayragi). */
  const stopScan = useCallback(async () => {
    try {
      await api.cancelScan();
    } catch {
      // Yoksay: tarama zaten bitmis olabilir.
    }
  }, []);

  /** Iki klasoru karsilastirir: A = rootPath, B = rootB (tasima hedefi A). */
  const compareScan = useCallback(async () => {
    if (!rootPath || !rootB || isScanning || scanLock.current) return;
    setCompareMode(true);
    setIsScanning(true);
    setScanResult(null);
    setProgress(null);
    setStatus({ type: "info", text: "İki klasör karşılaştırılıyor…" });
    try {
      const result = await invokeScan(rootPath, rootB, (p) =>
        setProgress(p.phase === "done" ? null : p),
      );
      pushRecent(rootPath);
      pushRecent(rootB);
      setScanResult(result);
      clearSelection();
      const exactCount = result.groups.filter((g) => g.groupType === "Exact").length;
      setStatus({
        type: "success",
        text: `A↔B: ${formatCount(result.totalFiles)} dosya · ${formatCount(result.groups.length)} grup · ${formatCount(exactCount)} birebir kopya seçildi.${skippedNote(result.warnings)}`,
      });
    } catch (e) {
      reportError(e);
    } finally {
      setProgress(null);
      setIsScanning(false);
      bumpStatsToken();
    }
  }, [rootPath, rootB, isScanning, invokeScan, reportError, setStatus, clearSelection, bumpStatsToken]);
  const scan = useCallback(async () => {
    if (compareMode) await compareScan();
    else if (rootPath) await scanAt(rootPath);
  }, [compareMode, compareScan, rootPath, scanAt]);
  /** Karsilastirma icin ikinci klasoru secer. */
  const pickFolderB = useCallback(async () => {
    try {
      const picked = await api.pickFolder();
      if (!picked) return;
      setRootB(picked);
      setScanResult(null);
      setCompareMode(true);
      setStatus({
        type: "info",
        text: "Klasör B seçildi — 'Karşılaştır' ile birebir kopyaları bulun.",
      });
    } catch (e) {
      setStatus({ type: "error", text: String(e) });
    }
  }, [setStatus]);

  const pickFolder = useCallback(async () => {
    try {
      const picked = await api.pickFolder();
      if (!picked) return;
      setRootPath(picked);
      setBaseRoot(picked);
      setScanResult(null);
      clearSelection();
      bumpStatsToken();
      setStatus({ type: "info", text: "Klasör seçildi — taramayı başlatabilirsiniz." });
    } catch (e) {
      setStatus({ type: "error", text: String(e) });
    }
  }, [setStatus, clearSelection, bumpStatsToken]);

  /** Alt klasor grafiginden drill-down: klasoru ac ve otomatik tara. */
  const openFolder = useCallback(
    async (path: string) => {
      if (scanLock.current) return;
      setRootPath(path);
      setCompareMode(false);
      clearSelection();
      setStatus({ type: "info", text: "Klasör açılıyor ve taranıyor…" });
      await scanAt(path);
    },
    [scanAt, setStatus, clearSelection],
  );

  /** Secilen ilk klasorun (baseRoot) uzerine cikilamaz; drive kokunde de durur. */
  const canGoUp = rootPath !== baseRoot && parentPath(rootPath) !== null;

  const goUp = useCallback(
    async (current: string) => {
      if (current === baseRoot) return;
      const parent = parentPath(current);
      if (parent) await openFolder(parent);
    },
    [openFolder, baseRoot],
  );

  return {
    rootPath,
    baseRoot,
    canGoUp,
    scanResult,
    groups,
    includeSubdirs,
    thresholdPercent,
    excludeText,
    excludeFolders,
    isScanning,
    progress,
    statsToken,
    setIncludeSubdirs,
    setThresholdPercent,
    setExcludeText,
    bumpStatsToken,
    pickFolder,
    pickFolderB,
    scan,
    compareScan,
    stopScan,
    compareMode,
    setCompareMode: (value: boolean) => { setCompareMode(value); setScanResult(null); clearSelection(); },
    rootB,
    openFolder,
    openRecent: async (path: string) => { if (scanLock.current) return; setBaseRoot(path); await openFolder(path); },
    goUp,
    rescan,
    /** Yasakli uzanti listesi ve islemleri (F-B). */
    banned,
    /** "Diğer dosyaları da tara" (FAZ 2). */
    includeOther,
    setIncludeOther,
  };
}

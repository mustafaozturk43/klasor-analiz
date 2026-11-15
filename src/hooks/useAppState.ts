import { useCallback, useEffect, useMemo, useState } from "react";
import { suggestTagName } from "../lib/tagSuggest";
import type { FileGroup, FileListMode, KeepPolicy } from "../types/file";
import type { StatusMessage } from "../types/ui";
import { useMoveOps } from "./useMoveOps";
import { useScanState } from "./useScanState";

/**
 * Uygulamanin tum is mantigi durumu: tarama, tasma, secim ve etiket.
 * Ince orkestrator; detaylar useScanState ve useMoveOps icinde.
 */
export function useAppState() {
  const [status, setStatus] = useState<StatusMessage | null>(null);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [tagName, setTagName] = useState("");
  /** Exact grupta tutulacak dosya politikasi (F2). */
  const [keepPolicy, setKeepPolicy] = useState<KeepPolicy>("first");
  /** 500 MB+ dosyalar icin tam hash dogrulamasi (F3). */
  const [verifyLarge, setVerifyLarge] = useState(false);
  const [fileOperationBusy, setFileOperationBusy] = useState(false);
  const [groupNames, setGroupNames] = useState<Record<string, string>>({});
  /** Dosya listesi paneli acik mi (ScanSettings switch'i). */
  const [fileListEnabled, setFileListEnabled] = useState(false);
  /** "category" = sadece tarama kategorileri, "all" = tum dosyalar. */
  const [fileListMode, setFileListMode] = useState<FileListMode>("category");

  const clearSelection = useCallback(() => setSelectedIds(new Set()), []);
  const clearTagName = useCallback(() => setTagName(""), []);

  const scan = useScanState(setStatus, clearSelection);
  useEffect(() => {
    setGroupNames({});
    setSelectedIds(new Set(scan.scanResult?.comparison ? scan.scanResult.groups.filter(g => g.groupType === "Exact").map(g => g.id) : []));
  }, [scan.scanResult]);

  const selectedGroups = useMemo(
    () => scan.groups.filter((g) => selectedIds.has(g.id)),
    [scan.groups, selectedIds],
  );

  const move = useMoveOps({
    rootPath: scan.rootPath,
    rootB: scan.scanResult?.comparison?.rootB ?? "",
    groupNames,
    tagName,
    selectedGroups,
    keepPolicy,
    verifyLarge,
    clearTagName,
    rescan: scan.rescan,
    bumpStatsToken: scan.bumpStatsToken,
    onStatus: setStatus,
  });

  const toggleGroup = useCallback((id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  const toggleAllVisible = useCallback((visible: FileGroup[]) => {
    setSelectedIds((prev) => {
      const allSelected = visible.every((g) => prev.has(g.id));
      const next = new Set(prev);
      for (const g of visible) {
        if (allSelected) next.delete(g.id);
        else next.add(g.id);
      }
      return next;
    });
  }, []);

  /** Bir (veya birden fazla) grup icin otomatik etiket onerir ve uygular. */
  const autoTag = useCallback((targets: FileGroup[]) => {
    if (targets.length === 0) return;
    setSelectedIds(new Set(targets.map((g) => g.id)));
    setTagName(suggestTagName(targets.flatMap((g) => g.files)));
  }, []);

  const selectedFileCount = selectedGroups.reduce((sum, g) => sum + g.files.length, 0);

  return {
    // tarama
    rootPath: scan.rootPath,
    baseRoot: scan.baseRoot,
    canGoUp: scan.canGoUp,
    scanResult: scan.scanResult,
    groups: scan.groups,
    includeSubdirs: scan.includeSubdirs,
    thresholdPercent: scan.thresholdPercent,
    excludeText: scan.excludeText,
    isScanning: scan.isScanning,
    progress: scan.progress,
    statsToken: scan.statsToken,
    setIncludeSubdirs: scan.setIncludeSubdirs,
    setThresholdPercent: scan.setThresholdPercent,
    setExcludeText: scan.setExcludeText,
    pickFolder: scan.pickFolder,
    pickFolderB: scan.pickFolderB,
    scan: scan.scan,
    compareScan: scan.compareScan,
    stopScan: scan.stopScan,
    compareMode: scan.compareMode,
    setCompareMode: scan.setCompareMode,
    rootB: scan.rootB,
    openFolder: scan.openFolder,
    openRecent: scan.openRecent,
    goUp: scan.goUp,
    // "Diğer dosyaları da tara" (FAZ 2).
    includeOther: scan.includeOther,
    setIncludeOther: scan.setIncludeOther,
    // yasakli uzantilar
    bannedExtensions: scan.banned.bannedExtensions,
    addBannedExtension: scan.banned.addExtension,
    removeBannedExtension: scan.banned.removeExtension,
    resetBannedExtensions: scan.banned.resetExtensions,
    // tasma / gecmis
    isMoving: move.isMoving || fileOperationBusy,
    setFileOperationBusy,
    history: move.history,
    refreshHistory: move.refreshHistory,
    applyMove: move.applyMove,
    undoLast: move.undoLast,
    undoEntry: move.undoEntry,
    clearHistory: move.clearHistory,
    clearQuarantineFolder: move.clearQuarantineFolder,
    exportCsv: move.exportCsv,
    reveal: move.reveal,
    keepPolicy,
    setKeepPolicy,
    verifyLarge,
    setVerifyLarge,
    // dosya listesi paneli
    fileListEnabled,
    setFileListEnabled,
    fileListMode,
    setFileListMode,
    rescan: scan.rescan,
    bumpStatsToken: scan.bumpStatsToken,
    clearSelection,
    selectGroups: (type: "Exact" | "Similar") => setSelectedIds(new Set(scan.groups.filter(g => g.groupType === type).map(g => g.id))),
    // secim + etiket
    selectedIds,
    groupNames,
    setGroupName: (id: string, name: string) => setGroupNames(prev => ({ ...prev, [id]: name })),
    selectedGroups,
    selectedFileCount,
    tagName,
    setTagName,
    toggleGroup,
    toggleAllVisible,
    autoTag,
    // bildirim
    status,
    setStatus,
  };
}

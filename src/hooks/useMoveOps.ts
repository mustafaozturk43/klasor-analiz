import { useCallback, useRef, useState } from "react";
import * as api from "../lib/commands";
import { moveNamedGroups } from "../lib/fileOperations";
import { undoHistoryEntry } from "../features/map/healthCommands";
import { suggestTagName } from "../features/duplicates/tagSuggest";
import { formatCount } from "../lib/format";
import type { FileGroup, KeepPolicy, UndoRecord } from "../types/file";
import type { StatusMessage } from "../types/ui";

interface MoveOpsDeps {
  rootPath: string;
  rootB: string;
  groupNames: Record<string, string>;
  tagName: string;
  selectedGroups: FileGroup[];
  /** Exact grupta tutulacak dosya politikasi (F2). */
  keepPolicy: KeepPolicy;
  /** 500 MB+ dosyalar icin tam hash dogrulamasi (F3). */
  verifyLarge: boolean;
  clearTagName: () => void;
  rescan: (path: string) => Promise<void>;
  bumpStatsToken: () => void;
  onStatus: (s: StatusMessage | null) => void;
}

/**
 * Tasma ve is gecmisi operasyonlari: etiketle-tasi, cok adimli undo,
 * karantina temizligi, CSV rapor, Explorer'da gosterme.
 */
export function useMoveOps(deps: MoveOpsDeps) {
  const {
    rootPath,
    rootB,
    groupNames,
    tagName,
    selectedGroups,
    keepPolicy,
    verifyLarge,
    clearTagName,
    rescan,
    bumpStatsToken,
    onStatus,
  } = deps;

  const [isMoving, setIsMoving] = useState(false);
  const operationLock = useRef(false);
  const [history, setHistory] = useState<UndoRecord[]>([]);

  const refreshHistory = useCallback(async () => {
    try {
      setHistory(await api.getMoveHistory());
    } catch (e) {
      onStatus({ type: "error", text: `İşlem geçmişi okunamadı: ${String(e)}` });
    }
  }, [onStatus]);

  /** Seçili grupları taşır; `only` verilirse yalnızca o grubu, kendi etiketiyle taşır. */
  const applyMove = useCallback(async (only?: FileGroup) => {
    const tag = only ? "" : tagName.trim();
    const picked = only ? [only] : selectedGroups;
    if (picked.length === 0 || operationLock.current) return;
    const batches = picked.map(group => ({ group, tag: tag || (groupNames[group.id] ?? suggestTagName(group.files)) }));
    setIsMoving(true);
    operationLock.current = true;
    let attempted = false;
    try {
      const preview = batches.slice(0, 8).map(b => `${b.group.id} → ${b.tag} (${b.group.files.length} dosya)`).join("\n");
      if (!await api.confirmDialog(`${preview}${batches.length > 8 ? "\n…" : ""}\n\nHedef: ${rootPath}\nExact: ana dosya etikete, kopyalar karantinaya. Similar: tüm üyeler etikete. Mevcut dosyaların üzerine yazılmaz. Onaylıyor musunuz?`, "Taşıma ön izlemesi", "Onayla ve taşı")) return;
      attempted = true;
      const total = await moveNamedGroups(rootPath, rootB, batches, keepPolicy, verifyLarge);
      clearTagName();
      onStatus({ type: "success", text: `${formatCount(total)} dosya taşındı. İşlem geri alınabilir.` });
    } catch (e) {
      onStatus({ type: "error", text: String(e) });
    } finally {
      if (attempted) { await rescan(rootPath); bumpStatsToken(); await refreshHistory(); }
      operationLock.current = false;
      setIsMoving(false);
    }
  }, [tagName, selectedGroups, rootPath, rootB, groupNames, keepPolicy, verifyLarge, clearTagName, onStatus, rescan, bumpStatsToken, refreshHistory]);

  const undoLast = useCallback(async () => {
    if (operationLock.current) return;
    operationLock.current = true;
    setIsMoving(true);
    try {
      const undone = await api.undoLastMove();
      if (undone) {
        onStatus({
          type: "success",
          text: `“${undone.tag}” işlemi geri alındı (${formatCount(undone.moves.length)} dosya yerine taşındı).`,
        });
        await rescan(rootPath);
      } else {
        onStatus({ type: "info", text: "Geri alınacak bir taşıma bulunmuyor." });
      }
    } catch (e) {
      onStatus({ type: "error", text: String(e) });
    } finally {
      bumpStatsToken();
      await refreshHistory();
      operationLock.current = false;
      setIsMoving(false);
    }
  }, [rootPath, rescan, onStatus, bumpStatsToken, refreshHistory]);

  /** Gecmisten secilen islemi geri alir (daha yeni islemlerle cakismiyorsa). */
  const undoEntry = useCallback(async (id: string) => {
    if (operationLock.current) return;
    operationLock.current = true;
    setIsMoving(true);
    try {
      const undone = await undoHistoryEntry(id);
      onStatus({ type: "success", text: `“${undone.tag}” işlemi geri alındı (${formatCount(undone.moves.length)} dosya yerine taşındı).` });
      await rescan(rootPath);
    } catch (e) {
      onStatus({ type: "error", text: String(e) });
    } finally {
      bumpStatsToken();
      await refreshHistory();
      operationLock.current = false;
      setIsMoving(false);
    }
  }, [rootPath, rescan, onStatus, bumpStatsToken, refreshHistory]);

  const clearHistory = useCallback(async () => {
    if (operationLock.current) return;
    operationLock.current = true;
    setIsMoving(true);
    try {
      const ok = await api.confirmDialog(
        "Tüm işlem geçmişi silinecek. Önceki işlemler artık geri alınamaz. Onaylıyor musunuz?",
        "Geçmişi Temizle", "Sil",
      );
      if (!ok) return;
      await api.clearMoveHistory();
      setHistory([]);
      onStatus({ type: "success", text: "İşlem geçmişi temizlendi." });
    } catch (e) {
      onStatus({ type: "error", text: String(e) });
    } finally {
      operationLock.current = false;
      setIsMoving(false);
    }
  }, [onStatus]);

  const clearQuarantineFolder = useCallback(async () => {
    if (!rootPath || operationLock.current) return;
    operationLock.current = true;
    setIsMoving(true);
    let attempted = false;
    try {
      const ok = await api.confirmDialog(
        "_Kopyalar_Karantina klasörü tüm kopyalarıyla Windows Geri Dönüşüm Kutusu’na gönderilecek. Uygulamadan geri alınamaz; dosyalar Geri Dönüşüm Kutusu’ndan geri yüklenebilir.",
        "Karantinayı Temizle", "Kutuya gönder",
      );
      if (!ok) return;
      attempted = true;
      await api.clearQuarantine(rootPath);
      onStatus({ type: "success", text: "Karantina Geri Dönüşüm Kutusu’na gönderildi." });
      await rescan(rootPath);
    } catch (e) {
      onStatus({ type: "error", text: String(e) });
    } finally {
      if (attempted) bumpStatsToken();
      operationLock.current = false;
      setIsMoving(false);
    }
  }, [rootPath, onStatus, rescan, bumpStatsToken]);

  const exportCsv = useCallback(
    async (groups: FileGroup[]) => {
      if (groups.length === 0) return;
      try {
        const target = await api.pickCsvTarget("smart-file-organizer-rapor.csv");
        if (!target) return;
        const rows = await api.writeGroupsCsv(target, groups);
        onStatus({
          type: "success",
          text: `CSV rapor kaydedildi · ${formatCount(rows)} satır → ${target}`,
        });
      } catch (e) {
        onStatus({ type: "error", text: String(e) });
      }
    },
    [onStatus],
  );

  const reveal = useCallback(
    async (path: string) => {
      try {
        await api.revealInExplorer(path);
      } catch (e) {
        onStatus({ type: "error", text: String(e) });
      }
    },
    [onStatus],
  );

  return {
    isMoving,
    history,
    refreshHistory,
    applyMove,
    undoLast,
    undoEntry,
    clearHistory,
    clearQuarantineFolder,
    exportCsv,
    reveal,
  };
}

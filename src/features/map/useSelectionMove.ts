import { useCallback, useState } from "react";
import type { OrganizerState } from "../../types/app";
import { undoLastMove } from "../../lib/commands";
import { formatCount } from "../../lib/format";
import type { MoveOutcome } from "../today/inboxCommands";
import { applySelectionMove, previewSelectionMove, type SelectionMovePlan, type SelectionMoveRequest } from "./selectionCommands";

/** Secilenleri tasima: on izle → uygula → (istenirse) son islemi geri al. Hatalar durum cubuguna yazilir. */
export function useSelectionMove(app: OrganizerState) {
  const [busy, setBusy] = useState(false);
  const { setStatus, refreshHistory, bumpStatsToken } = app;

  const run = useCallback(async <T,>(fn: () => Promise<T>): Promise<T | null> => {
    setBusy(true);
    try {
      return await fn();
    } catch (e) {
      setStatus({ type: "error", text: String(e) });
      return null;
    } finally {
      setBusy(false);
    }
  }, [setStatus]);

  const refresh = useCallback(async () => { await refreshHistory(); bumpStatsToken(); }, [refreshHistory, bumpStatsToken]);

  const preview = useCallback((req: SelectionMoveRequest) => run(() => previewSelectionMove(req)), [run]);

  const apply = useCallback((plan: SelectionMovePlan) => run(async (): Promise<MoveOutcome> => {
    const out = await applySelectionMove(plan.id);
    await refresh();
    if (out.failed.length) {
      setStatus({ type: "error", text: `${formatCount(out.moved.length)} dosya taşındı; ${formatCount(out.failed.length)} dosya taşınamadı: ${out.failed.slice(0, 3).map(f => `${f.path} (${f.error})`).join(", ")}` });
    }
    return out;
  }), [run, refresh, setStatus]);

  const undo = useCallback(() => run(async () => {
    const undone = await undoLastMove();
    await refresh();
    setStatus(undone
      ? { type: "success", text: `“${undone.tag}” işlemi geri alındı (${formatCount(undone.moves.length)} dosya).` }
      : { type: "info", text: "Geri alınacak bir işlem bulunmuyor." });
  }), [run, refresh, setStatus]);

  return { busy, preview, apply, undo };
}

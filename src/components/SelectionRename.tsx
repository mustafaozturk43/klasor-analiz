import { useCallback } from "react";
import type { OrganizerState } from "../types/app";
import type { BrowserEntry } from "../lib/browserView";
import { applyRenamePlan } from "../lib/automationCommands";
import type { RenamePlan } from "../lib/fileOperations";
import { effectiveDate } from "../lib/fileKinds";
import { formatCount } from "../lib/format";
import type { ListRenameOptions } from "../lib/inboxCommands";
import { previewSelectionRename } from "../lib/selectionCommands";
import TemplateRenameDialog from "./TemplateRenameDialog";

const TOKENS = ["{Grup}", "{OzelMetin}", "{tarih}", "{sıra}", "{ad}", "{yıl}", "{ay}", "{gün}"];

interface Props {
  app: OrganizerState;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  roots: string[];
  selected: BrowserEntry[];
  takenOf: (path: string) => number | null | undefined;
  /** `{Grup}` degeri: tasinacak son klasorun adi. */
  groupLabel: string;
  canMove: boolean;
  /** Adlandirmadan sonra eski → yeni yollar. */
  onRenamed: (renamed: Map<string, string>) => void;
  onMoveRenamed: (paths: string[]) => Promise<void>;
}

/** Secilenleri sablonla adlandirma; sira numarasi cekim/degisiklik tarihine gore (eskiden yeniye). */
export default function SelectionRename({ app, open, onOpenChange, roots, selected, takenOf, groupLabel, canMove, onRenamed, onMoveRenamed }: Props) {
  const ordered = [...selected].sort((a, b) =>
    (effectiveDate(a.file.modifiedAt, takenOf(a.file.path)) ?? 0) - (effectiveDate(b.file.modifiedAt, takenOf(b.file.path)) ?? 0)
    || a.file.name.localeCompare(b.file.name, "tr"));
  const pathsKey = ordered.map(e => e.file.path).join("\n");
  const datesKey = ordered.map(e => takenOf(e.file.path) ?? "").join(",");
  const rootsKey = roots.join("\n");
  const { refreshHistory, bumpStatsToken, setStatus } = app;

  const preview = useCallback((o: ListRenameOptions) => {
    const paths = pathsKey.split("\n");
    const dates = datesKey.split(",").map(d => (d ? Number(d) : null));
    return previewSelectionRename(rootsKey.split("\n"), paths, { ...o, group: groupLabel, dates, tidy: true });
  }, [pathsKey, datesKey, rootsKey, groupLabel]);

  const apply = useCallback(async (plan: RenamePlan) => {
    try {
      const n = await applyRenamePlan(plan.id);
      const renamed = new Map(plan.entries.filter(e => e.to).map(e => [e.from, e.to as string]));
      onRenamed(renamed);
      setStatus({ type: "success", text: `${formatCount(n)} dosya yeniden adlandırıldı. İşlem Geçmişi’nden geri alınabilir.` });
      await refreshHistory();
      bumpStatsToken();
      return renamed;
    } catch (e) {
      setStatus({ type: "error", text: String(e) });
      return null;
    }
  }, [onRenamed, refreshHistory, bumpStatsToken, setStatus]);

  return <TemplateRenameDialog open={open} onOpenChange={onOpenChange} count={selected.length} busy={app.isMoving || app.isScanning}
    tokens={TOKENS} defaultTemplate="{Grup}_{tarih}_{sıra}"
    description="Uzantılar korunur, dosyalar bulundukları klasörde kalır. İşlem Geçmişi’nden geri alınabilir."
    hint={<>{"{tarih}, {yıl}, {ay}, {gün}"}: fotoğraf ve videolarda çekim tarihi, diğer dosyalarda değiştirme tarihi. {"{Grup}"}: taşınacak klasörün adı (“{groupLabel || "—"}”). Sıra numarası tarihe göre verilir (eskiden yeniye).</>}
    preview={preview} apply={async plan => { await apply(plan); }}
    secondary={{ label: "Adlandır, sonra taşı…", disabled: !canMove, title: canMove ? undefined : "Taşımak için önce geçerli bir hedef ve grup adı yazın.",
      run: async plan => { const renamed = await apply(plan); if (renamed) await onMoveRenamed(selected.map(e => renamed.get(e.file.path) ?? e.file.path)); } }} />;
}

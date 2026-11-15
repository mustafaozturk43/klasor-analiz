import { useEffect, useMemo, useState } from "react";
import { X } from "lucide-react";
import type { OrganizerState } from "../types/app";
import type { FileSelection } from "../hooks/useFileSelection";
import type { useFolderBrowser } from "../hooks/useFolderBrowser";
import { useFolderTargets } from "../hooks/useFolderTargets";
import { useSelectionMove } from "../hooks/useSelectionMove";
import type { BrowserEntry } from "../lib/browserView";
import { effectiveDate } from "../lib/fileKinds";
import { formatCount } from "../lib/format";
import { leafName, resolveGroupTarget, suggestGroupName, type GroupTarget } from "../lib/groupPath";
import type { SelectionMovePlan } from "../lib/selectionCommands";
import SelectionBar from "./SelectionBar";
import SelectionMoveDialog from "./SelectionMoveDialog";
import SelectionRename from "./SelectionRename";
import TargetFolderSelect, { type RootOption } from "./TargetFolderSelect";

interface Props {
  app: OrganizerState;
  browser: ReturnType<typeof useFolderBrowser>;
  selection: FileSelection;
  entries: BrowserEntry[];
  datesReady: boolean;
  takenOf: (path: string) => number | null | undefined;
  thumbOf: (path: string) => string | null | undefined;
  requestThumbs: (paths: string[]) => void;
}

const lower = (s: string) => s.toLocaleLowerCase("tr");

/** Secim cubugu + hedef/grup durumu + tasima onayi + tasima sonrasi bildirim. Hedef secimi oturum boyunca korunur. */
export default function SelectionPanel({ app, browser, selection, entries, datesReady, takenOf, thumbOf, requestThumbs }: Props) {
  const [group, setGroup] = useState("");
  const [target, setTarget] = useState<string | null>(null);
  const [external, setExternal] = useState<string[]>([]);
  const [plan, setPlan] = useState<SelectionMovePlan | null>(null);
  const [renameOpen, setRenameOpen] = useState(false);
  const [notice, setNotice] = useState<{ id: string; text: string; dir: string; source: "A" | "B" | null } | null>(null);
  const move = useSelectionMove(app);
  const selected = useMemo(() => entries.filter(e => selection.selected.has(e.file.path)), [entries, selection.selected]);
  const roots: RootOption[] = useMemo(() => [{ label: "Klasör A", path: app.rootPath }, { label: "Klasör B", path: app.rootB }].filter(r => r.path), [app.rootPath, app.rootB]);
  const excludes = useMemo(() => app.excludeText.split(",").map(s => s.trim()).filter(Boolean), [app.excludeText]);
  const targets = useFolderTargets(roots.map(r => r.path), excludes, app.statsToken, selected.length > 0);
  const currentDir = browser.current?.path ?? (browser.scope === "B" ? app.rootB : browser.scope === "A" ? app.rootPath : "") ?? "";
  const base = target ?? (currentDir || null);
  const resolved: GroupTarget = base
    ? resolveGroupTarget(base, group, [...targets.folders, ...roots.map(r => r.path), ...external], currentDir)
    : { ok: false, error: false, message: "Bir hedef klasör seçin." };
  const suggestion = datesReady ? suggestGroupName(selected.map(e => effectiveDate(e.file.modifiedAt, takenOf(e.file.path))), target !== null) : null;
  const locked = app.isScanning || app.isMoving || move.busy;
  const miniKey = selected.slice(0, 8).map(e => e.file.path).join("\n");
  useEffect(() => { if (miniKey) requestThumbs(miniKey.split("\n")); }, [miniKey, requestThumbs]);

  const startMove = async (paths = selected.map(e => e.file.path)) => {
    if (!resolved.ok || !base) return;
    const p = await move.preview({ roots: roots.map(r => r.path), paths, targetBase: base, group });
    if (p) setPlan(p);
  };
  const confirm = async (p: SelectionMovePlan) => {
    const out = await move.apply(p);
    setPlan(null);
    if (!out) return;
    selection.clear();
    setGroup("");
    const root = roots.find(r => lower(p.targetDir).startsWith(lower(r.path.replace(/[\\/]+$/, "")) + "\\"));
    setNotice({ id: p.id, text: `${formatCount(out.moved.length)} dosya “${leafName(p.targetDir)}” klasörüne taşındı.`, dir: p.targetDir, source: root ? (root.label === "Klasör B" ? "B" : "A") : null });
  };

  return <>
    {notice && <div className="browser-notice" role="status">
      <span>{notice.text}</span>
      {notice.source && <button type="button" className="btn" onClick={() => { browser.enter(notice.dir, notice.source as "A" | "B"); setNotice(null); }}>Klasörü aç</button>}
      {/* Yalnizca Gecmis'in en ustundeki kayit bu tasimaysa: araya giren baska islem geri alinmasin. */}
      {app.history[0]?.id === notice.id && <button type="button" className="btn" disabled={locked} onClick={() => { setNotice(null); void move.undo(); }}>Geri al</button>}
      <button type="button" className="btn ghost" aria-label="Bildirimi kapat" onClick={() => setNotice(null)}><X size={14} /></button>
    </div>}
    {selected.length > 0 && <SelectionBar files={selected.map(e => e.file)} hiddenCount={selection.hiddenCount} thumbOf={thumbOf} suggestion={suggestion}
      target={<TargetFolderSelect value={target && lower(target) === lower(currentDir) ? null : target} currentDir={currentDir || null} roots={roots} folders={targets.folders} external={external}
        onChange={setTarget} onPickExternal={p => { setExternal(x => (x.includes(p) ? x : [...x, p])); setTarget(p); }} />}
      group={group} onGroup={setGroup} resolved={resolved} disabled={locked} onClear={selection.clear} onRename={() => setRenameOpen(true)} onMove={() => void startMove()} />}
    <SelectionRename app={app} open={renameOpen} onOpenChange={setRenameOpen} roots={roots.map(r => r.path)} selected={selected} takenOf={takenOf}
      groupLabel={resolved.ok ? leafName(resolved.target) : group.trim()} canMove={resolved.ok && !locked} onRenamed={selection.remap} onMoveRenamed={startMove} />
    <SelectionMoveDialog plan={plan} busy={move.busy} onCancel={() => setPlan(null)} onConfirm={p => void confirm(p)} />
  </>;
}

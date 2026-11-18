import { useMemo, useState } from "react";
import { FolderInput, PenLine } from "lucide-react";
import { formatCount } from "../../lib/format";
import { leafName, resolveGroupTarget } from "../duplicates/groupPath";
import type { InboxState } from "./useInbox";
import { useFolderTargets } from "../../hooks/useFolderTargets";
import InboxTrashDialog from "./InboxTrashDialog";
import TargetFolderSelect, { type RootOption } from "../../components/TargetFolderSelect";

interface Props { inbox: InboxState; onRename: () => void }

const SUGGESTED = "Önerilen klasörler (dosya başına)";
const parentOf = (p: string) => p.replace(/[\\/][^\\/]*$/, "");

/** Uygulama cubugu: "Tasi" dosyalarinin hedef klasoru (Haritalar'daki secici) + grup adi, adlandirma ve kararlarin uygulanmasi. */
export default function InboxActionBar({ inbox, onRename }: Props) {
  const [trashOpen, setTrashOpen] = useState(false);
  const [external, setExternal] = useState<string[]>([]);
  const { move, bin } = inbox.counts;
  const backup = inbox.settings?.backupRoot ?? "";
  const sources = useMemo(() => inbox.settings?.sourceFolders ?? [], [inbox.settings]);
  const roots: RootOption[] = useMemo(() => [
    ...(backup ? [{ label: "Yedek kökü", path: backup }] : []),
    ...sources.map(path => ({ label: "İzlenen klasör", path })),
  ], [backup, sources]);
  const folders = useFolderTargets(backup ? [backup] : [], ["_GelenKutusu"], 0, move > 0);
  // "Bu klasör": izlenen klasorde bekleyen ilk "Tasi" dosyasinin klasoru (gelen kutusu hedef olamaz).
  const pending = useMemo(() => new Set((inbox.listing?.groups ?? []).flatMap(g => g.files.filter(f => f.pending).map(f => f.path))), [inbox.listing]);
  const firstPending = inbox.movePaths.find(p => pending.has(p));
  const currentDir = firstPending ? parentOf(firstPending) : null;
  const chosen = inbox.defaultPath;
  const resolved = chosen ? resolveGroupTarget(chosen, inbox.group, [...folders.folders, ...roots.map(r => r.path), ...external], currentDir ?? "") : null;
  const total = move + bin;
  const needsTarget = inbox.missingTarget > 0;
  const parts = inbox.groupParts;
  const groupError = typeof parts === "string" ? parts : resolved && !resolved.ok ? resolved.message : null;
  const blocked = total === 0 || inbox.busy || (move > 0 && (needsTarget || !!groupError));
  const apply = () => { if (bin > 0) setTrashOpen(true); else void inbox.applyAll(); };

  return <section className="inbox-apply" aria-label="Kararları uygula">
    <div className="inbox-apply-text">
      <strong>{formatCount(move)} dosya taşınır, {formatCount(bin)} dosya Geri Dönüşüm Kutusu’na gider.</strong>
      <span>Taşımalar İşlem Geçmişi’nden geri alınır. Kutuya gidenler Windows Geri Dönüşüm Kutusu’ndan geri yüklenir.</span>
    </div>
    <div className="inbox-apply-dest">
      <label className="selection-field" htmlFor="inbox-target">Hedef klasör
        <TargetFolderSelect id="inbox-target" value={chosen} currentDir={currentDir} roots={roots} folders={folders.folders} external={external}
          nullLabel={SUGGESTED} rootLabel={r => `${r.label} · ${leafName(r.path)}`} disabled={move === 0}
          onChange={inbox.setDefaultPath} onPickExternal={p => { setExternal(x => (x.includes(p) ? x : [...x, p])); inbox.setDefaultPath(p); }} />
      </label>
      <label className="selection-field">Grup adı (alt klasör)
        <input id="inbox-group" type="text" value={inbox.group} autoComplete="off" placeholder="Örn. Mayıs ya da Faturalar\Mayıs" disabled={move === 0}
          onChange={e => inbox.setGroup(e.target.value)} /></label>
      <p className={`selection-path${groupError && (typeof parts === "string" || resolved?.ok === false && resolved.error) ? " err" : ""}`} aria-live="polite">
        {groupError ?? (needsTarget ? `${formatCount(inbox.missingTarget)} dosyanın önerilen klasörü yok. Bir hedef klasör seçin.` : <>Hedef: <span className="font-mono">{chosen ?? "her dosyanın önerilen klasörü"}</span>
          {(parts as string[]).map((p, i) => <span key={i} className="font-mono">\{p}</span>)}
          <span className="badge">{(parts as string[]).length ? "alt klasör yoksa oluşturulur" : "doğrudan bu klasöre"}</span></>)}
      </p>
    </div>
    <div className="inbox-apply-actions">
      <button type="button" className="btn" disabled={move === 0 || inbox.busy} onClick={onRename}><PenLine size={14} /> Adlandır…</button>
      <button type="button" className="btn primary inbox-apply-go" disabled={blocked} onClick={apply}><FolderInput size={16} /> {formatCount(total)} kararı uygula</button>
    </div>
    <InboxTrashDialog inbox={inbox} open={trashOpen} onOpenChange={setTrashOpen} onConfirm={() => inbox.applyAll()} />
  </section>;
}

import { useState } from "react";
import { Check, ChevronDown, FolderInput, FolderPlus, PenLine } from "lucide-react";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { formatCount } from "../lib/format";
import type { InboxState } from "../hooks/useInbox";
import InboxTrashDialog from "./InboxTrashDialog";

interface Props { inbox: InboxState; onRename: () => void }

/** Uygulama cubugu: "Tasi" dosyalarinin hedef klasoru + grup adi (alt klasor), adlandirma ve tum kararlarin uygulanmasi. */
export default function InboxActionBar({ inbox, onRename }: Props) {
  const [open, setOpen] = useState(false);
  const [trashOpen, setTrashOpen] = useState(false);
  const [newName, setNewName] = useState("");
  const [createError, setCreateError] = useState<string | null>(null);
  const { move, bin } = inbox.counts;
  const chosen = inbox.defaultTarget;
  const total = move + bin;
  const needsTarget = inbox.missingTarget > 0;
  const parts = inbox.groupParts;
  const groupError = typeof parts === "string" ? parts : null;
  const blocked = total === 0 || inbox.busy || (move > 0 && (needsTarget || !!groupError));

  const choose = (path: string | null) => { inbox.setDefaultTarget(inbox.targets.find(t => t.path === path) ?? null); setOpen(false); };
  const create = async () => {
    try {
      inbox.setDefaultTarget(await inbox.createTarget(newName));
      setNewName("");
      setCreateError(null);
      setOpen(false);
    } catch (e) {
      setCreateError(String(e));
    }
  };
  const apply = () => { if (bin > 0) setTrashOpen(true); else void inbox.applyAll(); };

  return <section className="inbox-apply" aria-label="Kararları uygula">
    <div className="inbox-apply-text">
      <strong>{formatCount(move)} dosya taşınır, {formatCount(bin)} dosya Geri Dönüşüm Kutusu’na gider.</strong>
      <span>Taşımalar İşlem Geçmişi’nden geri alınır. Kutuya gidenler Windows Geri Dönüşüm Kutusu’ndan geri yüklenir.</span>
    </div>
    <div className="inbox-apply-dest">
      <label className="selection-field" htmlFor="inbox-target">Hedef klasör
        <Popover open={open} onOpenChange={setOpen}>
          <PopoverTrigger id="inbox-target" className="target-trigger" aria-label="Hedef klasör" aria-haspopup="listbox" disabled={move === 0}>
            <span className="truncate">{chosen ? chosen.name : "Önerilen klasörler (dosya başına)"}</span><ChevronDown size={15} aria-hidden />
          </PopoverTrigger>
          <PopoverContent collisionPadding={8} className="target-list inbox-target-list">
            <div role="listbox" aria-label="Hedef klasör">
              <button type="button" role="option" aria-selected={!chosen} className="target-option" onClick={() => choose(null)}>
                <span className="truncate">Önerilen klasörler (dosya başına)</span>{!chosen && <Check size={14} aria-hidden />}
              </button>
              {inbox.targets.length > 0 && <div className="target-group">Yedek klasöründeki klasörler</div>}
              {inbox.targets.map(t => <button key={t.path} type="button" role="option" aria-selected={chosen?.path === t.path} className="target-option nested" title={t.path}
                onClick={() => choose(t.path)}><span className="truncate">{t.name}</span>{chosen?.path === t.path && <Check size={14} aria-hidden />}</button>)}
            </div>
            <form className="inbox-target-new" onSubmit={e => { e.preventDefault(); void create(); }}>
              <input id="inbox-new-folder" className="settings-group-name" value={newName} onChange={e => setNewName(e.target.value)} placeholder="Yeni klasör adı" aria-label="Yeni klasör adı" />
              <button type="submit" className="btn" disabled={!newName.trim()} aria-label="Klasörü oluştur"><FolderPlus size={14} /></button>
            </form>
            {createError && <p className="settings-error px-2 pb-1">{createError}</p>}
          </PopoverContent>
        </Popover>
      </label>
      <label className="selection-field">Grup adı (alt klasör)
        <input id="inbox-group" type="text" value={inbox.group} autoComplete="off" placeholder="Örn. Mayıs ya da Faturalar\Mayıs" disabled={move === 0}
          onChange={e => inbox.setGroup(e.target.value)} /></label>
      <p className={`selection-path${groupError ? " err" : ""}`} aria-live="polite">
        {groupError ?? (needsTarget ? `${formatCount(inbox.missingTarget)} dosyanın önerilen klasörü yok. Bir hedef klasör seçin.` : <>Hedef: <span className="font-mono">{chosen ? chosen.path : "her dosyanın önerilen klasörü"}</span>
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

import { useState } from "react";
import { CheckCircle2, FolderOpen, MoonStar, RefreshCw, Settings2, X } from "lucide-react";
import { useInbox } from "../hooks/useInbox";
import { openFileDefault } from "../lib/commands";
import { formatBytes, formatCount } from "../lib/format";
import { localDate } from "../lib/settingsCommands";
import type { OrganizerState } from "../types/app";
import InboxActionBar from "./InboxActionBar";
import InboxDetail from "./InboxDetail";
import InboxGroupList from "./InboxGroupList";
import InboxRenameDialog from "./InboxRenameDialog";
import InboxVerdictSummary from "./InboxVerdictSummary";

const label = (date: string) => new Date(`${date}T12:00:00`).toLocaleDateString("tr-TR", { weekday: "long", day: "numeric", month: "long" });
const short = (date: string) => new Date(`${date}T12:00:00`).toLocaleDateString("tr-TR", { weekday: "short", day: "2-digit", month: "2-digit", year: "numeric" });

interface Props { app: OrganizerState; active: boolean; onOpenSettings: () => void }

/** Bugun: gunun gelen kutusu. Her dosya icin Tut / Tasi / At karari verilir, sonra hepsi tek dokunusla uygulanir. */
export default function EveningPanel({ app, active, onOpenSettings }: Props) {
  const inbox = useInbox(app, active);
  const [renameOpen, setRenameOpen] = useState(false);
  const l = inbox.listing;
  const today = localDate();
  const days = l?.days ?? [];
  const showToday = !days.some(d => d.date === today);
  const [selected, setSelected] = useState<string | null>(null);
  const files = l?.groups.flatMap(g => g.files) ?? [];
  const shown = files.find(f => f.path === selected) ?? files[0] ?? null;
  const openFile = (path: string) => void openFileDefault(path).catch(e => inbox.setNotice({ type: "error", text: String(e) }));

  if (inbox.error && !l) {
    return <section className="panel"><div className="empty-state flex flex-col items-center gap-3">
      <MoonStar size={28} /><p>{inbox.error}</p>
      <button type="button" className="btn primary" onClick={onOpenSettings}><Settings2 size={14} /> Ayarlar’a git</button>
    </div></section>;
  }

  const has = !!l && l.totalFiles > 0;
  const actions = <div className="inbox-head-actions">
    {l?.exists && <button type="button" className="btn" onClick={() => void app.reveal(l.dir)}><FolderOpen size={15} /> Klasörü aç</button>}
    <button type="button" className="btn" onClick={() => void inbox.reload()} disabled={inbox.loading}><RefreshCw size={15} className={inbox.loading ? "animate-spin" : ""} /> Yenile</button>
  </div>;
  return <div className="organize-stack">
    {has && l
      ? <InboxVerdictSummary inbox={inbox} dateLabel={label(l.date)} actions={actions} />
      : <section className="inbox-summary"><div className="inbox-summary-head"><div><h1>Bakılacak dosya yok</h1>
        <p>{l ? (inbox.date ? "Bu günde bakılacak dosya kalmadı." : "Bugün henüz gelen dosya yok. Otomasyon açıkken kaynak klasörlere düşen dosyalar burada toplanır.") : "Yükleniyor…"}</p></div>{actions}</div></section>}

    <div className="inbox-toolbar">
      <p className="break-all">{l ? `${l.dir}${has ? `, ${formatCount(l.totalFiles)} dosya, ${formatBytes(l.totalBytes)}` : ""}` : ""}</p>
      <select id="inbox-day" aria-label="Gün" value={inbox.date ?? today} onChange={e => inbox.pickDate(e.target.value === today ? null : e.target.value)}>
        {showToday && <option value={today}>Bugün, {short(today)}</option>}
        {days.map(d => <option key={d.date} value={d.date}>{d.date === today ? "Bugün, " : ""}{short(d.date)}, {formatCount(d.files)} dosya</option>)}
      </select>
    </div>

    {inbox.notice && <p className={`inbox-notice ${inbox.notice.type}`} role="status">
      {inbox.notice.type === "ok" && <CheckCircle2 size={15} />}<span className="flex-1">{inbox.notice.text}</span>
      <button type="button" onClick={() => inbox.setNotice(null)} aria-label="Bildirimi kapat"><X size={14} /></button>
    </p>}
    {l?.truncated && <p className="inbox-notice error" role="status">Bu günde çok fazla dosya var; ilk 5.000 dosya gösteriliyor.</p>}

    {has && l && <>
      <div className="inbox-split">
        <InboxGroupList groups={l.groups} verdicts={inbox.verdicts} setVerdict={inbox.setVerdict} onReveal={p => void app.reveal(p)} selected={shown?.path ?? null} onSelect={setSelected} targetNameOf={p => inbox.targetOf(p)?.name ?? null} />
        <InboxDetail file={shown} verdict={shown ? inbox.verdicts[shown.path] ?? "keep" : "keep"} onReveal={p => void app.reveal(p)} onOpen={openFile} targetName={shown ? inbox.targetOf(shown.path)?.name ?? null : null} />
      </div>
      <InboxActionBar inbox={inbox} onRename={() => setRenameOpen(true)} />
    </>}

    <InboxRenameDialog inbox={inbox} open={renameOpen} onOpenChange={setRenameOpen} />
  </div>;
}

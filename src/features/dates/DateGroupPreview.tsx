import { CalendarDays, ExternalLink, Folder, RefreshCw, Undo2 } from "lucide-react";
import { UNDATED } from "./dateGroupCommands";
import { formatCount } from "../../lib/format";
import MoveProgress from "../../components/MoveProgress";
import type { DateGroupState } from "./useDateGroup";

interface Props { dg: DateGroupState; rootPath: string; locked: boolean }

const MONTHS = ["Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"];
const monthName = (folder: string) => (folder === UNDATED ? "" : MONTHS[Number(folder.split("\\")[1]) - 1] ?? "");

/** Onizleme: Yıl\Ay klasoru, dosya sayisi ve ornek adlar; altta uygulama ve geri alma. */
export default function DateGroupPreview({ dg, rootPath, locked }: Props) {
  const { plan, error, progress } = dg;
  const n = plan?.entries.length ?? 0;
  return <section className="panel naming-preview">
    <div className="panel-head"><h2><CalendarDays size={18} /> Önizleme</h2><span className="hint">Dosyalar seçili klasörün içinde Yıl\Ay klasörlerine taşınır.</span></div>
    {!rootPath ? <div className="empty-state">Gruplamak için yukarıdan bir klasör seçin.</div>
      : error ? <div className="error-state" role="alert">{error}</div>
      : !plan ? <div className="empty-state" role="status">Tarihler okunuyor{progress?.total ? ` · ${formatCount(progress.done)} / ${formatCount(progress.total)}` : "…"}</div>
      : !plan.buckets.length ? <div className="empty-state">{plan.inPlace ? "Dosyaların hepsi zaten doğru klasörde." : "Bu seçimle taşınacak dosya yok."}</div>
      : <ul className="date-group-list">
        {plan.buckets.map(b => <li key={b.folder} className="date-group-row" data-undated={b.folder === UNDATED || undefined}>
          <span className="naming-name"><Folder size={16} aria-hidden="true" /><span className="font-mono">{b.folder}</span></span>
          <span className="hint">{monthName(b.folder)}</span>
          <span className="date-group-samples" title={b.samples.join(", ")}>{b.samples.join(", ")}{b.count > b.samples.length ? ` +${formatCount(b.count - b.samples.length)}` : ""}</span>
          <span className="badge">{formatCount(b.count)} dosya</span>
        </li>)}
      </ul>}
    {dg.moving && <MoveProgress done={dg.moving.done} total={dg.moving.total} />}
    <div className="naming-foot">
      <span role="status">{plan && n ? `${formatCount(n)} dosya, ${formatCount(plan.buckets.length)} klasör · ${formatCount(plan.fromTaken)} çekim tarihinden`
        + `${plan.renamed ? ` · ${formatCount(plan.renamed)} ad dolu olduğu için numaralandı` : ""}${plan.inPlace ? ` · ${formatCount(plan.inPlace)} zaten yerinde` : ""}.` : ""}
        {progress && plan ? " Güncelleniyor…" : ""}</span>
      {rootPath && <>
        <button type="button" className="btn" onClick={() => void dg.openIndex()} disabled={locked} title="Tarayıcıda açar: hangi dosya hangi klasörde"><ExternalLink size={14} /> Dizini aç</button>
        <button type="button" className="btn" onClick={() => void dg.rebuildIndex()} disabled={locked} title="Dosya taşımaz; dizini klasörün şimdiki durumundan baştan yazar"><RefreshCw size={14} /> Dizini yenile</button>
      </>}
      {dg.canUndo && <button type="button" className="btn" onClick={() => void dg.undo()} disabled={locked}><Undo2 size={14} /> Geri al</button>}
      <button type="button" className="btn primary naming-go" onClick={() => void dg.apply()} disabled={locked || !!progress || !n}>
        {n ? `${formatCount(n)} dosyayı grupla` : "Gruplanacak dosya yok"}</button>
    </div>
  </section>;
}

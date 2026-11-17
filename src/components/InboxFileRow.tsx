import { memo } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { FolderOpen } from "lucide-react";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { formatBytes, PREVIEW_IMAGE_EXTS } from "../lib/format";
import { duplicateText } from "../lib/duplicateText";
import type { Verdict } from "../hooks/useInbox";
import type { FileDetail } from "../types/inbox";
import VerdictToggle from "./VerdictToggle";

const at = (ms: number) => new Date(ms).toLocaleTimeString("tr-TR", { hour: "2-digit", minute: "2-digit" });
const day = (ms: number) => new Date(ms).toLocaleDateString("tr-TR", { day: "2-digit", month: "2-digit", year: "numeric" });

interface Props {
  file: FileDetail;
  verdict: Verdict;
  onVerdict: (path: string, v: Verdict) => void;
  onReveal: (path: string) => void;
  selected: boolean;
  onSelect: (path: string) => void;
  /** Gidecegi klasorun adi ("Tasi" kararindaysa). */
  target: string | null;
}

/** Kucuk resim; tiklayinca buyuk onizleme (gorsel ya da metin ozeti) acilir. */
function Thumb({ file }: { file: FileDetail }) {
  const label = file.ext ? file.ext.slice(0, 4).toLowerCase() : "?";
  const box = "inbox-thumb";
  const face = file.thumbnailPath
    ? <img src={convertFileSrc(file.thumbnailPath)} alt="" loading="lazy" decoding="async" />
    : <span>{label}</span>;
  const canPreview = !!file.thumbnailPath || !!file.contentPreview;
  if (!canPreview) return <span className={box} aria-hidden="true">{face}</span>;
  return <Popover>
    <PopoverTrigger className={`${box} cursor-zoom-in`} aria-label={`${file.name} önizlemesini aç`}>{face}</PopoverTrigger>
    <PopoverContent side="right" className="max-w-[min(360px,90vw)] p-2">
      {PREVIEW_IMAGE_EXTS.has(file.ext)
        ? <img src={convertFileSrc(file.path)} alt={file.name} className="max-h-[320px] w-full rounded-md object-contain" />
        : <p className="whitespace-pre-wrap p-1 text-[12px] leading-relaxed">{file.contentPreview}</p>}
      <p className="mt-1.5 truncate px-1 text-[12px] text-[var(--color-muted)]">{file.name}</p>
    </PopoverContent>
  </Popover>;
}

/** Gelen kutusundaki bir dosya: kucuk resim, tarih rozetleri, icerik ozeti ve Tut / Tasi / At karari. */
function InboxFileRow({ file, verdict, onVerdict, onReveal, selected, onSelect, target }: Props) {
  return <div className="inbox-row" data-verdict={verdict} data-selected={selected || undefined}>
    <Thumb file={file} />
    <div className="min-w-0">
      <button type="button" className="inbox-name" title={file.name} aria-current={selected || undefined} onClick={() => onSelect(file.path)}>{file.name}</button>
      <span className="inbox-meta">
        {file.pending && <span className="badge pending" title="Otomatik taşıma kapalı; dosya izlenen klasörde bekliyor">İzlenen klasörde</span>}
        {file.arrivedMs !== null && <span className="badge">Geldi {at(file.arrivedMs)}</span>}
        {file.createdMs !== null && <span className="badge">Oluşturma {day(file.createdMs)}</span>}
        {file.modifiedMs !== null && <span className="badge">Değişiklik {day(file.modifiedMs)} {at(file.modifiedMs)}</span>}
        {file.accessedMs !== null && <span className="badge dim" title="Windows son erişim zamanını çoğu zaman güncellemez">Son erişim ≈ {day(file.accessedMs)}</span>}
        {verdict === "move" && <span className="badge target">{target ? `Hedef: ${target}` : "Hedef seçilmedi"}</span>}
        {file.duplicateOf &&<span className="badge exact" title={file.duplicateOf}>{duplicateText(file)}</span>}
        {file.similarTo &&<span className="badge warn" title="Adı benzer; içerik farklı olabilir">%{file.similarity} benzer: {file.similarTo}</span>}
      </span>
      {file.contentPreview && <span className="inbox-peek"><b>İçerik:</b> {file.contentPreview}</span>}
    </div>
    <span className="inbox-size">{formatBytes(file.size)}</span>
    <div className="inbox-side">
      <VerdictToggle value={verdict} onChange={v => onVerdict(file.path, v)} label={`${file.name} kararı`} />
      <button type="button" className="btn" onClick={() => onReveal(file.path)} aria-label={`${file.name} klasörde göster`}><FolderOpen size={14} /></button>
    </div>
  </div>;
}

export default memo(InboxFileRow);

import { convertFileSrc } from "@tauri-apps/api/core";
import { ExternalLink, FolderOpen } from "lucide-react";
import { duplicateNote } from "../lib/duplicateText";
import { formatBytes, PREVIEW_IMAGE_EXTS } from "../lib/format";
import type { Verdict } from "../hooks/useInbox";
import type { FileDetail } from "../types/inbox";

const LABEL: Record<Verdict, string> = { keep: "Tut", move: "Taşı", bin: "At" };
const SAYS: Record<Verdict, string> = {
  keep: "Dosya olduğu yerde kalır.",
  move: "Seçili hedef klasöre taşınır.",
  bin: "Windows Geri Dönüşüm Kutusu’na gider.",
};
const stamp = (ms: number | null, time: boolean) => ms === null ? "—" : new Date(ms).toLocaleString("tr-TR",
  time ? { day: "2-digit", month: "2-digit", year: "numeric", hour: "2-digit", minute: "2-digit" } : { day: "2-digit", month: "2-digit", year: "numeric" });

interface Props {
  file: FileDetail | null;
  verdict: Verdict;
  onReveal: (path: string) => void;
  onOpen: (path: string) => void;
  /** Dosyanin gidecegi klasor (alttaki cubuktan secilir); yoksa null. */
  targetName: string | null;
}

/** Secili dosyanin ayrintisi: tam yol, karar ve gerekcesi, hedef klasor, tarihler, benzerlik, onizleme, "Klasorde goster" / "Ac". */
export default function InboxDetail({ file, verdict, onReveal, onOpen, targetName }: Props) {
  if (!file) return <aside className="inbox-detail" aria-label="Dosya ayrıntısı"><p className="inbox-detail-empty">Ayrıntısını görmek için bir dosya seç.</p></aside>;
  const image = PREVIEW_IMAGE_EXTS.has(file.ext);
  const sug = file.suggestion;
  const agrees = sug?.verdict === verdict;
  return <aside className="inbox-detail" aria-label="Dosya ayrıntısı">
    <div>
      <h2 className="inbox-detail-name">{file.name}</h2>
      <p className="inbox-detail-path">{file.path}</p>
    </div>
    <p className="inbox-detail-verdict" data-verdict={verdict}>
      <strong>Karar: {LABEL[verdict]}.</strong> {agrees && sug ? sug.reason : SAYS[verdict]}
      {sug && !agrees && <span className="inbox-detail-hint">Öneri: {LABEL[sug.verdict]}. {sug.reason}</span>}
    </p>
    {verdict === "move" && <p className="inbox-detail-target">Hedef klasör: <strong>{targetName ?? "seçilmedi"}</strong></p>}
    <dl className="inbox-detail-dates">
      <dt>Geldi</dt><dd>{stamp(file.arrivedMs, true)}</dd>
      <dt>Oluşturma</dt><dd>{stamp(file.createdMs, false)}</dd>
      <dt>Değişiklik</dt><dd>{stamp(file.modifiedMs, true)}</dd>
      <dt>Boyut</dt><dd>{formatBytes(file.size)}</dd>
    </dl>
    {file.duplicateOf && <p className="inbox-detail-exact"><strong>Birebir kopya.</strong> {duplicateNote(file)}</p>}
    {file.similarTo && <p className="inbox-detail-similar"><strong>Adı %{file.similarity} benzer:</strong> {file.similarTo}. İçerik farklı olabilir, iki dosyayı birlikte taşıyın.</p>}
    {image
      ? <img className="inbox-detail-image" src={convertFileSrc(file.thumbnailPath ?? file.path)} alt={`${file.name} önizlemesi`} />
      : file.contentPreview && <div className="inbox-detail-peek"><span>İçerikten ilk satırlar</span>{file.contentPreview}</div>}
    <div className="inbox-detail-actions">
      <button type="button" className="btn" onClick={() => onReveal(file.path)}><FolderOpen size={14} /> Klasörde göster</button>
      <button type="button" className="btn" onClick={() => onOpen(file.path)}><ExternalLink size={14} /> Aç</button>
    </div>
  </aside>;
}

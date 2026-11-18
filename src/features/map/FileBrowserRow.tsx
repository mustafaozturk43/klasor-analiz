import { memo } from "react";
import { Camera, ExternalLink, File, FileArchive, FileText, Film, Image, PenLine } from "lucide-react";
import type { FolderFile } from "../../types/file";
import { formatBytes } from "../../lib/format";
import { extOf, formatDay, kindOf, KIND_LABELS, type FileKind } from "./fileKinds";

const ICONS: Record<FileKind, typeof File> = { image: Image, video: Film, pdf: FileText, doc: FileText, other: File };

/** Windows resim veremediginde tur ikonu + uzanti. */
export function KindIcon({ name, size = 22 }: { name: string; size?: number }) {
  const ext = extOf(name);
  const Icon = ["zip", "rar", "7z"].includes(ext) ? FileArchive : ICONS[kindOf(name)];
  return <span className="browser-icon"><Icon size={size} aria-hidden="true" /><small>{ext.slice(0, 5) || "?"}</small></span>;
}

/** Cekim tarihi (kamera), degisiklik tarihi (kalem) ya da okunuyor. */
function DateCell({ file, taken }: { file: FolderFile; taken: number | null | undefined }) {
  if (typeof taken === "number") {
    return <span className="browser-date taken" title={`Çekim tarihi${file.modifiedAt ? ` (değişiklik: ${formatDay(file.modifiedAt)})` : ""}`}>
      <Camera size={14} aria-hidden="true" />{formatDay(taken)} <small>çekim</small></span>;
  }
  if (!file.modifiedAt) return <span className="browser-date">—</span>;
  return <span className={`browser-date${taken === undefined ? " pending" : ""}`} title={taken === undefined ? "Değiştirme tarihi; çekim tarihi okunuyor" : "Değiştirme tarihi"}>
    <PenLine size={14} aria-hidden="true" />{formatDay(file.modifiedAt)} <small>değişiklik</small></span>;
}

interface Props {
  file: FolderFile;
  source: "A" | "B";
  selected: boolean;
  taken: number | null | undefined;
  thumb: string | null | undefined;
  onToggle: (path: string, shift: boolean) => void;
  onPreview: (path: string) => void;
  onReveal: (path: string) => void;
}

/** Tek dosya satiri: secim, kucuk resim, ad, tarih, boyut. Satirin bos yerine tiklamak secer. */
function FileBrowserRow({ file, source, selected, taken, thumb, onToggle, onPreview, onReveal }: Props) {
  const kind = kindOf(file.name);
  return <div className={`browser-row${selected ? " is-sel" : ""}`} onClick={e => { if (!(e.target as HTMLElement).closest("button, input")) onToggle(file.path, e.shiftKey); }}>
    <input type="checkbox" checked={selected} onChange={() => undefined} onClick={e => onToggle(file.path, e.shiftKey)} aria-label={`${file.name} seç`} />
    <button type="button" className="browser-thumb" onClick={() => onPreview(file.path)} aria-label={`${file.name} ön izlemesini aç`} title="Büyüt">
      {thumb === undefined ? <span className="skel" /> : thumb ? <img src={thumb} alt="" loading="lazy" decoding="async" /> : <KindIcon name={file.name} />}
    </button>
    <div className="browser-name">
      <button type="button" onClick={() => onPreview(file.path)} title={file.path}>{file.name}</button>
      <small>{KIND_LABELS[kind]} · Klasör {source}{thumb === null ? " · ön izleme yok" : ""}</small>
      <small className="browser-narrow">{typeof taken === "number" ? formatDay(taken) : file.modifiedAt ? formatDay(file.modifiedAt) : ""} · {formatBytes(file.size)}</small>
    </div>
    <DateCell file={file} taken={taken} />
    <span className="browser-size">{formatBytes(file.size)}</span>
    <button type="button" className="btn" onClick={() => onReveal(file.path)} aria-label={`${file.name} klasörde göster`} title="Klasörde göster"><ExternalLink size={14} /></button>
  </div>;
}

export default memo(FileBrowserRow);

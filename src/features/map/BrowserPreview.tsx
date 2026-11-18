import { useEffect, type KeyboardEvent } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Check, ChevronLeft, ChevronRight } from "lucide-react";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import type { BrowserEntry } from "./browserView";
import { formatBytes, PREVIEW_IMAGE_EXTS } from "../../lib/format";
import { extOf, formatDay, kindOf, KIND_LABELS } from "./fileKinds";
import { KindIcon } from "./FileBrowserRow";

interface Props {
  items: BrowserEntry[];
  index: number | null;
  onIndex: (i: number | null) => void;
  isSelected: (path: string) => boolean;
  onToggle: (path: string) => void;
  takenOf: (path: string) => number | null | undefined;
  bigThumb: (path: string) => string | null | undefined;
  requestBig: (paths: string[]) => void;
}

/** Buyuk on izleme: ← → ile gezinme, Bosluk ile secme, Esc ile kapatma. */
export default function BrowserPreview({ items, index, onIndex, isSelected, onToggle, takenOf, bigThumb, requestBig }: Props) {
  const entry = index === null ? undefined : items[index];
  const file = entry?.file;
  const isImage = !!file && PREVIEW_IMAGE_EXTS.has(extOf(file.name));
  useEffect(() => { if (file && !isImage) requestBig([file.path]); }, [file, isImage, requestBig]);
  if (!file || index === null) return null;
  const go = (d: number) => { const n = index + d; if (n >= 0 && n < items.length) onIndex(n); };
  const onKey = (e: KeyboardEvent) => {
    if (e.key === "ArrowRight") { e.preventDefault(); go(1); }
    else if (e.key === "ArrowLeft") { e.preventDefault(); go(-1); }
    else if (e.key === " ") { e.preventDefault(); onToggle(file.path); }
  };
  const big = isImage ? convertFileSrc(file.path) : bigThumb(file.path);
  const taken = takenOf(file.path);
  const selected = isSelected(file.path);
  return <Dialog open onOpenChange={o => { if (!o) onIndex(null); }}>
    <DialogContent title={file.name} description={`${index + 1} / ${items.length} · ${KIND_LABELS[kindOf(file.name)]}`} className="max-w-[780px]"
      onKeyDown={onKey} onKeyUp={e => { if (e.key === " ") e.preventDefault(); }}>
      <div className="browser-stage">
        {big === undefined ? <span className="skel" /> : big ? <img src={big} alt={file.name} /> : <KindIcon name={file.name} size={48} />}
      </div>
      <dl className="browser-facts">
        <div><dt>Çekim tarihi</dt><dd>{taken === undefined ? "okunuyor…" : taken === null ? "—" : formatDay(taken)}</dd></div>
        <div><dt>Değiştirme tarihi</dt><dd>{file.modifiedAt ? formatDay(file.modifiedAt) : "—"}</dd></div>
        <div><dt>Boyut</dt><dd>{formatBytes(file.size)}</dd></div>
        <div><dt>Konum</dt><dd className="font-mono break-all">{file.path.replace(/[\\/][^\\/]+$/, "")}</dd></div>
      </dl>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <span className="inline-actions">
          <button type="button" className="btn" disabled={index <= 0} onClick={() => go(-1)}><ChevronLeft size={14} /> Önceki</button>
          <button type="button" className="btn" disabled={index >= items.length - 1} onClick={() => go(1)}>Sonraki <ChevronRight size={14} /></button>
        </span>
        <button type="button" className={`btn${selected ? "" : " primary"}`} onClick={() => onToggle(file.path)} aria-pressed={selected}>
          {selected ? <><Check size={14} /> Seçildi · seçimden çıkar</> : "Seç"} <kbd className="browser-kbd">Boşluk</kbd>
        </button>
      </div>
    </DialogContent>
  </Dialog>;
}

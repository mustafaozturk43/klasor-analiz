import { useEffect, useRef } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ExternalLink, FolderOpen, Play, X } from "lucide-react";
import type { FileItem } from "../../types/file";
import { CATEGORY_LABELS, formatBytes } from "../../lib/format";

interface FilePreviewProps {
  file: FileItem | null;
  onClose: () => void;
  onReveal: (path: string) => void;
  onOpen: (path: string) => void;
}

/** Webview'in gosterebildigi resim uzantilari. */
const IMAGE_EXTS = new Set(["jpg", "jpeg", "png", "webp", "gif", "bmp"]);

/** Dosya onizleme modali: resimlerde gercek onizleme, digerlerinde meta + aksiyonlar. */
export default function FilePreview({
  file,
  onClose,
  onReveal,
  onOpen,
}: FilePreviewProps) {
  const dialog = useRef<HTMLDialogElement>(null);
  // Pencere yalnizca dosya degisince acilir; ust bilesenin her cizimde yeni `onClose` vermesi onu yeniden acmaz.
  useEffect(() => {
    if (!file) return;
    const el = dialog.current;
    if (el && !el.open) el.showModal();
    return () => el?.close();
  }, [file]);

  if (!file) return null;
  const isImage = IMAGE_EXTS.has(file.extension);

  return (
    <dialog
      ref={dialog}
      aria-label="Dosya ön izlemesi"
      className="modal-card preview-card m-auto"
      onCancel={(e) => { e.preventDefault(); onClose(); }}
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div className="relative p-5">
        <button type="button" onClick={onClose} className="btn preview-close" title="Kapat (Esc)" aria-label="Kapat">
          <X size={16} />
        </button>

        <h2 className="truncate pr-12 text-sm font-semibold">{file.name}</h2>
        <p className="preview-path pr-12" title={file.path}>{file.path}</p>

        <div className="preview-stage">
          {isImage ? (
            <img
              src={convertFileSrc(file.path)}
              alt={file.name}
              className="max-h-[55vh] max-w-full rounded-lg object-contain"
            />
          ) : (
            <div className="p-8 text-center">
              <FolderOpen className="mx-auto h-10 w-10" aria-hidden />
              <p className="mt-2 text-xs">
                Bu dosya türü için önizleme yok; varsayılan uygulamayla açabilirsiniz.
              </p>
            </div>
          )}
        </div>

        <div className="preview-meta">
          <span>{CATEGORY_LABELS[file.category]}</span>
          <span className="tabular-nums">{formatBytes(file.size)}</span>
          {file.hashId && (
            <span className="font-mono" title={file.hashId}>
              {file.hashId.length > 16 ? `${file.hashId.slice(0, 16)}…` : file.hashId}
            </span>
          )}
        </div>

        <div className="mt-4 flex flex-wrap gap-2">
          <button type="button" className="btn" onClick={() => onReveal(file.path)}>
            <ExternalLink size={16} />
            Klasörde Göster
          </button>
          <button type="button" className="btn primary" onClick={() => onOpen(file.path)}>
            <Play size={16} />
            Varsayılan Uygulamayla Aç
          </button>
        </div>
      </div>
    </dialog>
  );
}

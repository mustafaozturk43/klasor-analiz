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

const actionBtn =
  "inline-flex items-center gap-1.5 rounded-lg border border-zinc-300 bg-white px-3 py-1.5 text-xs font-medium text-zinc-700 transition-colors hover:bg-zinc-50 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800";

/** Dosya onizleme modali: resimlerde gercek onizleme, digerlerinde meta + aksiyonlar. */
export default function FilePreview({
  file,
  onClose,
  onReveal,
  onOpen,
}: FilePreviewProps) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    if (!file) return;
    dialog.current?.showModal();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", onKey);
    return () => { document.removeEventListener("keydown", onKey); dialog.current?.close(); };
  }, [file, onClose]);

  if (!file) return null;
  const isImage = IMAGE_EXTS.has(file.extension);

  return (
    <dialog
      ref={dialog}
      aria-label="Dosya ön izlemesi"
      className="m-auto w-full max-w-2xl rounded-2xl p-0 text-inherit backdrop:bg-black/50"
      onCancel={onClose}
      onClick={e => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div
        className="relative w-full max-w-2xl rounded-2xl border border-zinc-200 bg-white shadow-2xl dark:border-zinc-700 dark:bg-zinc-900"
        onClick={(e) => e.stopPropagation()}
      >
        <button
          type="button"
          onClick={onClose}
          className="absolute right-3 top-3 rounded-lg p-1 text-zinc-400 transition-colors hover:bg-zinc-100 hover:text-zinc-700 dark:hover:bg-zinc-800"
          title="Kapat (Esc)"
        >
          <X className="h-4 w-4" />
        </button>

        <div className="p-5">
          <h2 className="truncate pr-8 text-sm font-semibold">{file.name}</h2>
          <p className="truncate text-xs text-zinc-500 dark:text-zinc-400" title={file.path}>
            {file.path}
          </p>

          <div className="mt-4 flex min-h-[180px] items-center justify-center rounded-xl bg-zinc-100 dark:bg-zinc-950">
            {isImage ? (
              <img
                src={convertFileSrc(file.path)}
                alt={file.name}
                className="max-h-[55vh] max-w-full rounded-lg object-contain"
              />
            ) : (
              <div className="p-8 text-center">
                <FolderOpen className="mx-auto h-10 w-10 text-zinc-300 dark:text-zinc-700" />
                <p className="mt-2 text-xs text-zinc-500 dark:text-zinc-400">
                  Bu dosya türü için önizleme yok; varsayılan uygulamayla açabilirsiniz.
                </p>
              </div>
            )}
          </div>

          <div className="mt-3 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-zinc-500 dark:text-zinc-400">
            <span>{CATEGORY_LABELS[file.category]}</span>
            <span className="tabular-nums">{formatBytes(file.size)}</span>
            <span className="font-mono" title={file.hashId}>
              {file.hashId.slice(0, 16)}…
            </span>
          </div>

          <div className="mt-4 flex flex-wrap gap-2">
            <button type="button" className={actionBtn} onClick={() => onReveal(file.path)}>
              <ExternalLink className="h-4 w-4" />
              Klasörde Göster
            </button>
            <button
              type="button"
              onClick={() => onOpen(file.path)}
              className="inline-flex items-center gap-1.5 rounded-lg bg-blue-600 px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-blue-500"
            >
              <Play className="h-4 w-4" />
              Varsayılan Uygulamayla Aç
            </button>
          </div>
        </div>
      </div>
    </dialog>
  );
}

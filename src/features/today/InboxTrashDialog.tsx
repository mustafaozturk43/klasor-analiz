import { useMemo } from "react";
import { Trash2 } from "lucide-react";
import { Dialog, DialogClose, DialogContent } from "@/components/ui/dialog";
import { formatBytes, formatCount } from "../../lib/format";
import type { InboxState } from "./useInbox";

const SHOW = 50;

interface Props { inbox: InboxState; open: boolean; onOpenChange: (open: boolean) => void; onConfirm: () => Promise<void> }

/** Geri Donusum Kutusu onayi: hangi dosyalarin gidecegi ve geri almanin nereden yapilacagi. */
export default function InboxTrashDialog({ inbox, open, onOpenChange, onConfirm }: Props) {
  const files = useMemo(() => {
    const bin = new Set(inbox.binPaths);
    return (inbox.listing?.groups ?? []).flatMap(g => g.files).filter(f => bin.has(f.path));
  }, [inbox.listing, inbox.binPaths]);
  const total = files.reduce((s, f) => s + f.size, 0);
  const confirm = async () => {
    await onConfirm();
    onOpenChange(false);
  };
  return <Dialog open={open} onOpenChange={onOpenChange}>
    <DialogContent title={`${formatCount(files.length)} dosya Geri Dönüşüm Kutusu’na gönderilsin mi?`} description={`Toplam ${formatBytes(total)}. Dosyalar kalıcı olarak silinmez.`}>
      <div className="inbox-preview">
        {files.slice(0, SHOW).map(f => <div key={f.path} className="!grid-cols-[minmax(0,1fr)_auto]">
          <span className="truncate">{f.name}</span><span className="text-[12px] tabular-nums text-[var(--color-muted)]">{formatBytes(f.size)}</span>
        </div>)}
        {files.length > SHOW && <div><span className="text-[var(--color-muted)]">ve {formatCount(files.length - SHOW)} dosya daha</span></div>}
      </div>
      <p className="inbox-note"><b>Bu işlem İşlem Geçmişi’nden geri alınamaz.</b> Dosyaları geri almak için Windows Geri Dönüşüm Kutusu’nu açıp “Geri yükle”yi kullanın. Taşıma kararları da aynı anda uygulanır.</p>
      <div className="flex flex-wrap justify-end gap-2">
        <DialogClose className="btn">Vazgeç</DialogClose>
        <button type="button" className="btn inbox-danger" disabled={!files.length || inbox.busy} onClick={() => void confirm()}><Trash2 size={14} /> Uygula ve kutuya gönder</button>
      </div>
    </DialogContent>
  </Dialog>;
}

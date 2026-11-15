import { ArrowRight, FolderInput } from "lucide-react";
import { Dialog, DialogClose, DialogContent } from "@/components/ui/dialog";
import { baseName, formatCount } from "../lib/format";
import { leafName } from "../lib/groupPath";
import type { SelectionMovePlan } from "../lib/selectionCommands";

interface Props { plan: SelectionMovePlan | null; busy: boolean; onCancel: () => void; onConfirm: (plan: SelectionMovePlan) => void }

/** Tasima onayi: hedef yol, olusturulacak klasorler ve numaralanacak dosyalar. */
export default function SelectionMoveDialog({ plan, busy, onCancel, onConfirm }: Props) {
  const n = plan?.entries.length ?? 0;
  return <Dialog open={!!plan} onOpenChange={o => { if (!o) onCancel(); }}>
    {plan && <DialogContent title={`${formatCount(n)} dosya taşınsın mı?`} description="İşlem Geçmiş'e kaydedilir; “Son işlemi geri al” ile tek tıkla geri dönebilirsiniz." className="max-w-[640px]">
      <div className="selection-target-box">
        <span className="font-mono break-all">{plan.targetDir}\</span>
        <span className="text-[13px]">{plan.createdDirs.length
          ? <><span className="badge ok">Yeni</span> {plan.createdDirs.map(d => `“${leafName(d)}”`).join(" ve ")} klasörü oluşturulur.</>
          : <><span className="badge warn">Var olan klasör</span> Dosyalar mevcut içeriğin yanına eklenir.</>}
          {plan.renamed > 0 && ` ${formatCount(plan.renamed)} dosyanın adı hedefte zaten var; üzerine yazılmaz, numara eklenir.`}</span>
      </div>
      <div className="inbox-preview" aria-label="Taşınacak dosyalar">
        {plan.entries.map(e => <div key={e.from}>
          <span className="truncate" title={e.from}>{baseName(e.from)}</span>
          <ArrowRight size={13} className="text-[var(--color-muted)]" />
          <span className={`truncate ${baseName(e.to) !== baseName(e.from) ? "text-[var(--color-warn-ink)] font-medium" : ""}`}>{baseName(e.to)}</span>
        </div>)}
      </div>
      <div className="flex flex-wrap justify-end gap-2">
        <DialogClose className="btn">Vazgeç</DialogClose>
        <button type="button" className="btn primary" disabled={busy} onClick={() => onConfirm(plan)}><FolderInput size={14} /> {formatCount(n)} dosyayı taşı</button>
      </div>
    </DialogContent>}
  </Dialog>;
}

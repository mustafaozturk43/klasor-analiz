import type { InboxState } from "./useInbox";
import TemplateRenameDialog from "../rename/TemplateRenameDialog";

const TOKENS = ["{OzelMetin}", "{Grup}", "{tarih}", "{sıra}", "{ad}", "{yıl}", "{ay}", "{gün}"];

interface Props { inbox: InboxState; open: boolean; onOpenChange: (open: boolean) => void }

/** "Tasi" kararli dosyalari sablonla adlandirma (ortak `TemplateRenameDialog`). */
export default function InboxRenameDialog({ inbox, open, onOpenChange }: Props) {
  return <TemplateRenameDialog open={open} onOpenChange={onOpenChange} count={inbox.movePaths.length} busy={inbox.busy}
    tokens={TOKENS} defaultTemplate="{OzelMetin}_{Grup}_{tarih}_{sıra}"
    description="Uzantılar korunur, dosyalar bulundukları klasörde kalır. İşlem Geçmişi’nden geri alınabilir."
    preview={inbox.previewRename} apply={async plan => { await inbox.applyRename(plan); }} />;
}

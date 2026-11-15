import { CheckCircle2, FolderOpen, Undo2 } from "lucide-react";
import type { OrganizeOutcome } from "../types/organize";
import { baseName, formatBytes, formatCount } from "../lib/format";

/** Uygulama sonrasi ozet: tasinan, karantinaya alinan ve tasinamayan dosyalar. */
export default function OrganizeResult({ outcome, onOpen, onUndo, disabled }: {
  outcome: OrganizeOutcome; onOpen: () => void; onUndo: () => void; disabled: boolean;
}) {
  return <section className="panel">
    <div className="panel-head">
      <div><h2><CheckCircle2 size={18} /> İşlem özeti</h2><p>Tüm taşımalar İşlem Geçmişi’nde tek kayıt olarak tutulur.</p></div>
      <div className="inline-actions">
        <button className="btn" onClick={onOpen}><FolderOpen size={14} /> Klasörü aç</button>
        <button className="btn primary" onClick={onUndo} disabled={disabled}><Undo2 size={14} /> Tümünü geri al</button>
      </div>
    </div>
    <div className="organize-row"><div>Tür klasörlerine taşınan</div><div className="target">{formatCount(outcome.moved)} dosya · {formatBytes(outcome.movedBytes)}</div><span className="badge ok">Tamam</span></div>
    <div className="organize-row"><div>Karantinaya alınan kopya</div><div className="target">{formatCount(outcome.quarantined)} dosya · {formatBytes(outcome.quarantinedBytes)} yer açılabilir</div><span className="badge ok">Tamam</span></div>
    {outcome.removedDirs > 0 && <div className="organize-row"><div>Boşalan ve kaldırılan klasör</div><div className="target">{formatCount(outcome.removedDirs)} klasör · geri almada yeniden oluşur</div><span className="badge ok">Tamam</span></div>}
    {outcome.failed.length > 0 && <>
      <div className="organize-row"><div>Taşınamayan (açık veya izinsiz)</div><div className="target">{formatCount(outcome.failed.length)} dosya yerinde bırakıldı</div><span className="badge warn">Yeniden önizleyin</span></div>
      {outcome.failed.slice(0, 20).map(f => <div className="organize-row" key={f.path}><div className="min-w-0"><span className="name">{baseName(f.path)}</span><small title={f.path}>{f.path}</small></div><div className="target">{f.error}</div><span /></div>)}
    </>}
  </section>;
}

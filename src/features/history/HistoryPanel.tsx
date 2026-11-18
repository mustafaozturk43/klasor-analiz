import { Clock, Download, History, Trash2, Undo2 } from "lucide-react";
import type { UndoRecord } from "../../types/file";
import { formatCount } from "../../lib/format";

interface HistoryPanelProps {
  history: UndoRecord[];
  onUndo: (id: string) => void;
  onClear: () => void;
  onExport: () => void;
}

function formatTime(ms: number): string {
  return new Date(ms).toLocaleString("tr-TR", { day: "2-digit", month: "2-digit", year: "numeric", hour: "2-digit", minute: "2-digit" });
}

/** Sistem islemleri adiyla, etiket tasimalari “etiket” olarak gosterilir. */
const SYSTEM_TAGS = /^(Türüne göre düzenleme|Grup taşıma|Tarihle isimlendirme|Şablonla adlandırma|Otomatik yerleştirme|Karantinadan geri yükleme|Karantinaya alma|Gelen kutusundan taşıma|Gelen kutusu adlandırma)/;
const titleOf = (r: UndoRecord) => (SYSTEM_TAGS.test(r.tag) ? r.tag : `“${r.tag}” etiketine taşıma`);

/**
 * Islem gecmisi (en yeni ustte, son 200). Her islem ayri ayri geri alinabilir;
 * daha yeni bir islem ayni dosyalara dokunduysa backend once onun geri alinmasini ister.
 */
export default function HistoryPanel({ history, onUndo, onClear, onExport }: HistoryPanelProps) {
  if (history.length === 0) {
    return <div className="empty-state"><History size={28} className="mx-auto mb-3" /><p>İşlem geçmişi boş.</p><p className="hint">Taşıma, düzenleme ve karantina işlemleri burada listelenir.</p></div>;
  }
  return <div className="flex flex-col gap-3">
    <div className="inline-actions">
      <p className="hint mr-auto">Son 200 işlem tutulur. İstediğiniz işlemi geri alabilirsiniz; daha yeni bir işlem aynı dosyalara dokunduysa önce onu geri almanız istenir.</p>
      <button type="button" className="btn" onClick={onExport}><Download size={14} /> CSV dışa aktar</button>
      <button type="button" className="btn" onClick={onClear}><Trash2 size={14} /> Geçmişi temizle</button>
    </div>
    <table className="data-table">
      <thead><tr><th scope="col">İşlem</th><th scope="col">Zaman</th><th scope="col" className="num">Dosya</th><th scope="col"><span className="visually-hidden">Geri al</span></th></tr></thead>
      <tbody>
        {history.map((record, index) => <tr key={record.id}>
          <td className="data-table-name"><Clock size={16} aria-hidden="true" /><span>{titleOf(record)}{index === 0 && <small>En son işlem</small>}</span></td>
          <td className="num">{formatTime(record.timestamp)}</td>
          <td className="num">{formatCount(record.moves.length)}</td>
          <td className="data-table-action"><button type="button" className={`btn ${index === 0 ? "primary" : ""}`} onClick={() => onUndo(record.id)}><Undo2 size={14} /> Geri al</button></td>
        </tr>)}
      </tbody>
    </table>
  </div>;
}

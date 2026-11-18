import { ShieldCheck } from "lucide-react";
import type { OrganizerState } from "../../types/app";
import Switch from "../../components/Switch";

/** Secim ozeti ve saklama politikasi: eski sabit alt cubugun sayfa icindeki karsiligi. */
export default function SelectionSummary({ app }: { app: OrganizerState }) {
  return <div className="dup-summary">
    <div><strong>{app.selectedGroups.length} grup seçildi</strong><small>{app.selectedFileCount} dosya · Taşımalar grup kurallarına göre uygulanır</small></div>
    <details><summary>Saklama politikası ve doğrulama</summary><div className="inline-actions mt-2 text-xs">
      <select aria-label="Tutulacak ana dosya" value={app.keepPolicy} onChange={e => app.setKeepPolicy(e.target.value as OrganizerState["keepPolicy"])} disabled={app.isMoving}>
        <option value="first">İlk dosyayı tut</option><option value="newest">En yeniyi tut</option><option value="oldest">En eskiyi tut</option><option value="shortestPath">En kısa yolu tut</option>
      </select>
      <ShieldCheck size={14} /><span>500 MiB+ tam hash doğrulaması</span><Switch label="Büyük dosyalarda tam hash doğrula" checked={app.verifyLarge} onChange={app.setVerifyLarge} />
      <span className="hint">Exact: ana dosya etikete, kopyalar karantinaya. Similar: tüm üyeler etikete.</span>
    </div></details>
  </div>;
}

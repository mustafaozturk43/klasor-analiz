import type { ReactNode } from "react";
import { Check, FolderInput, MoveRight, Sparkles } from "lucide-react";
import type { FileGroup, FileItem, KeepPolicy } from "../../types/file";
import { baseName, formatBytes, formatDate } from "../../lib/format";
import { keptIndex } from "./keepPolicy";

const POLICIES: { value: KeepPolicy; label: string }[] = [
  { value: "first", label: "İlk" }, { value: "newest", label: "En yeni" }, { value: "oldest", label: "En eski" }, { value: "shortestPath", label: "En kısa yol" },
];
const LARGE = 524288000;

interface Props {
  group: FileGroup;
  policy: KeepPolicy;
  setPolicy: (p: KeepPolicy) => void;
  name: string;
  setName: (name: string) => void;
  suggest: () => void;
  rootName: string;
  busy: boolean;
  onPreview: (file: FileItem) => void;
  onMove: () => void;
  /** Kutularla secili grup sayisi; 2 ve ustunde dugme secili gruplarin hepsini tasir. */
  selectedCount: number;
  /** Tasima alaninin hemen ustunde gosterilir (secim ozeti). */
  summary?: ReactNode;
}

/** Yolun klasör kısmını soluk, dosya adını koyu gösterir. */
function PathText({ path }: { path: string }) {
  const name = baseName(path);
  return <p className="dup-path">{path.slice(0, path.length - name.length)}<b>{name}</b></p>;
}

/** Seçili grubun karşılaştırması: ana dosya, kopyalar, doğrulama notu ve "Bu grubu taşı". */
export default function GroupCompare({ group, policy, setPolicy, name, setName, suggest, rootName, busy, onPreview, onMove, selectedCount, summary }: Props) {
  const exact = group.groupType === "Exact";
  const kept = keptIndex(group.files, policy);
  const sampled = exact && group.files.some(f => f.size >= LARGE);
  const tag = name.trim() || "Etiket";
  const many = selectedCount > 1;
  return <section className="dup-compare" aria-label="Karşılaştırma">
    <div className="dup-compare-head">
      <h2>{group.files[0]?.name ?? group.id}</h2>
      {exact && <div className="dup-policy"><span>Hangisi kalsın</span>
        <div role="group" aria-label="Saklama politikası">{POLICIES.map(p => <button key={p.value} type="button" aria-pressed={policy === p.value} onClick={() => setPolicy(p.value)} disabled={busy}>{p.label}</button>)}</div>
      </div>}
    </div>
    <div className="dup-cards">
      {group.files.map((file, i) => {
        const main = exact && i === kept;
        const kind = !exact ? "move" : main ? "keep" : "move";
        return <article key={file.path} className="dup-card" data-kind={kind}>
          <div className="dup-card-head">{main ? <Check size={20} aria-hidden="true" /> : <MoveRight size={20} aria-hidden="true" />}
            {!exact ? "Etikete taşınır" : main ? "Ana dosya, yerinde kalır" : "Kopya, karantinaya gider"}</div>
          <div className="dup-card-body">
            <PathText path={file.path} />
            <dl><dt>Boyut</dt><dd>{formatBytes(file.size)}</dd><dt>Değişiklik</dt><dd>{formatDate(file.mtime)}</dd><dt>Yol uzunluğu</dt><dd>{Array.from(file.path).length} karakter</dd></dl>
            <button type="button" className="btn" onClick={() => onPreview(file)}>Ön izle</button>
          </div>
        </article>;
      })}
    </div>
    <p className="dup-note" data-kind={exact && !sampled ? "ok" : "warn"}>{exact
      ? sampled ? <><strong>Örnek eşleşme.</strong> 500 MB üstündeki dosyalarda yalnızca baştan, ortadan ve sondan örnek alınır; tam doğrulama önerilir (aşağıdaki “Saklama politikası ve doğrulama” bölümünden açılır).</>
        : <><strong>İçerik doğrulandı.</strong> Boyut ve tam BLAKE3 özeti birebir aynı.</>
      : <><strong>Adlar benziyor, içerik farklı.</strong> Bu yüzden hiçbir dosya silinmez ya da karantinaya gitmez; hepsi etikete taşınır.</>}</p>
    {summary}
    <div className="dup-move">
      <div className="dup-move-field"><label htmlFor="dup-tag">Etiket, yani klasör adı</label>
        <div><input id="dup-tag" type="text" value={name} onChange={e => setName(e.target.value)} disabled={busy} />
          <button type="button" className="btn" onClick={suggest} disabled={busy} aria-label="Bu gruba otomatik ad öner"><Sparkles size={14} /></button></div></div>
      <p>{many ? <>Seçili {selectedCount} grubun her biri kendi etiket klasörüne taşınır.</>
        : exact ? <>Ana dosya <code>{rootName}\{tag}\</code> içine, kopya <code>{rootName}\_Kopyalar_Karantina\{tag}\</code> içine taşınır.</> : <>Tüm dosyalar <code>{rootName}\{tag}\</code> içine taşınır.</>} Aynı ad varsa dosya (1) olur, hiçbir şeyin üstüne yazılmaz.</p>
      <button type="button" className="btn primary dup-move-go" onClick={onMove} disabled={busy}><FolderInput size={16} /> {many ? `Seçili ${selectedCount} grubu taşı` : "Bu grubu taşı"}</button>
    </div>
  </section>;
}

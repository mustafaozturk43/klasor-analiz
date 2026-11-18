import type { ReactNode } from "react";
import { FolderInput, PenLine } from "lucide-react";
import type { FolderFile } from "../../types/file";
import { formatBytes, formatCount } from "../../lib/format";
import type { GroupTarget } from "../duplicates/groupPath";
import { KindIcon } from "./FileBrowserRow";

interface Props {
  files: FolderFile[];
  hiddenCount: number;
  thumbOf: (path: string) => string | null | undefined;
  suggestion: string | null;
  target: ReactNode;
  group: string;
  onGroup: (v: string) => void;
  resolved: GroupTarget;
  disabled: boolean;
  onClear: () => void;
  onRename: () => void;
  onMove: () => void;
}

/** Secim cubugu: ozet, mini kucuk resimler, hedef + grup adi, yol on izlemesi ve eylemler. */
export default function SelectionBar({ files, hiddenCount, thumbOf, suggestion, target, group, onGroup, resolved, disabled, onClear, onRename, onMove }: Props) {
  const n = files.length;
  return <div className="selection-bar" role="region" aria-label="Seçilen dosyalar">
    <div className="selection-sum">
      <strong>{formatCount(n)} dosya seçili</strong>
      <span className="hint">{formatBytes(files.reduce((s, f) => s + f.size, 0))}{hiddenCount ? ` · ${formatCount(hiddenCount)} tanesi şu an süzgeç dışında, onlar da taşınır` : ""}</span>
      <div className="selection-minis">
        {files.slice(0, 8).map(f => { const t = thumbOf(f.path); return <span key={f.path} className="selection-mini" title={f.name}>{t ? <img src={t} alt="" /> : <KindIcon name={f.name} size={14} />}</span>; })}
        {n > 8 && <span className="badge">+{n - 8}</span>}
        {suggestion && <button type="button" className="browser-chip" title="Seçilenlerin hepsi aynı aydan" onClick={() => onGroup(suggestion)}>Öneri: {suggestion}</button>}
      </div>
    </div>
    <div className="selection-dest">
      <label className="selection-field" htmlFor="selection-target">Hedef klasör{target}</label>
      <label className="selection-field">Grup adı (alt klasör)
        <input id="selection-group" type="text" value={group} autoComplete="off" placeholder="Örn. Mayıs ya da Faturalar\Mayıs"
          onChange={e => onGroup(e.target.value)} onKeyDown={e => { if (e.key === "Enter" && resolved.ok && !disabled) onMove(); }} /></label>
      <p className={`selection-path${!resolved.ok && resolved.error ? " err" : ""}`} aria-live="polite">
        {resolved.ok ? <>Hedef: <span className="font-mono">{resolved.base}</span>
          {resolved.parts.map((p, i) => <span key={i} className="inline-flex items-center gap-1"><span className="font-mono">\{p.name}</span><span className={`badge ${p.exists ? "" : "ok"}`}>{p.exists ? "var" : "yeni"}</span></span>)}
          {!resolved.parts.length && <span className="badge">doğrudan bu klasöre</span>}</> : resolved.message}
      </p>
    </div>
    <div className="selection-actions">
      <button type="button" className="btn" onClick={onClear}>Seçimi temizle</button>
      <button type="button" className="btn" onClick={onRename} disabled={disabled}><PenLine size={14} /> Adlandır…</button>
      <button type="button" className="btn primary" onClick={onMove} disabled={disabled || !resolved.ok}><FolderInput size={14} /> {resolved.ok ? `${formatCount(n)} dosyayı taşı…` : "Klasöre taşı"}</button>
    </div>
  </div>;
}

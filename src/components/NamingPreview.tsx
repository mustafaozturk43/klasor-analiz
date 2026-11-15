import { File, Folder, PenLine } from "lucide-react";
import { baseName, formatCount } from "../lib/format";
import type { NameEntry } from "../lib/namingCommands";
import type { NamingState } from "../hooks/useNaming";
import VirtualRows from "./VirtualRows";

interface Props { naming: NamingState; rootPath: string; locked: boolean; onReset: () => void }

/** Uzantisiz ad uzunlugu (uzunluk siniri bununla karsilastirilir). */
const stemLength = (name: string, isDir: boolean) => Array.from(isDir ? name : name.replace(/\.[^.]+$/, "")).length;

function status(e: NameEntry): { kind: string; label: string } {
  if (!e.to) return e.warning ? { kind: "bad", label: "Atlanacak" } : { kind: "", label: "Aynı kalır" };
  if (e.warning) return { kind: "warn", label: "Numaralandı" };
  return baseName(e.to).toLocaleLowerCase("tr") === baseName(e.from).toLocaleLowerCase("tr") ? { kind: "ok", label: "Yalnız harf" } : { kind: "ok", label: "Yeni ad" };
}

/** Canli onizleme: klasor agaci uzerinde simdiki ad → yeni ad, uzunluk ve durum; altta uygulama dugmesi. */
export default function NamingPreview({ naming, rootPath, locked, onReset }: Props) {
  const { plan, changing, error, loading } = naming;
  const depthOf = (path: string) => Math.max(0, path.slice(rootPath.length).split(/[\\/]/).filter(Boolean).length - 1);
  const dirs = changing.filter(e => e.isDir).length;
  const numbered = changing.filter(e => e.warning).length;
  const skipped = plan?.entries.filter(e => !e.to && e.warning) ?? [];
  return <section className="panel naming-preview">
    <div className="panel-head"><h2><PenLine size={18} /> Önizleme</h2><span className="hint">Kök klasörün kendi adı değişmez. Uzantılar korunur.</span></div>
    {!rootPath ? <div className="empty-state">Adlandırmak için yukarıdan bir klasör seçin.</div>
      : error ? <div className="error-state" role="alert">{error}</div>
      : !plan ? <div className="empty-state" role="status">Önizleme hazırlanıyor…</div>
      : !plan.entries.length ? <div className="empty-state">Bu klasörde adlandırılabilecek öğe yok.</div>
      : <>
        <div className="naming-row naming-head"><span>Şimdiki ad</span><span>Yeni ad</span><span className="naming-len">Uzunluk</span><span>Durum</span></div>
        <VirtualRows items={plan.entries} estimate={41} maxHeight={560} itemKey={e => e.from} render={e => {
          const st = status(e), name = baseName(e.from), next = e.to ? baseName(e.to) : name, pad = { paddingLeft: depthOf(e.from) * 20 };
          const Icon = e.isDir ? Folder : File;
          return <div className="naming-row" data-same={!e.to || undefined}>
            <div className="naming-name naming-old" style={pad} title={e.from}><Icon size={16} aria-hidden="true" /><span>{name}</span></div>
            <div className="naming-name naming-new" style={pad} title={next}><Icon size={16} aria-hidden="true" /><span>{next}</span></div>
            <span className="naming-len">{stemLength(next, e.isDir)}</span>
            <span><span className={`badge ${st.kind}`} title={e.warning ?? undefined}>{st.label}</span></span>
          </div>;
        }} />
        {skipped.length > 0 && <details className="naming-skipped"><summary>{formatCount(skipped.length)} öğe atlanacak, nedenleri</summary>
          {skipped.slice(0, 50).map(e => <p className="hint break-all" key={e.from}>{baseName(e.from)}: {e.warning}</p>)}</details>}
      </>}
    <div className="naming-foot">
      <span role="status">{!plan ? "" : changing.length
        ? `${formatCount(dirs)} klasör ve ${formatCount(changing.length - dirs)} dosyanın adı değişecek${numbered ? ` · ${formatCount(numbered)} ad dolu olduğu için numaralandı` : ""}.`
        : "Bu kurallarla değişecek ad yok."}{loading && plan ? " Güncelleniyor…" : ""}</span>
      <button type="button" className="btn" onClick={onReset} disabled={locked}>Kuralları sıfırla</button>
      <button type="button" className="btn primary naming-go" onClick={() => void naming.apply()} disabled={locked || loading || !changing.length}>
        {changing.length ? `${formatCount(changing.length)} adı uygula` : "Uygulanacak ad yok"}</button>
    </div>
  </section>;
}

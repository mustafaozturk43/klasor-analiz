import { SlidersHorizontal } from "lucide-react";
import { NAMING_PRESETS, type NameRuleOptions } from "./namingCommands";
import Switch from "../../components/Switch";

interface Props { options: NameRuleOptions; onChange: (next: NameRuleOptions) => void; disabled: boolean }

const SCOPE = [["both", "Klasör ve dosya"], ["dirs", "Yalnız klasörler"], ["files", "Yalnız dosyalar"]] as const;
const CASE = [["keep", "Olduğu gibi"], ["title", "İlk Harfler Büyük"], ["sentence", "Yalnız ilk harf"], ["lower", "küçük"], ["upper", "BÜYÜK"]] as const;
const SPACE = [["keep", "Boşluk kalsın"], ["dash", "Tire (-)"], ["underscore", "Alt çizgi (_)"], ["none", "Bitişik"]] as const;
const DATE = [["none", "Yok"], ["my", "Ay-Yıl (11-2026)"], ["ym", "Yıl-Ay (2026-11)"]] as const;

/** Tek secimli dugme grubu. */
function Seg<T extends string>({ label, value, items, onPick }: { label: string; value: T; items: readonly (readonly [T, string])[]; onPick: (v: T) => void }) {
  return <div className="naming-seg" role="group" aria-label={label}>
    {items.map(([v, text]) => <button key={v} type="button" aria-pressed={value === v} onClick={() => onPick(v)}>{text}</button>)}
  </div>;
}

/** Ad kurallari: hazir kaliplar, kapsam, harf bicimi, bosluk, Turkce harfler, sira no, tarih ve uzunluk. */
export default function NamingOptions({ options: o, onChange, disabled }: Props) {
  const set = <K extends keyof NameRuleOptions>(key: K, value: NameRuleOptions[K]) => onChange({ ...o, [key]: value });
  const num = (key: "start" | "maxLen", max: number) => (e: React.ChangeEvent<HTMLInputElement>) => set(key, Math.max(0, Math.min(max, Math.floor(Number(e.target.value)) || 0)));
  return <section className="panel naming-options">
    <div className="panel-head"><h2><SlidersHorizontal size={18} /> Ad kuralları</h2></div>
    <fieldset disabled={disabled}>
      <div className="naming-opt"><b>Hazır kalıplar</b>
        <div className="inline-actions">{NAMING_PRESETS.map(p => <button key={p.id} type="button" className="btn" title={p.hint} onClick={() => onChange(p.options)}>{p.label}</button>)}</div></div>
      <div className="naming-opt"><b>Kapsam</b>
        <Seg label="Kapsam" value={o.scope} items={SCOPE} onPick={v => set("scope", v)} />
        <label><Switch checked={o.includeSubdirs} onChange={v => set("includeSubdirs", v)} label="Alt klasörlerin içini de adlandır" /> Alt klasörlerin içi de</label></div>
      <div className="naming-opt"><b>Harf biçimi</b><Seg label="Harf biçimi" value={o.case} items={CASE} onPick={v => set("case", v)} /></div>
      <div className="naming-opt"><b>Boşluk yerine</b><Seg label="Boşluk yerine" value={o.space} items={SPACE} onPick={v => set("space", v)} /></div>
      <div className="naming-opt"><b>Türkçe harfler</b>
        <Seg label="Türkçe harfler" value={o.ascii ? "ascii" : "keep"} items={[["keep", "Kullan (ç ğ ı ö ş ü)"], ["ascii", "Kullanma (c g i o s u)"]] as const} onPick={v => set("ascii", v === "ascii")} />
        <label><Switch checked={o.clean} onChange={v => set("clean", v)} label="Parantez ve noktalama işaretlerini kaldır" /> Parantez ve noktalama işaretlerini kaldır</label></div>
      <div className="naming-opt"><b>Başa sıra numarası</b>
        <label><Switch checked={o.seqDirs} onChange={v => set("seqDirs", v)} label="Klasörlere sıra numarası ver" /> Klasörlere</label>
        <label><Switch checked={o.seqFiles} onChange={v => set("seqFiles", v)} label="Dosyalara sıra numarası ver" /> Dosyalara</label>
        <div className="inline-actions">
          <label>Başlangıç <input type="number" min={0} max={9999} value={o.start} onChange={num("start", 9999)} aria-label="Sıra başlangıcı" /></label>
          <label>Basamak <select value={o.pad} onChange={e => set("pad", Number(e.target.value))} aria-label="Sıra basamak sayısı">
            {[1, 2, 3, 4].map(n => <option key={n} value={n}>{"0".repeat(n - 1)}1</option>)}</select></label>
        </div>
        <span className="hint">Her klasörün içi ada göre ayrı numaralanır. Addaki eski numara tekrar eklenmez.</span></div>
      <div className="naming-opt"><b>Sona tarih</b><Seg label="Sona tarih" value={o.date} items={DATE} onPick={v => set("date", v)} />
        <span className="hint">Tarih, öğenin son değişiklik tarihidir.</span></div>
      <div className="naming-opt"><b>Uzunluk sınırı</b>
        <label>En fazla <input type="number" min={0} max={255} value={o.maxLen} onChange={num("maxLen", 255)} aria-label="En fazla karakter" /> karakter</label>
        <span className="hint">0 = sınır yok. Uzantı sayılmaz; sıra numarası ve tarih korunur, ad gövdesi kısalır.</span></div>
    </fieldset>
  </section>;
}

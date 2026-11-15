import type { AppSettings } from "../types/settings";
import { todayInboxPath } from "../lib/settingsCommands";
import Switch from "./Switch";

interface Props {
  draft: AppSettings;
  update: (patch: Partial<AppSettings>) => void;
  pickFolder: (title: string) => Promise<string | null>;
  onReveal: (path: string) => void;
}

/** Otomasyon kartı: aç-kapa, izlenen kaynaklar, yedek klasörü, bugünün klasörü ve iki davranış anahtarı. */
export default function AutomationCard({ draft: d, update, pickFolder, onReveal }: Props) {
  const missing = !d.sourceFolders.length || !d.backupRoot;
  const addSource = async () => {
    const f = await pickFolder("İzlenecek kaynak klasörü seçin");
    if (f && !d.sourceFolders.some(x => x.toLowerCase() === f.toLowerCase())) update({ sourceFolders: [...d.sourceFolders, f] });
  };
  const pickBackup = async () => {
    const f = await pickFolder("Yedek (hedef) klasörünü seçin");
    if (f) update({ backupRoot: f });
  };
  const today = d.backupRoot ? todayInboxPath(d.backupRoot) : "";
  return <section className="rules-card" aria-label="Otomasyon">
    <div className="rules-card-head">
      <div><h2>Otomasyon</h2><p>Pencereyi kapatsan da tepside çalışır. Tamamen kapatmak için tepsi menüsündeki “Çıkış”ı kullan.</p></div>
      <Switch checked={d.automationEnabled} onChange={v => update({ automationEnabled: v })} label="Otomasyon" />
    </div>
    <dl className="rules-paths">
      <dt>İzlenen klasör</dt>
      <dd>{d.sourceFolders.length ? d.sourceFolders.map(f => <span key={f} className="rules-path">{f}
        <button type="button" className="btn" onClick={() => update({ sourceFolders: d.sourceFolders.filter(x => x !== f) })} aria-label={`${f} kaynağını kaldır`}>Kaldır</button></span>)
        : <span className="hint">Henüz kaynak klasör yok.</span>}</dd>
      <dd className="rules-path-act"><button type="button" className="btn" onClick={() => void addSource()}>{d.sourceFolders.length ? "Ekle" : "Klasör seç"}</button></dd>
      <dt>Yedek klasörü</dt>
      <dd><span className="rules-path">{d.backupRoot || "Henüz seçilmedi."}</span></dd>
      <dd className="rules-path-act"><button type="button" className="btn" onClick={() => void pickBackup()}>{d.backupRoot ? "Değiştir" : "Klasör seç"}</button></dd>
      <dt>Bugünün klasörü</dt>
      <dd><span className="rules-path">{today || "Yedek klasörü seçilince görünür."}</span></dd>
      <dd className="rules-path-act"><button type="button" className="btn" disabled={!today} onClick={() => onReveal(today)}>Aç</button></dd>
    </dl>
    <div className="rules-toggles">
      <label><span>Hiçbir gruba uymayanlar “Diğer” klasörüne gitsin<small>Kapalıyken hiçbir gruba uymayan dosyalar kaynak klasörde kalır.</small></span>
        <Switch checked={d.includeOther} onChange={v => update({ includeOther: v })} label="Diğer grubu" /></label>
      <label><span>Uygulama açılınca bekleyen dosyaları da yerleştir<small>Kapalıyken yalnızca otomasyon açıldıktan sonra gelen dosyalar taşınır.</small></span>
        <Switch checked={d.scanPendingOnStart} onChange={v => update({ scanPendingOnStart: v })} label="Açılışta bekleyenleri yerleştir" /></label>
      <label><span>Windows açılınca başlat<small>Otomasyon açıksa pencere göstermeden tepside başlar; kapalıysa pencere açılır.</small></span>
        <Switch checked={d.startWithWindows} onChange={v => update({ startWithWindows: v })} label="Windows ile başlat" /></label>
      <label><span>Otomatik taşımaları bildir<small>Pencere önde değilken bir dosya gelen kutusuna taşınınca Windows bildirimi gösterir.</small></span>
        <Switch checked={d.notifyMoves} onChange={v => update({ notifyMoves: v })} label="Taşıma bildirimi" /></label>
    </div>
    {d.automationEnabled && missing && <p className="settings-error">Otomasyonu açmak için en az bir kaynak klasör ve bir yedek klasörü seçin.</p>}
  </section>;
}

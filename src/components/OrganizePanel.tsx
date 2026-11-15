import { Play, Square } from "lucide-react";
import type { OrganizerState } from "../types/app";
import type { OrganizeToggles } from "../types/organize";
import type { OrganizeSettings } from "../hooks/useOrganizeSettings";
import type { OrganizerHook } from "../hooks/useOrganizer";
import { formatCount } from "../lib/format";
import OrganizeProfiles from "./OrganizeProfiles";
import Switch from "./Switch";

const OPTIONS: { key: keyof OrganizeToggles; label: string; hint: string }[] = [
  { key: "quarantineDuplicates", label: "Birebir kopyaları karantinaya al", hint: "İçeriği aynı olanlardan biri kalır" },
  { key: "protectProjects", label: "Proje klasörlerini koru", hint: ".git, package.json, .sln içeren klasörler" },
  { key: "flat", label: "Tek kademeli klasör", hint: "Dokümanlar/PDF yerine doğrudan PDF" },
  { key: "byYear", label: "Yıla göre alt klasör", hint: "Değişiklik yılına göre: PDF/2026" },
  { key: "removeEmptyDirs", label: "Boşalan klasörleri kaldır", hint: "Yalnızca tamamen boş kalanlar; geri almada yeniden oluşur" },
];

interface Props { app: OrganizerState; settings: OrganizeSettings; org: OrganizerHook; locked: boolean }

/** Kurallar ekranının sağ paneli: "Türüne göre düzenle" seçenekleri, profiller ve "Tara ve önizle". */
export default function OrganizePanel({ app, settings, org, locked }: Props) {
  const p = org.progress;
  const pct = p && p.totalFiles ? Math.round((p.filesDone / p.totalFiles) * 100) : null;
  const off = !!org.busy || locked;
  return <aside className="rules-organize" aria-label="Türüne göre düzenle">
    <div><h2>Türüne göre düzenle</h2>
      <p>Seçtiğin klasörü Dokümanlar/PDF, Resimler/JPG gibi klasörlere toplar. Önizleme onaylanmadan hiçbir dosya taşınmaz.</p></div>
    <fieldset disabled={off}>
      <ul>{OPTIONS.map(o => <li key={o.key}><span>{o.label}<small>{o.hint}</small></span>
        <Switch checked={settings.toggles[o.key]} onChange={v => settings.setToggle(o.key, v)} label={o.label} /></li>)}</ul>
    </fieldset>
    <div className="rules-organize-profiles"><OrganizeProfiles settings={settings} disabled={off} /></div>
    {org.busy === "preview"
      ? <button type="button" className="btn primary" onClick={org.stop}><Square size={14} /> Durdur</button>
      : <button type="button" className="btn primary" onClick={() => void org.preview()} disabled={!app.rootPath || off || !!settings.rulesError}><Play size={14} /> Tara ve önizle</button>}
    {org.busy === "preview" && <div role="status" className="rules-organize-progress">
      <span>{p?.phase === "hashing" ? "Kopyalar karşılaştırılıyor" : "Dosyalar taranıyor"}{p ? `, ${formatCount(p.filesDone)}${p.totalFiles ? ` / ${formatCount(p.totalFiles)}` : ""}` : ""}</span>
      {pct !== null && <div className="organize-progress"><span style={{ transform: `scaleX(${pct / 100})` }} /></div>}
    </div>}
    {!app.rootPath && <p className="rules-organize-note">Önce üstten bir ana klasör seçin.</p>}
    <p className="rules-organize-note">Kapsam üstteki ayarlardan gelir: alt klasörler {app.includeSubdirs ? "dahil" : "hariç"}, diğer dosyalar {app.includeOther ? "Diğer/ altına taşınır" : "yerinde kalır"}.
      {" "}Bu kuralların uzantı grupları soldakilerden bağımsızdır. Uygulanan her düzenleme İşlem Geçmişi’nden geri alınır.</p>
  </aside>;
}

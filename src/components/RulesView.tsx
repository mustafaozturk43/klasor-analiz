import type { OrganizerState } from "../types/app";
import { useOrganizer } from "../hooks/useOrganizer";
import { useOrganizeSettings } from "../hooks/useOrganizeSettings";
import OrganizeOutput from "./OrganizeOutput";
import OrganizePanel from "./OrganizePanel";
import SettingsView from "./SettingsView";

/** Kurallar: solda otomasyon ve uzantı grupları, sağda "Türüne göre düzenle", altta önizleme ve sonuç. */
export default function RulesView({ app }: { app: OrganizerState }) {
  const settings = useOrganizeSettings();
  const org = useOrganizer(app, settings);
  const locked = app.isScanning || (app.isMoving && !org.busy);
  return <div className="rules-screen">
    <div className="rules-layout">
      <SettingsView onReveal={p => void app.reveal(p)} />
      <OrganizePanel app={app} settings={settings} org={org} locked={locked} />
    </div>
    <OrganizeOutput app={app} settings={settings} org={org} locked={locked} />
  </div>;
}

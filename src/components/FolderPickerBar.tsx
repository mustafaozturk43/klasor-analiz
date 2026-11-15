import type { ReactNode } from "react";
import { FolderOpen, Loader2, ScanSearch, Square } from "lucide-react";
import type { OrganizerState } from "../types/app";
import RecentFolders from "./RecentFolders";
import Switch from "./Switch";

interface Props {
  app: OrganizerState;
  /** Tarama ve karsilastirma denetimleri; Adlandirma ekraninda gizlenir (tarama gerektirmez). */
  scanControls: boolean;
  /** Panelin alt satiri (tarama kapsami). */
  children?: ReactNode;
}

/** Sayfa basindaki klasor secimi: yol, Klasör A/B, son klasorler, karsilastirma anahtari ve tarama dugmesi. */
export default function FolderPickerBar({ app, scanControls, children }: Props) {
  const busy = app.isScanning || app.isMoving;
  return <section className="panel" aria-label="Klasör seçimi">
    <div className="picker-row">
      <div className="picker-paths"><small>Seçili klasör</small>
        <b title={app.rootPath}>{app.rootPath || "Başlamak için bir klasör seçin"}</b>
        {scanControls && app.compareMode && app.rootB && <b title={app.rootB}>{app.rootB}</b>}
      </div>
      <fieldset className="header-folder-controls" disabled={busy} aria-label="Klasör seçimi ve karşılaştırma">
        <button className="btn" onClick={app.pickFolder}><FolderOpen size={16} /> Klasör A</button>
        <RecentFolders currentPath={app.rootPath} onPick={path => { if (!busy) void app.openRecent(path); }} />
        {scanControls && <label className="header-compare-toggle"><Switch checked={app.compareMode} onChange={app.setCompareMode} label="İki klasör karşılaştır" /> İki klasör karşılaştır</label>}
        {scanControls && app.compareMode && <button className="btn" onClick={app.pickFolderB} title={app.rootB || "Klasör B seçilmedi"}><FolderOpen size={16} /> {app.rootB ? "Klasör B değiştir" : "Klasör B seç"}</button>}
      </fieldset>
      {scanControls && <button className="btn primary" onClick={app.scan} disabled={!app.rootPath || (app.compareMode && !app.rootB) || busy}>
        {app.isScanning ? <Loader2 className="animate-spin" size={16} /> : <ScanSearch size={16} />}
        {app.isScanning ? "Taranıyor…" : app.compareMode ? "Karşılaştır" : "Taramayı başlat"}
      </button>}
      {scanControls && app.isScanning && <button className="btn" onClick={app.stopScan}><Square size={14} /> Durdur</button>}
    </div>
    {children}
  </section>;
}

import { useState } from "react";
import { ChevronDown, FolderSearch, SlidersHorizontal } from "lucide-react";
import Switch from "./Switch";
import type { OrganizerState } from "../types/app";

/** Tarama kapsami. `embedded`: klasor secimi panelinin alt satiri olarak cizilir (kendi cercevesi olmaz). */
export default function WorkspaceSettings({ app, embedded }: { app: OrganizerState; embedded?: boolean }) {
  const [advanced, setAdvanced] = useState(false);
  return <section className={embedded ? "scan-scope-embedded" : "panel"} aria-label="Tarama ayarları">
    <fieldset disabled={app.isScanning || app.isMoving}>
      <div className="scan-scope-row">
        <h2 className="scan-scope-title"><FolderSearch size={15} /> Tarama kapsamı</h2>
        <div className="scan-scope-controls">
          <label><Switch checked={app.includeSubdirs} onChange={app.setIncludeSubdirs} label="Alt klasörleri tara" /> Alt klasörler</label>
          <label><Switch checked={app.includeOther} onChange={app.setIncludeOther} label="Diğer dosyaları da tara" /> Diğer dosyalar</label>
        </div>
        <button type="button" className="btn scan-advanced-toggle" aria-expanded={advanced} aria-controls="advanced-scan-settings" onClick={() => setAdvanced(value => !value)}>
          <SlidersHorizontal size={14} /> Gelişmiş tarama ayarları <ChevronDown size={14} className={advanced ? "rotate-180" : ""} /></button>
      </div>
      <div id="advanced-scan-settings" className="scan-advanced-content" hidden={!advanced}>
        <div className="inline-actions">
          <label>Benzerlik eşiği %{app.thresholdPercent} <input aria-label="Benzerlik eşiği" type="range" min="70" max="95" value={app.thresholdPercent} onChange={e => app.setThresholdPercent(Number(e.target.value))} /></label>
          <input aria-label="Hariç tutulan klasörler" type="text" placeholder="Hariç klasörler, virgülle" value={app.excludeText} onChange={e => app.setExcludeText(e.target.value)} />
          <span className="hint">Yasaklı uzantılar ve karantina taranmaz.</span>
        </div>
      </div>
    </fieldset>
  </section>;
}

import { useState } from "react";
import { CalendarDays } from "lucide-react";
import { useDateGroup } from "../hooks/useDateGroup";
import { DATE_INDEX_FILE, LAYOUTS } from "../lib/dateGroupCommands";
import { splitExtensions } from "../lib/settingsCommands";
import type { OrganizerState } from "../types/app";
import DateGroupPreview from "./DateGroupPreview";
import Switch from "./Switch";

const same = (a: string[], b: string[]) => a.length === b.length && a.every(x => b.includes(x));

/** Tarihe Gore: secili klasordeki dosyalari cekim (yoksa olusturma) tarihine gore Yıl\Ay klasorlerine toplar. */
export default function DateGroupView({ app, active }: { app: OrganizerState; active: boolean }) {
  const dg = useDateGroup(app, active);
  const locked = app.isScanning || app.isMoving || dg.busy;
  const o = dg.options;
  const [custom, setCustom] = useState("");
  const chip = !o.extensions.length || dg.groups.some(g => same(g.extensions, o.extensions));
  const pick = (extensions: string[]) => { setCustom(""); dg.setOptions({ ...o, extensions }); };
  return <div className="naming-layout">
    <section className="panel naming-options">
      <div className="panel-head"><h2><CalendarDays size={18} /> Yıl\Ay gruplama</h2></div>
      <fieldset disabled={locked}>
        <div className="naming-opt"><b>Hangi dosyalar</b>
          <div className="naming-seg" role="group" aria-label="Dosya türü">
            <button type="button" aria-pressed={!o.extensions.length} onClick={() => pick([])}>Tümü</button>
            {dg.groups.map(g => <button key={g.name} type="button" aria-pressed={!!o.extensions.length && same(g.extensions, o.extensions)} title={g.extensions.join(", ")}
              onClick={() => pick(g.extensions)}>{g.name}</button>)}
          </div>
          <label className="w-full">Özel uzantılar
            <input type="text" className="date-group-exts" value={custom || (chip ? "" : o.extensions.join(", "))} placeholder="Örn. jpg, png, heic" autoComplete="off"
              onChange={e => { setCustom(e.target.value); dg.setOptions({ ...o, extensions: splitExtensions(e.target.value) }); }} /></label></div>
        <div className="naming-opt"><b>Kapsam</b>
          <label><Switch checked={o.recursive} onChange={v => dg.setOptions({ ...o, recursive: v })} label="Alt klasörlerdeki dosyaları da topla" /> Alt klasörlerdeki dosyalar da</label>
          <p className="hint">Kapalıyken yalnızca seçili klasörün kendi dosyaları taşınır.</p></div>
        <div className="naming-opt"><b>Klasör düzeni</b>
          <div className="naming-seg" role="group" aria-label="Klasör düzeni">
            {LAYOUTS.map(l => <button key={l.id} type="button" aria-pressed={o.layout === l.id} title={l.example} onClick={() => dg.setOptions({ ...o, layout: l.id })}>{l.label}</button>)}
          </div>
          <p className="hint">{LAYOUTS.find(l => l.id === o.layout)?.example}</p>
          <p className="hint">Gruplama öncesi kök klasöre {DATE_INDEX_FILE} yazılır: klasör, dosya adı ve dosyaya link; tarayıcıda açıp arayabilirsin.</p></div>
        <div className="naming-opt"><b>Tarih</b>
          <p className="hint">Fotoğraf ve videolarda çekim tarihi, diğerlerinde oluşturma tarihi kullanılır. Tarihi okunamayanlar “Tarihsiz” klasörüne gider.</p></div>
      </fieldset>
    </section>
    <DateGroupPreview dg={dg} rootPath={app.rootPath} locked={locked} />
  </div>;
}

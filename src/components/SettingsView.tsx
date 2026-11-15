import { useMemo } from "react";
import { Plus, RotateCcw, Save } from "lucide-react";
import { useSettings } from "../hooks/useSettings";
import { extensionConflicts } from "../lib/settingsCommands";
import AutomationCard from "./AutomationCard";
import ExtGroupCard from "./ExtGroupCard";

/** Kurallar ekraninin sol yarisi: otomasyon (kaynaklar, yedek klasoru, anahtarlar) ve uzanti gruplari. */
export default function SettingsView({ onReveal }: { onReveal: (path: string) => void }) {
  const s = useSettings();
  const conflicts = useMemo(() => extensionConflicts(s.draft?.extGroups ?? []), [s.draft?.extGroups]);
  if (!s.draft) return <section className="rules-card">{s.error ? <p className="error-state" role="alert">{s.error}</p> : <div className="empty-state">Ayarlar yükleniyor…</div>}</section>;
  const d = s.draft;
  return <div className="rules-stack">
    <AutomationCard draft={d} update={s.update} pickFolder={s.pickFolder} onReveal={onReveal} />

    <section className="rules-groups" aria-label="Uzantı grupları">
      <div className="rules-groups-head">
        <h2>Uzantı grupları</h2>
        <div className="inline-actions">
          <button type="button" className="btn" onClick={() => void s.resetGroups()}><RotateCcw size={14} /> Varsayılanlar</button>
          <button type="button" className="btn" onClick={s.addGroup}><Plus size={14} /> Grup ekle</button>
        </div>
      </div>
      {d.extGroups.map((g, i) => <ExtGroupCard key={i} group={g} conflicts={conflicts}
        onChange={next => s.setGroup(i, next)} onRemove={() => s.setGroup(i, null)} canRemove={d.extGroups.length > 1} />)}
      <p className="hint">Bir uzantı yalnızca bir gruba bağlanabilir. Aynı uzantıyı ikinci gruba yazarsan çakışan etiket kırmızıya döner ve kayıt durur.</p>
    </section>

    <div className="settings-footer" role="status">
      {s.error && <p className="settings-error mr-auto" role="alert">{s.error}</p>}
      {!s.error && s.notice && <p className="hint mr-auto">{s.notice}</p>}
      {!s.error && !s.notice && <p className="hint mr-auto">{s.dirty ? "Kaydedilmemiş değişiklikler var." : "Tüm değişiklikler kaydedildi."}</p>}
      <button type="button" className="btn" onClick={s.discard} disabled={!s.dirty || s.saving}>Vazgeç</button>
      <button type="button" className="btn primary" onClick={() => void s.save()} disabled={!s.dirty || s.saving || conflicts.size > 0}><Save size={14} /> {s.saving ? "Kaydediliyor…" : "Kaydet"}</button>
    </div>
  </div>;
}

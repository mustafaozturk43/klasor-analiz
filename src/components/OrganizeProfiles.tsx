import { useState } from "react";
import { Bookmark, Save, Trash2 } from "lucide-react";
import type { OrganizeSettings } from "../hooks/useOrganizeSettings";

/** Kayitli duzen profilleri: kurallar + secenekler tek adla saklanir ve geri yuklenir. */
export default function OrganizeProfiles({ settings, disabled }: { settings: OrganizeSettings; disabled: boolean }) {
  const { profiles, activeProfile, saveProfile, applyProfile, deleteProfile } = settings;
  const [name, setName] = useState("");
  const exists = profiles.some(p => p.name.toLocaleLowerCase("tr") === name.trim().toLocaleLowerCase("tr"));
  const save = () => { saveProfile(name); setName(""); };

  return <fieldset className="inline-actions" disabled={disabled}>
    <Bookmark size={15} className="text-[var(--color-muted)]" aria-hidden="true" />
    <select aria-label="Kayıtlı düzen profili" value={activeProfile ?? ""} onChange={e => e.target.value && applyProfile(e.target.value)}>
      <option value="">{profiles.length ? "Profil seçin…" : "Kayıtlı profil yok"}</option>
      {profiles.map(p => <option key={p.name} value={p.name}>{p.name}</option>)}
    </select>
    {activeProfile && <button type="button" className="btn" onClick={() => deleteProfile(activeProfile)} aria-label={`${activeProfile} profilini sil`}><Trash2 size={14} /></button>}
    <input type="text" aria-label="Yeni profil adı" placeholder="Profil adı, örn. Muhasebe" value={name} maxLength={40}
      onChange={e => setName(e.target.value)} onKeyDown={e => { if (e.key === "Enter" && name.trim()) save(); }} />
    <button type="button" className="btn" onClick={save} disabled={!name.trim()}><Save size={14} /> {exists ? "Üzerine kaydet" : "Profili kaydet"}</button>
  </fieldset>;
}

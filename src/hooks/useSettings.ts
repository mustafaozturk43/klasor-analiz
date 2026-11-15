import { useCallback, useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { defaultExtGroups, getSettings, saveSettings } from "../lib/settingsCommands";
import type { AppSettings, ExtGroup } from "../types/settings";

const LEGACY_KEY = "sfo-watch-folder";
const MIGRATED_KEY = "sfo-settings-migrated";

/**
 * Eski "Izlenen klasor" ayarini (localStorage) bir kez yeni ayarlara aktarir.
 * Klasor kaynak listesine eklenir ama otomasyon her zaman KAPALI kalir.
 */
function migrateLegacy(s: AppSettings): AppSettings | null {
  try {
    if (localStorage.getItem(MIGRATED_KEY)) return null;
    localStorage.setItem(MIGRATED_KEY, "1");
    const folder = (JSON.parse(localStorage.getItem(LEGACY_KEY) ?? "{}") as { folder?: string }).folder?.trim();
    if (!folder || s.sourceFolders.some(f => f.toLowerCase() === folder.toLowerCase())) return null;
    return { ...s, sourceFolders: [...s.sourceFolders, folder], automationEnabled: false };
  } catch {
    return null;
  }
}

/** Ilk yukleme + eski ayar aktarimi bir kez yapilir (StrictMode'da efekt iki kez calisir). */
let initial: Promise<{ settings: AppSettings; notice: string | null }> | null = null;
function loadInitial() {
  initial ??= (async () => {
    const s = await getSettings();
    const migrated = migrateLegacy(s);
    if (!migrated) return { settings: s, notice: null };
    try {
      return { settings: await saveSettings(migrated), notice: "Eski izlenen klasör kaynak klasörlere eklendi. Otomasyon kapalı; açmak için yedek klasörünü seçin." };
    } catch {
      return { settings: s, notice: null };
    }
  })();
  initial.catch(() => { initial = null; });
  return initial;
}

/** Ayarlar ekraninin durumu: kayitli hal, duzenlenen taslak, kaydetme. */
export function useSettings() {
  const [saved, setSaved] = useState<AppSettings | null>(null);
  const [draft, setDraft] = useState<AppSettings | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    let alive = true;
    loadInitial().then(({ settings, notice }) => {
      if (!alive) return;
      setSaved(settings);
      setDraft(settings);
      setNotice(notice);
    }, e => { if (alive) setError(String(e)); });
    return () => { alive = false; };
  }, []);

  const dirty = useMemo(() => !!draft && JSON.stringify(draft) !== JSON.stringify(saved), [draft, saved]);

  const update = useCallback((patch: Partial<AppSettings>) => {
    setDraft(d => (d ? { ...d, ...patch } : d));
    setError(null);
    setNotice(null);
  }, []);

  const setGroup = useCallback((index: number, group: ExtGroup | null) => {
    setDraft(d => {
      if (!d) return d;
      const groups = [...d.extGroups];
      if (group) groups[index] = group; else groups.splice(index, 1);
      return { ...d, extGroups: groups };
    });
    setError(null);
  }, []);

  const addGroup = useCallback(() => {
    setDraft(d => (d ? { ...d, extGroups: [...d.extGroups, { name: `Yeni grup ${d.extGroups.length + 1}`, extensions: [] }] } : d));
  }, []);

  const resetGroups = useCallback(async () => {
    try { update({ extGroups: await defaultExtGroups() }); } catch (e) { setError(String(e)); }
  }, [update]);

  const pickFolder = useCallback(async (title: string) => {
    const picked = await open({ directory: true, multiple: false, title });
    return typeof picked === "string" && picked ? picked : null;
  }, []);

  const save = useCallback(async () => {
    if (!draft) return;
    setSaving(true);
    try {
      const clean = await saveSettings(draft);
      setSaved(clean);
      setDraft(clean);
      setError(null);
      setNotice("Ayarlar kaydedildi.");
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }, [draft]);

  const discard = useCallback(() => { setDraft(saved); setError(null); setNotice(null); }, [saved]);

  return { draft, saved, dirty, saving, error, notice, update, setGroup, addGroup, resetGroups, pickFolder, save, discard };
}

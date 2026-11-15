import { useCallback, useEffect, useState } from "react";
import { defaultOrganizeRules } from "../lib/organizeCommands";
import {
  DEFAULT_TOGGLES, loadProfiles, loadSettings, saveProfiles, saveSettings, validateRules,
} from "../lib/organizeSettings";
import type { OrganizeProfile, OrganizeToggles, RuleDef } from "../types/organize";

/** Sabit referans: her render'da yeni dizi olusup onizlemeyi sifirlamasin. */
const NO_RULES: RuleDef[] = [];

/**
 * Duzenleme ayarlari: secenekler, tur klasoru kurallari ve kayitli profiller.
 * Son ayarlar ve profiller settings.json icinde kalir; varsayilan kurallar Rust'tan gelir.
 */
export function useOrganizeSettings() {
  const [initial] = useState(loadSettings);
  const [toggles, setToggles] = useState<OrganizeToggles>(initial.toggles);
  /** null = varsayilan kurallar. */
  const [customRules, setCustomRules] = useState<RuleDef[] | null>(initial.rules);
  const [defaults, setDefaults] = useState<RuleDef[]>([]);
  const [profiles, setProfiles] = useState<OrganizeProfile[]>(loadProfiles);
  const [activeProfile, setActiveProfile] = useState<string | null>(null);

  useEffect(() => {
    defaultOrganizeRules().then(r => setDefaults(Array.isArray(r) ? r : [])).catch(() => setDefaults([]));
  }, []);
  useEffect(() => { saveSettings({ rules: customRules, toggles }); }, [customRules, toggles]);

  const rules = customRules ?? defaults;
  const rulesError = customRules ? validateRules(customRules) : null;

  const setToggle = useCallback((key: keyof OrganizeToggles, value: boolean) => {
    setToggles(prev => ({ ...prev, [key]: value }));
    setActiveProfile(null);
  }, []);

  const setRules = useCallback((next: RuleDef[]) => {
    setCustomRules(next);
    setActiveProfile(null);
  }, []);

  const resetRules = useCallback(() => {
    setCustomRules(null);
    setActiveProfile(null);
  }, []);

  const saveProfile = useCallback((rawName: string) => {
    const name = rawName.trim();
    if (!name) return;
    setProfiles(prev => {
      const next = [...prev.filter(p => p.name.toLocaleLowerCase("tr") !== name.toLocaleLowerCase("tr")),
        { name, rules: customRules ?? [], toggles }].sort((a, b) => a.name.localeCompare(b.name, "tr"));
      saveProfiles(next);
      return next;
    });
    setActiveProfile(name);
  }, [customRules, toggles]);

  const applyProfile = useCallback((name: string) => {
    const p = profiles.find(x => x.name === name);
    if (!p) return;
    setToggles({ ...DEFAULT_TOGGLES, ...p.toggles });
    setCustomRules(p.rules.length ? p.rules : null);
    setActiveProfile(p.name);
  }, [profiles]);

  const deleteProfile = useCallback((name: string) => {
    setProfiles(prev => {
      const next = prev.filter(p => p.name !== name);
      saveProfiles(next);
      return next;
    });
    setActiveProfile(cur => (cur === name ? null : cur));
  }, []);

  return {
    toggles, setToggle,
    rules, isCustomRules: customRules !== null, rulesError, setRules, resetRules,
    /** Rust'a gonderilecek liste (bos = varsayilan). */
    rulesForBackend: customRules ?? NO_RULES,
    profiles, activeProfile, saveProfile, applyProfile, deleteProfile,
  };
}

export type OrganizeSettings = ReturnType<typeof useOrganizeSettings>;

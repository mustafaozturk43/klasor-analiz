import { useEffect, useState } from "react";
import { getPref, setPref, THEME_KEY } from "../lib/prefs";

export type Theme = "light" | "dark" | "auto";

function systemPrefersDark(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function readStoredTheme(): Theme {
  const saved = getPref<string>(THEME_KEY);
  return saved === "light" || saved === "dark" ? saved : "auto";
}

/**
 * Tema yonetimi: "light" | "dark" | "auto" (sistem tercihi).
 * Secim `settings.json` icinde saklanir (localStorage yalnizca ilk boyama onbellegidir); `html.dark` sinifi ve `color-scheme`
 * ozelligi uygulanir (Tailwind v4 custom dark variant ile uyumlu).
 */
export function useTheme() {
  const [theme, setTheme] = useState<Theme>(readStoredTheme);

  useEffect(() => {
    if (getPref<string>(THEME_KEY) !== theme) setPref(THEME_KEY, theme);
    try { localStorage.setItem(THEME_KEY, theme); } catch { /* onbellek yazilamadi: yalnizca ilk boyama etkilenir */ }
  }, [theme]);

  useEffect(() => {
    const apply = () => {
      const dark = theme === "dark" || (theme === "auto" && systemPrefersDark());
      document.documentElement.classList.toggle("dark", dark);
      document.documentElement.style.colorScheme = dark ? "dark" : "light";
    };

    apply();

    // "auto" modunda sistem temi degisimini dinle.
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    mq.addEventListener("change", apply);
    return () => mq.removeEventListener("change", apply);
  }, [theme]);

  return { theme, setTheme };
}

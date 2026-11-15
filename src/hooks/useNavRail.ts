import { useCallback, useEffect, useState } from "react";
import { getPref, setPref } from "../lib/prefs";

const NARROW = "(max-width: 60rem)";
const KEY = "navCollapsed";

/** Kenar cubugu ikon seridi mi? Dar ekranda kendiliginden; genis ekranda kullanicinin tercihi (`uiPrefs.navCollapsed`). */
export function useNavRail() {
  const [narrow, setNarrow] = useState(() => window.matchMedia(NARROW).matches);
  const [collapsed, setCollapsed] = useState(() => getPref<boolean>(KEY) ?? false);
  useEffect(() => {
    const mq = window.matchMedia(NARROW);
    const onChange = () => setNarrow(mq.matches);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);
  const toggle = useCallback(() => {
    setPref(KEY, !collapsed);
    setCollapsed(!collapsed);
  }, [collapsed]);
  return { rail: narrow || collapsed, narrow, collapsed, toggle };
}

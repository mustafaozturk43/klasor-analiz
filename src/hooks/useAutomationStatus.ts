import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getAutomationStatus, setAutomationPaused, type AutomationStatus } from "../lib/automationCommands";

/** Otomasyon durumu (kenar cubugu icin): ilk okuma, `automation-status` olayi ve duraklat/devam. */
export function useAutomationStatus() {
  const [status, setStatus] = useState<AutomationStatus | null>(null);
  useEffect(() => {
    let alive = true;
    void getAutomationStatus().then(s => { if (alive) setStatus(s); }).catch(() => undefined);
    const off = listen<AutomationStatus>("automation-status", e => setStatus(e.payload));
    return () => { alive = false; void off.then(f => f()).catch(() => undefined); };
  }, []);
  const togglePause = useCallback(() => {
    if (!status) return;
    void setAutomationPaused(!status.paused).then(setStatus).catch(() => undefined);
  }, [status]);
  return { status, togglePause };
}

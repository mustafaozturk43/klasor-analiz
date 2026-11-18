import { useCallback, useRef } from "react";
import { check } from "@tauri-apps/plugin-updater";
import { confirmDialog } from "../lib/commands";
import { describeUpdate, downloadText } from "../lib/updater";
import type { StatusMessage } from "../types/ui";

/** "Güncellemeleri denetle": yeni sürüm varsa onay sonrasi indirir ve kurar (Windows'ta uygulama kapanip yeniden acilir). */
export function useUpdater(setStatus: (s: StatusMessage | null) => void, busy: boolean) {
  const running = useRef(false);
  return useCallback(async () => {
    if (running.current || busy) return;
    running.current = true;
    try {
      setStatus({ type: "info", text: "Güncellemeler denetleniyor…" });
      const update = await check();
      if (!update) {
        setStatus({ type: "success", text: "Uygulama güncel." });
        return;
      }
      setStatus(null);
      if (!await confirmDialog(describeUpdate(update), "Güncelleme var", "İndir ve kur")) return;
      let downloaded = 0;
      let total: number | null | undefined = null;
      let lastText = "";
      await update.downloadAndInstall(event => {
        if (event.event === "Started") total = event.data.contentLength;
        if (event.event === "Progress") downloaded += event.data.chunkLength;
        const text = downloadText(downloaded, total);
        if (text !== lastText) { lastText = text; setStatus({ type: "info", text }); }
      });
      setStatus({ type: "success", text: "Güncelleme kuruldu; uygulama yeniden başlıyor." });
    } catch (e) {
      setStatus({ type: "error", text: `Güncelleme yapılamadı: ${String(e)}` });
    } finally {
      running.current = false;
    }
  }, [setStatus, busy]);
}

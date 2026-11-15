import { useEffect, useState } from "react";
import { Inbox, Pause, Play } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import type { OrganizerState } from "../types/app";
import { getAutomationStatus, setAutomationPaused, type AutomationStatus, type InboxMove, type InboxUpdate } from "../lib/automationCommands";
import { baseName, formatCount } from "../lib/format";
import VirtualRows from "./VirtualRows";

interface LogEntry extends InboxMove { at: number }
const MAX_LOG = 200;
const time = (ms: number) => new Date(ms).toLocaleTimeString("tr-TR", { hour: "2-digit", minute: "2-digit", second: "2-digit" });

/** Gelen kutusu izleme durumu (Rust watcher), duraklatma ve bu oturumdaki son tasimalar. */
export default function InboxStatusPanel({ app }: { app: OrganizerState }) {
  const [status, setStatus] = useState<AutomationStatus | null>(null);
  const [log, setLog] = useState<LogEntry[]>([]);
  const [error, setError] = useState<string | null>(null);
  const { refreshHistory } = app;

  useEffect(() => {
    void getAutomationStatus().then(setStatus).catch(e => setError(String(e)));
    const offs = [
      listen<AutomationStatus>("automation-status", e => setStatus(e.payload)),
      listen<string>("inbox-error", e => setError(e.payload)),
      listen<InboxUpdate>("inbox-updated", e => {
        const at = Date.now();
        if (e.payload.moved.length) {
          setLog(prev => [...e.payload.moved.map(m => ({ ...m, at })), ...prev].slice(0, MAX_LOG));
          void refreshHistory();
        }
        setError(e.payload.failed.length ? `${formatCount(e.payload.failed.length)} dosya taşınamadı: ${e.payload.failed[0].error}` : null);
      }),
    ];
    return () => { for (const off of offs) void off.then(f => f()).catch(() => undefined); };
  }, [refreshHistory]);

  const togglePause = async () => {
    if (!status) return;
    try { setStatus(await setAutomationPaused(!status.paused)); } catch (e) { setError(String(e)); }
  };

  return <section className="panel">
    <div className="panel-head">
      <div><h2><Inbox size={18} /> Gelen kutusu</h2>
        <p>Kaynak klasörlere gelen dosyalar yazımları bitince yedek klasöründeki günün “Bakılacaklar” klasörüne taşınır. Uygulama penceresi kapalıyken de sistem tepsisinden çalışır. Kaynaklar, yedek klasörü ve gruplar Ayarlar sekmesinde.</p></div>
      <div className="inline-actions">
        <span className={`badge ${status?.running ? "ok" : ""}`} role="status">{status?.label ?? "Durum okunuyor…"}</span>
        <button type="button" className="btn" onClick={() => void togglePause()} disabled={!status?.enabled}>
          {status?.paused ? <><Play size={14} /> Devam et</> : <><Pause size={14} /> Duraklat</>}
        </button>
      </div>
    </div>
    {error && <p className="error-state" role="alert">{error}</p>}
    {log.length ? <>
      <p className="organize-group-label px-5 pt-3">Bu oturumda taşınanlar · {formatCount(log.length)}</p>
      <VirtualRows items={log} estimate={52} maxHeight={320} itemKey={e => `${e.at}:${e.from}`} render={e => <div className="organize-row">
        <div className="min-w-0"><span className="name block">{baseName(e.from)}</span><small>{time(e.at)} · {e.group}</small></div>
        <div className="target">{baseName(e.to)}</div>
        <button type="button" className="btn" onClick={() => void app.reveal(e.to)} aria-label={`${baseName(e.to)} klasörde göster`}>Göster</button>
      </div>} />
    </> : <div className="empty-state">{status?.running ? "Yeni dosya bekleniyor." : "Otomasyon açıldığında taşınan dosyalar burada listelenir."}</div>}
  </section>;
}

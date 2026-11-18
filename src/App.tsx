import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useAppState } from "./hooks/useAppState";
import { useTheme } from "./hooks/useTheme";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useAutomationStatus } from "./features/rules/useAutomationStatus";
import { useHealth } from "./features/map/useHealth";
import { useNavRail } from "./hooks/useNavRail";
import { statusAlert } from "./lib/alerts";
import { exportHistoryCsv, openFileDefault, pickCsvTarget } from "./lib/commands";
import { formatBytes, formatCount } from "./lib/format";
import { DESTINATIONS, defaultViews, destinationOf } from "./lib/navigation";
import type { Destination, Module } from "./types/app";
import type { FileItem, FolderStatsResult } from "./types/file";
import WorkspaceHeader from "./components/WorkspaceHeader";
import WorkspaceSettings from "./components/WorkspaceSettings";
import FolderPickerBar from "./components/FolderPickerBar";
import ScreenHeader from "./components/ScreenHeader";
import FolderAnalysis from "./features/map/FolderAnalysis";
import CompareView from "./features/duplicates/CompareView";
import GroupsView from "./features/duplicates/GroupsView";
import FileBrowser from "./features/map/FileBrowser";
import HistoryPanel from "./features/history/HistoryPanel";
import RulesView from "./features/rules/RulesView";
import HealthView from "./features/map/HealthView";
import VisualView from "./features/duplicates/VisualView";
import QuarantineView from "./features/duplicates/QuarantineView";
import AutomationView from "./features/rules/AutomationView";
import EveningPanel from "./features/today/EveningPanel";
import DateGroupView from "./features/dates/DateGroupView";
import RuleEngineView from "./features/engine/RuleEngineView";
import NamingView from "./features/rename/NamingView";
import FilePreview from "./features/map/FilePreview";
import CleanupTable from "./features/map/CleanupTable";
import AppNav from "./components/AppNav";

// Ust bilgiler yalnizca ilgili ekranlarda: tarama kapsami onu kullananlarda,
// sayi seridi ve tarama rozeti tarama sonucu gosterenlerde.
const SCAN_SCOPE = new Set<Module>(["analysis", "compare", "files", "groups", "settings", "health", "visual", "automation"]);
const SCAN_STATS = new Set<Module>(["analysis", "compare", "files", "groups"]);

export default function App() {
  const app = useAppState();
  const { theme, setTheme } = useTheme();
  const automation = useAutomationStatus();
  const nav = useNavRail();
  const [dest, setDest] = useState<Destination>("today");
  const [views, setViews] = useState(defaultViews);
  const [both, setBoth] = useState(true);
  const [previewFile, setPreviewFile] = useState<FileItem | null>(null);
  const [mapStats, setMapStats] = useState<FolderStatsResult | null>(null);
  const health = useHealth(app);
  const seen = useRef(new Set<Module>());
  const module: Module = views[dest];
  /** Bir ekrana git: hedefi ve o hedefin alt sekmesini birlikte ayarlar. */
  const go = useCallback((m: Module) => { const d = destinationOf(m); setViews(v => ({ ...v, [d]: m })); setDest(d); }, []);
  useEffect(() => { void app.refreshHistory(); }, [app.refreshHistory]);
  // Durum bildirimleri (basari, bilgi, hata) sag ustte SweetAlert bildirimi olarak gosterilir.
  useEffect(() => { if (app.status) void statusAlert(app.status, () => app.setStatus(null)); }, [app.status, app.setStatus]);
  // Tepsi menusu bir ekrani acmak isteyebilir (orn. yedek klasoru secilmemisse Ayarlar).
  useEffect(() => {
    const off = listen<Module>("open-module", e => go(e.payload));
    return () => { void off.then(f => f()).catch(() => undefined); };
  }, [go]);
  useKeyboardShortcuts({
    visibleGroups: module === "compare" ? app.groups : [],
    scan: () => { if (!app.isScanning && !app.isMoving) void app.scan(); },
    undoLast: () => { if (!app.isScanning && !app.isMoving) void app.undoLast(); },
    toggleAllVisible: groups => { if (!app.isScanning && !app.isMoving) app.toggleAllVisible(groups); },
  });
  const exact = app.groups.filter(g => g.groupType === "Exact");
  const wasted = exact.reduce((s, g) => s + g.wastedBytes, 0);
  const stats = [
    ["Taranan dosya", formatCount(app.scanResult?.totalFiles ?? 0), "Son tarama kapsamı"],
    ["Bulunan grup", formatCount(app.groups.length), `${exact.length} birebir, ${app.groups.length - exact.length} benzer`],
    ["Kopya alanı", formatBytes(wasted), "Karantinaya taşınır; otomatik silinmez"],
    ["Seçili grup", String(app.selectedGroups.length), `${app.selectedFileCount} dosya`],
  ];
  const def = DESTINATIONS.find(d => d.id === dest)!;
  // Klasor secimi yalnizca "Klasör-Dosya İşlemleri" grubunda (Kopyalar, Harita, Adlandirma) gorunur.
  const ops = def.group === "ops";
  const top = mapStats ? [...mapStats.folders].sort((a, b) => b.totalBytes - a.totalBytes).slice(0, 2) : [];
  const topShare = mapStats?.rootTotalBytes ? Math.round(top.reduce((s, f) => s + f.totalBytes, 0) / mapStats.rootTotalBytes * 100) : 0;
  const mapView = dest === "map" && module === "analysis" && !!mapStats;
  const title = dest === "duplicates" && wasted > 0 ? `${formatBytes(wasted)} geri kazanılabilir`
    : mapView && mapStats ? `${formatBytes(mapStats.rootTotalBytes)}, ${formatCount(mapStats.rootFileCount)} dosya`
    : dest === "engine" ? "Kendi kurallarınla düzenle"
    : dest === "rename" ? "Adları tek kurala bağla"
    : dest === "dates" ? "Dosyaları Yıl\\Ay klasörlerine topla"
    : dest === "rules" && module === "settings" ? "Kurallar cümle gibi okunur" : def.label;
  const sub = mapView && top.length > 1 ? `${top[0].name} ve ${top[1].name}, alanın yüzde ${topShare}’ini tutuyor. Kutuya tıklayıp içine girebilirsin.`
    : dest === "dates" ? "Seçili klasördeki dosyalar çekim ya da oluşturma tarihine göre 2026\\01 gibi klasörlere taşınır. Önce önizlersin; onaylamadan hiçbir dosya taşınmaz."
    : dest === "engine" ? "Uzantı, ad, boyut ve yaş koşullarıyla dosyaları klasöre taşı ya da Geri Dönüşüm Kutusu'na gönder. Önce önizlersin; onaylamadan hiçbir dosyaya dokunulmaz."
    : dest === "rename" ? "Seçili klasörün içindeki klasör ve dosya adları aşağıdaki kurallara göre düzenlenir. Önce önizlersin; onaylamadan hiçbir ad değişmez."
    : dest === "rules" && module === "settings" ? "İzlenen klasöre düşen dosya, uzantısına göre günün klasörüne taşınır. Uyan grup yoksa dosya olduğu yerde kalır."
    : dest === "duplicates" && app.groups.length > 0 ? `${exact.length} birebir kopya, ${app.groups.length - exact.length} benzer ad. Ana dosya yerinde kalır, kopyalar silinmez, karantinaya gider.`
    : null;
  // Ekranlar ilk acildiklarinda baglanir, sonra gizlenerek durumlarini korur: acilista
  // yalnizca gorunen ekranin hook'lari calisir.
  seen.current.add(module);
  const on = (m: Module) => seen.current.has(m);
  const idle = !app.isScanning && !app.isMoving;
  const scoped = SCAN_SCOPE.has(module);
  const exportHistory = async () => {
    try {
      const target = await pickCsvTarget("islem-gecmisi.csv");
      if (target) app.setStatus({ type: "success", text: `İşlem geçmişi kaydedildi · ${formatCount(await exportHistoryCsv(target))} satır → ${target}` });
    } catch (e) { app.setStatus({ type: "error", text: String(e) }); }
  };
  return <div className="app-shell" data-rail={nav.rail || undefined}>
    <WorkspaceHeader app={app} theme={theme} setTheme={setTheme} setModule={go} nav={nav} />
    <div className="app-body">
    <AppNav dest={dest} setDest={setDest} counts={{ duplicates: app.groups.length }} automation={automation.status} onTogglePause={automation.togglePause} rail={nav.rail} />
    <main className="shell-scroll" id="workspace"><div className="shell-width workspace">
      {ops && <FolderPickerBar app={app} scanControls={module !== "naming" && module !== "dateGroup" && module !== "ruleEngine" && module !== "visual"}>{scoped && <WorkspaceSettings app={app} embedded />}</FolderPickerBar>}
      {dest !== "today" && <ScreenHeader dest={def} module={module} onPick={go} title={title} sub={sub} stats={SCAN_STATS.has(module) ? stats : null}
        scan={!SCAN_STATS.has(module) ? null : app.isScanning ? "busy" : app.scanResult ? "done" : "idle"} />}
      {!ops && scoped && <WorkspaceSettings app={app} />}
      <div hidden={module !== "analysis"} className="analysis-stack">
        <div className="inline-actions">
          <label htmlFor="analysis-scope" className="hint">Analiz kapsamı</label>
          <select id="analysis-scope" value={both && app.rootB ? "both" : "single"} onChange={e => setBoth(e.target.value === "both")}>
            <option value="single">Yalnızca Klasör A</option><option value="both" disabled={!app.rootB}>Klasör A ve B, alt alta</option>
          </select>
        </div>
        {on("analysis") && <FolderAnalysis root={app.rootPath} token={app.statsToken} label="Klasör A" onStats={setMapStats}
          aside={<CleanupTable app={app} health={health} onGo={go} />} onOpen={idle ? app.openFolder : undefined}
          onUp={app.canGoUp && idle ? () => void app.goUp(app.rootPath) : undefined} />}
        {on("analysis") && both && app.rootB && <FolderAnalysis root={app.rootB} token={app.statsToken} label="Klasör B" />}
      </div>
      <div hidden={module !== "compare"}>{on("compare") && <CompareView app={app} onPreview={setPreviewFile} />}</div>
      <div hidden={module !== "files"}>{on("files") && <FileBrowser app={app} />}</div>
      <fieldset hidden={module !== "groups"} disabled={!idle}>
        {on("groups") && <GroupsView active={dest === "duplicates" && module === "groups"} app={app} onPreview={setPreviewFile} />}
      </fieldset>
      <div hidden={module !== "health"}>{on("health") && <HealthView app={app} health={health} />}</div>
      <div hidden={module !== "visual"}>{on("visual") && <VisualView app={app} />}</div>
      <div hidden={module !== "quarantine"}>{on("quarantine") && <QuarantineView app={app} active={dest === "duplicates" && module === "quarantine"} />}</div>
      <div hidden={module !== "naming"}>{on("naming") && <NamingView app={app} active={module === "naming"} />}</div>
      <div hidden={module !== "dateGroup"}>{on("dateGroup") && <DateGroupView app={app} active={module === "dateGroup"} />}</div>
      <div hidden={module !== "ruleEngine"}>{on("ruleEngine") && <RuleEngineView app={app} active={module === "ruleEngine"} />}</div>
      <div hidden={module !== "evening"}><EveningPanel app={app} active={dest === "today"} onOpenSettings={() => go("settings")} /></div>
      <div hidden={module !== "automation"}>{on("automation") && <AutomationView app={app} />}</div>
      <div hidden={module !== "settings"}>{on("settings") && <RulesView app={app} />}</div>
      <fieldset hidden={module !== "history"} disabled={app.isMoving || app.isScanning} className="panel p-4"><HistoryPanel history={app.history} onUndo={id => void app.undoEntry(id)} onClear={app.clearHistory} onExport={() => void exportHistory()} /></fieldset>
    </div></main>
    </div>
    <FilePreview file={previewFile} onClose={() => setPreviewFile(null)} onReveal={app.reveal} onOpen={path => void openFileDefault(path).catch(e => app.setStatus({ type: "error", text: String(e) }))} />
  </div>;
}

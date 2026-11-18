import { Download, History, PanelLeft, RefreshCw, Search, Undo2 } from "lucide-react";
import type { OrganizerState, Module } from "../types/app";
import type { Theme } from "../hooks/useTheme";
import SettingsMenu from "./SettingsMenu";

interface Props {
  app: OrganizerState;
  theme: Theme;
  setTheme: (theme: Theme) => void;
  setModule: (module: Module) => void;
  /** Kenar cubugu daraltma dugmesi; dar ekranda serit zaten zorunlu oldugundan gizlenir. */
  nav: { narrow: boolean; collapsed: boolean; toggle: () => void };
  onPalette: () => void;
  onCheckUpdate: () => void;
}

/** Ust cubuk: yalnizca genel araclar. Klasor secimi sayfa basindadir (`FolderPickerBar`). */
export default function WorkspaceHeader({ app, theme, setTheme, setModule, nav, onPalette, onCheckUpdate }: Props) {
  return <div className="shell-top"><header className="shell-width compact-header">
    {!nav.narrow && <button className="btn" aria-pressed={nav.collapsed} onClick={nav.toggle}
      title={nav.collapsed ? "Kenar çubuğunu genişlet" : "Kenar çubuğunu daralt"} aria-label="Kenar çubuğunu daralt veya genişlet"><PanelLeft size={16} /></button>}
    <div className="brand" role="status">{app.progress && <p>{app.progress.phase === "hashing" ? "Hash hesaplanıyor" : "Dosyalar okunuyor"} · {app.progress.filesDone.toLocaleString("tr-TR")} / {app.progress.totalFiles || "…"}</p>}</div>
    <div className="header-tools">
      <button className="btn" title="Komut paleti (Ctrl+K)" aria-label="Komut paleti (Ctrl+K)" onClick={onPalette}><Search size={16} /></button>
      <button className="btn" title="Son işlemi geri al" aria-label="Son işlemi geri al" onClick={app.undoLast} disabled={!app.history.length || app.isMoving || app.isScanning}><Undo2 size={16} /></button>
      <details className="relative"><summary className="btn" aria-label="Diğer işlemler">•••</summary>
        <div className="panel absolute right-0 top-full z-30 mt-2 flex min-w-52 flex-col gap-2 p-3">
          <button className="btn" onClick={() => setModule("history")}><History size={16} /> İşlem geçmişi</button>
          <button className="btn" disabled={app.isScanning || app.isMoving} onClick={onCheckUpdate}><RefreshCw size={16} /> Güncellemeleri denetle</button>
          <button className="btn" disabled={!app.groups.length} onClick={() => void app.exportCsv(app.groups)}><Download size={16} /> CSV dışa aktar</button>
          <button className="btn" disabled={!app.rootPath || app.isScanning || app.isMoving} onClick={app.clearQuarantineFolder}>Karantinayı temizle…</button>
        </div>
      </details>
      <SettingsMenu theme={theme} onThemeChange={setTheme} bannedExtensions={app.bannedExtensions}
        onAddBanned={app.addBannedExtension} onRemoveBanned={app.removeBannedExtension} onResetBanned={app.resetBannedExtensions} />
    </div>
  </header></div>;
}

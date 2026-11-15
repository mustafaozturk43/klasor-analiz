/** Gelen kutusu otomasyonu ayarlari (Rust `settings.rs` ile ayni sekil). */
export interface ExtGroup {
  name: string;
  extensions: string[];
}

export interface AppSettings {
  sourceFolders: string[];
  backupRoot: string;
  extGroups: ExtGroup[];
  automationEnabled: boolean;
  /** Hicbir gruba uymayan dosyalar "Diğer" grubuna gitsin mi? */
  includeOther: boolean;
  /** Uygulama acilinca kaynakta bekleyen dosyalar da yerlestirilsin mi? */
  scanPendingOnStart: boolean;
  /** Windows oturumu acilinca uygulama baslasin mi? */
  startWithWindows: boolean;
  /** Otomatik tasimalarda Windows bildirimi gosterilsin mi? */
  notifyMoves: boolean;
  /** Arayuz tercihleri; `lib/prefs` yonetir, `save_settings` yok sayar. */
  uiPrefs?: Record<string, unknown>;
}

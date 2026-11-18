/** Uygulama ici guncelleme (tauri-plugin-updater): GitHub Releases'taki imzali `latest.json` denetlenir. */

/** Onay penceresinde gosterilen metin; notlar cok uzunsa kirpilir. */
export function describeUpdate(u: { version: string; currentVersion: string; body?: string | null }, maxNotes = 600): string {
  const notes = (u.body ?? "").trim();
  const shown = notes.length > maxNotes ? `${notes.slice(0, maxNotes).trimEnd()}…` : notes;
  return `Yeni sürüm ${u.version} hazır (şu an ${u.currentVersion}). Güncelleme imzası doğrulanır, indirilir ve kurulur; uygulama kapanıp yeniden açılır. `
    + `Dosyalarına ve ayarlarına dokunulmaz.${shown ? `\n\nSürüm notları:\n${shown}` : ""}`;
}

/** Indirme ilerlemesi; toplam bilinmiyorsa yalnizca indirilen miktar (MB). Yuzde 10'luk adimlarla degisir. */
export function downloadText(downloaded: number, total: number | null | undefined): string {
  const mb = (n: number) => (n / 1_048_576).toFixed(1).replace(".", ",");
  if (!total) return `Güncelleme indiriliyor · ${mb(downloaded)} MB`;
  const pct = Math.min(100, Math.floor((downloaded / total) * 10) * 10);
  return `Güncelleme indiriliyor · %${pct} (${mb(downloaded)} / ${mb(total)} MB)`;
}

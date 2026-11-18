/**
 * Yalnizca gelistirme: uygulama Tauri disinda (duz tarayicida) acildiginda
 * `window.__TAURI_INTERNALS__` yerine gecer ve komutlara ornek veri dondurur.
 * Boylece arayuz tarayicida incelenebilir ve ekran goruntusu alinabilir.
 * main.tsx bunu yalnizca `import.meta.env.DEV` ve Tauri yokken yukler; uretim paketine girmez.
 * Dosya sistemine dokunmaz: tasima/silme gibi komutlar yalnizca basari doner.
 */
import * as data from "./mockData";
import { mockNamePlan } from "./mockNaming";
import type { NameRuleOptions } from "../features/rename/namingCommands";

type Args = Record<string, unknown>;
const today = () => {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
};
const root = (a: Args) => String(a.rootPath ?? a.rootA ?? data.ROOT);
const paths = (a: Args) => (a.paths as string[] | undefined) ?? [];
let settings = data.settings;
let paused = false;
const uiPrefs: Record<string, unknown> = {};

const status = () => ({
  enabled: settings.automationEnabled, paused, running: settings.automationEnabled && !paused, sources: settings.sourceFolders.length,
  label: !settings.automationEnabled ? "Otomasyon kapalı" : paused ? "Otomasyon duraklatıldı" : `Otomasyon açık · ${settings.sourceFolders.length} kaynak izleniyor`,
});

const delay = <T,>(ms: number, v: T) => new Promise<T>(r => setTimeout(() => r(v), ms));
const base = (p: string) => p.split("\\").pop() ?? p;
const moveRequest = (a: Args) => a.request as { paths: string[]; targetBase: string; group: string };
const mockMovePlan = (a: Args) => {
  const r = moveRequest(a);
  const segs = r.group.split(/[\\/]/).map(s => s.trim()).filter(Boolean);
  const targetDir = [r.targetBase, ...segs].join("\\");
  return { id: "mock-move", targetDir, createdDirs: segs.length ? [targetDir] : [], renamed: 0, entries: r.paths.map(p => ({ from: p, to: `${targetDir}\\${base(p)}` })) };
};

const handlers: Record<string, (a: Args) => unknown> = {
  "plugin:dialog|open": () => data.ROOT,
  "plugin:updater|check": () => null,
  "plugin:dialog|save": () => `${data.ROOT}\\rapor.csv`,
  scan_and_group: a => data.scanResult(root(a)),
  compare_folders: a => data.scanResult(root(a)),
  get_folder_stats: a => data.folderStats(root(a)),
  list_files: () => data.folderFiles,
  get_move_history: () => data.history,
  undo_last_move: () => data.history[0],
  undo_history_entry: () => data.history[0],
  folder_health: a => data.health(root(a)),
  list_quarantine: a => data.quarantine(root(a)),
  remove_empty_folders: a => paths(a).length,
  quarantine_files: a => ({ done: paths(a).length, failed: [] }),
  restore_quarantined: a => ({ done: paths(a).length, failed: [] }),
  delete_quarantined: a => paths(a).length,
  get_settings: () => ({ ...settings, uiPrefs }),
  save_settings: a => (settings = a.settings as typeof settings),
  set_ui_pref: a => { uiPrefs[String(a.key)] = a.value; return null; },
  default_ext_groups: () => data.settings.extGroups,
  get_automation_status: status,
  set_automation_paused: a => { paused = Boolean(a.paused); return status(); },
  list_inbox: a => data.inbox(String(a.date ?? today()), !settings.autoMove),
  set_auto_move: a => (settings = { ...settings, autoMove: Boolean(a.enabled) }),
  get_target_subfolders: () => [{ name: "Faturalar", path: `${data.BACKUP}\\Faturalar` }, { name: "Projeler", path: `${data.BACKUP}\\Projeler` }],
  move_selected_files: a => ({ moved: paths(a).map(p => ({ from: p, to: [a.target, a.group].filter(Boolean).join("\\") })), failed: [], removedDirs: 0 }),
  delete_files_to_trash: a => ({ trashed: paths(a), failed: [] }),
  preview_inbox_rename: a => ({ id: "mock-plan", entries: paths(a).map((p, i) => ({ from: p, to: p.replace(/[^\\]+$/, `dosya_${i + 1}`), warning: null })) }),
  apply_rename_plan: () => 0,
  preview_name_rules: a => mockNamePlan(root(a), a.options as NameRuleOptions),
  preview_date_group: a => {
    const r = String(a.root);
    const b = (folder: string, names: string[]) => ({ folder, count: names.length, samples: names.slice(0, 3) });
    const buckets = [b("2025\\12", ["IMG_0412.jpg", "IMG_0413.jpg"]), b("2026\\01", ["IMG_0501.jpg", "IMG_0502.jpg", "ekran.png", "IMG_0503.jpg"]), b("2026\\11", ["IMG_1101.heic"]), b("Tarihsiz", ["eski.bmp"])];
    const entries = buckets.flatMap(x => x.samples.map(n => ({ from: `${r}\\${n}`, to: `${r}\\${x.folder}\\${n}` })));
    return { id: "DG-mock", root: r, entries, buckets, inPlace: 3, fromTaken: 6, renamed: 0, createdDirs: [] };
  },
  apply_date_group: () => ({ moved: [], failed: [], removedDirs: 0 }),
  rebuild_date_index: () => 12,
  trash_health_files: a => ({ done: (a.items as unknown[]).length, failed: [] }),
  export_table_csv: a => (a.rows as unknown[]).length,
  preview_rules: a => {
    const rules = (a.rules as { name: string; action: string; enabled: boolean }[]).filter(r => r.enabled);
    const counts = rules.map((r, i) => ({ rule: r.name, action: r.action, count: 3 + i, samples: ["rapor.pdf", "fatura_ocak.pdf", "notlar.txt"] }));
    const entries = counts.flatMap(c => c.samples.map(n => ({ from: `${String(a.root)}\\${n}`, to: c.action === "trash" ? "" : `${String(a.root)}\\Hedef\\${n}`, rule: c.rule, action: c.action })));
    return { id: "RE-mock", root: String(a.root), entries, counts, scanned: 40, unmatched: 31, inPlace: 0, trashCount: entries.filter(e => !e.to).length, createdDirs: [] };
  },
  find_similar_images: a => {
    const f = (n: string, w: number, h: number, size: number, distance: number) => ({ path: `${String(a.rootPath)}\\${n}`, size, width: w, height: h, mtime: 1_700_000_000, distance });
    return { rootPath: String(a.rootPath), scanned: 12, unreadable: 1, groups: [
      { id: "VG-1", files: [f("IMG_0412.jpg", 4032, 3024, 4_200_000, 0), f("IMG_0412_kucuk.jpg", 1280, 960, 310_000, 3), f("IMG_0412_whatsapp.jpg", 1600, 1200, 280_000, 5)] },
      { id: "VG-2", files: [f("logo.png", 800, 800, 90_000, 0), f("logo_kopya.png", 800, 800, 88_000, 2)] },
    ] };
  },
  apply_rules: () => ({ moved: [], trashed: 0, failed: [], removedDirs: 0 }),
  apply_name_rules: () => 9,
  list_subfolders: a => ({ paths: ["Arsiv", "Faturalar", "Faturalar\\Nisan", "Fotograflar", "Notlar", "Projeler"].map(n => `${String(a.root)}\\${n}`), truncated: false }),
  get_shell_thumbnails: a => paths(a).map(() => null),
  read_media_dates: a => delay(300, paths(a).map(p => (/\.jpe?g$/i.test(p) ? 1718366400 : null))),
  preview_selection_move: mockMovePlan,
  apply_selection_move: a => {
    const moves = [{ from: "mock", to: String(a.planId) }];
    data.history.unshift({ id: String(a.planId), timestamp: Date.now(), tag: "Seçilenleri taşı · mock", moves });
    return { moved: moves, failed: [], removedDirs: 0 };
  },
  preview_selection_rename: a => ({ id: "mock-plan", entries: paths(a).map((p, i) => ({ from: p, to: p.replace(/[^\\]+$/, `Grup_${String(i + 1).padStart(2, "0")}${p.match(/\.[^.\\]+$/)?.[0] ?? ""}`), warning: null })) }),
};

let next = 1;
const callbacks = new Map<number, (v: unknown) => void>();

async function invoke(cmd: string, args: Args = {}) {
  if (cmd.startsWith("plugin:event|")) return next++;
  const h = handlers[cmd];
  if (h) return structuredClone(await h(args));
  return cmd.startsWith("get_") || cmd.startsWith("list_") ? [] : null;
}

Object.assign(window, {
  __TAURI_INTERNALS__: {
    invoke,
    transformCallback: (cb: (v: unknown) => void) => { const id = next++; callbacks.set(id, cb); return id; },
    unregisterCallback: (id: number) => callbacks.delete(id),
    convertFileSrc: (p: string) => p,
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main", windowLabel: "main" } },
  },
  __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener: () => undefined },
});
console.info("[dev] Tauri taklidi etkin: komutlar örnek veri döndürüyor.");

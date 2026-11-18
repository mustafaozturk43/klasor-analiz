/** Klasor sagligi: tek tikla secim onerileri ("1 yildan eski", "100 MB ustu") ve secimin ozeti. */

export interface HealthRow { path: string; size: number; mtime: number }

export interface Suggestion { id: string; label: string; pick: (row: HealthRow, nowMs: number) => boolean }

const DAY_MS = 86_400_000;
const MB = 1_048_576;

/** Sekmeye gore oneriler; bos klasor ve 0 baytlik dosya sekmelerinde yok ("Tümünü seç" yeterli). */
export const SUGGESTIONS: Partial<Record<"largest" | "oldest", Suggestion[]>> = {
  largest: [
    { id: "gt100", label: "100 MB üstü", pick: r => r.size >= 100 * MB },
    { id: "gt1000", label: "1 GB üstü", pick: r => r.size >= 1024 * MB },
  ],
  oldest: [
    { id: "gt1y", label: "1 yıldan eski", pick: (r, now) => r.mtime > 0 && now - r.mtime * 1000 >= 365 * DAY_MS },
    { id: "gt3y", label: "3 yıldan eski", pick: (r, now) => r.mtime > 0 && now - r.mtime * 1000 >= 3 * 365 * DAY_MS },
  ],
};

/** Oneriye uyan satirlarin yollari. */
export const pickPaths = (rows: HealthRow[], s: Suggestion, nowMs = Date.now()): string[] => rows.filter(r => s.pick(r, nowMs)).map(r => r.path);

/** Secili satirlarin toplam boyutu (geri kazanilacak alan). */
export const selectedBytes = (rows: HealthRow[], selected: Set<string>): number => rows.reduce((s, r) => s + (selected.has(r.path) ? r.size : 0), 0);

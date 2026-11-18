import { describe, expect, it } from "vitest";
import { healthRows, HEALTH_HEADER } from "../features/map/healthReport";
import { visualRows, VISUAL_HEADER } from "../features/duplicates/visualReport";
import { ruleRows, RULE_HEADER } from "../features/engine/ruleReport";
import type { RulePlan } from "../features/engine/ruleEngineCommands";
import type { HealthReport } from "../types/health";
import { dateText } from "./tableExport";

describe("dateText", () => {
  it("0 bos, aksi halde gg.aa.yyyy ss:dd", () => {
    expect(dateText(0)).toBe("");
    expect(dateText(new Date(2024, 5, 4, 9, 7).getTime() / 1000)).toBe("04.06.2024 09:07");
  });
});

describe("healthRows", () => {
  const r: HealthReport = {
    rootPath: "C:\\Kok", totalFiles: 3, totalBytes: 10, warnings: [],
    largest: [{ path: "C:\\Kok\\Alt\\buyuk.iso", size: 500, mtime: 0 }],
    oldest: [{ path: "C:\\Kok\\eski.doc", size: 7, mtime: 0 }],
    emptyFiles: [], emptyDirs: ["C:\\Kok\\Bos"],
  };
  it("dort bolumu tek tabloda bolum sutunuyla verir; sutun sayisi basliga esit", () => {
    const rows = healthRows(r);
    expect(rows.map(x => x[0])).toEqual(["En büyük dosyalar", "En uzun süredir değişmeyen", "Boş klasörler"]);
    expect(rows.every(x => x.length === HEALTH_HEADER.length)).toBe(true);
    expect(rows[0]).toEqual(["En büyük dosyalar", "buyuk.iso", "Alt\\buyuk.iso", "C:\\Kok\\Alt\\buyuk.iso", "500", ""]);
    expect(rows[2].slice(1, 3)).toEqual(["Bos", "Bos"]);
  });
});

describe("visualRows", () => {
  it("her grubun ilk dosyasi Evet, benzerlik yuzdesi yazilir", () => {
    const f = (path: string, distance: number) => ({ path, size: 1, width: 4, height: 3, mtime: 0, distance });
    const rows = visualRows({ rootPath: "C:\\K", scanned: 2, unreadable: 0, groups: [{ id: "VG-1", files: [f("C:\\K\\a.jpg", 0), f("C:\\K\\b.jpg", 6)] }] });
    expect(rows.map(x => [x[0], x[1], x[2], x[7]])).toEqual([["VG-1", "Evet", "a.jpg", "100"], ["VG-1", "Hayır", "b.jpg", "91"]]);
    expect(rows.every(x => x.length === VISUAL_HEADER.length)).toBe(true);
  });
});

describe("ruleRows", () => {
  it("eylem etiketi yazilir, Geri Donusum Kutusu'nda hedef bos", () => {
    const plan = { entries: [
      { from: "C:\\K\\a.pdf", to: "C:\\K\\PDF\\a.pdf", rule: "PDF", action: "move" },
      { from: "C:\\K\\b.bak", to: "", rule: "Eski", action: "trash" },
    ] } as RulePlan;
    const rows = ruleRows(plan);
    expect(rows[0]).toEqual(["PDF", "Klasöre taşı", "C:\\K\\a.pdf", "C:\\K\\PDF\\a.pdf"]);
    expect(rows[1][1]).toBe("Geri Dönüşüm Kutusu'na gönder");
    expect(rows[1][3]).toBe("");
    expect(rows.every(x => x.length === RULE_HEADER.length)).toBe(true);
  });
});

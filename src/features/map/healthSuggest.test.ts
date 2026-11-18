import { describe, expect, it } from "vitest";
import { SUGGESTIONS, pickPaths, selectedBytes, type HealthRow } from "./healthSuggest";

const NOW = Date.UTC(2026, 10, 18);
const secs = (daysAgo: number) => (NOW - daysAgo * 86_400_000) / 1000;
const row = (path: string, size: number, daysAgo: number): HealthRow => ({ path, size, mtime: secs(daysAgo) });

describe("oneriler", () => {
  it("100 MB ve 1 GB esikleri dahil", () => {
    const rows = [row("a", 100 * 1_048_576, 1), row("b", 99 * 1_048_576, 1), row("c", 1024 * 1_048_576, 1)];
    const [mb100, gb1] = SUGGESTIONS.largest!;
    expect(pickPaths(rows, mb100, NOW)).toEqual(["a", "c"]);
    expect(pickPaths(rows, gb1, NOW)).toEqual(["c"]);
  });
  it("yas esikleri; tarihi bilinmeyen (0) secilmez", () => {
    const rows = [row("yeni", 1, 100), row("iki", 1, 400), row("dort", 1, 1500), { path: "bilinmeyen", size: 1, mtime: 0 }];
    const [y1, y3] = SUGGESTIONS.oldest!;
    expect(pickPaths(rows, y1, NOW)).toEqual(["iki", "dort"]);
    expect(pickPaths(rows, y3, NOW)).toEqual(["dort"]);
  });
  it("yalniz buyuk ve eski sekmelerinde oneri var", () => expect(Object.keys(SUGGESTIONS).sort()).toEqual(["largest", "oldest"]));
});

describe("selectedBytes", () => {
  it("yalniz secili satirlarin boyutunu toplar", () => {
    const rows = [row("a", 10, 1), row("b", 20, 1), row("c", 40, 1)];
    expect(selectedBytes(rows, new Set(["a", "c"]))).toBe(50);
    expect(selectedBytes(rows, new Set())).toBe(0);
  });
});

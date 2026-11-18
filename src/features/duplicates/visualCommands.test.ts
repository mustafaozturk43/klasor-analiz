import { describe, expect, it } from "vitest";
import { canSelect, defaultSelection, similarityPct, type VisualFile, type VisualGroup } from "./visualCommands";

const f = (path: string, distance = 0): VisualFile => ({ path, size: 1, width: 10, height: 10, mtime: 1, distance });
const g = (id: string, ...paths: string[]): VisualGroup => ({ id, files: paths.map((p, i) => f(p, i)) });

describe("similarityPct", () => {
  it("0 bit %100, 64 bit %0, 6 bit %91", () => {
    expect(similarityPct(0)).toBe(100);
    expect(similarityPct(64)).toBe(0);
    expect(similarityPct(6)).toBe(91);
  });
});

describe("defaultSelection", () => {
  it("her grupta ilk dosya tutulur, digerleri secilir", () => {
    const sel = defaultSelection([g("1", "a", "b", "c"), g("2", "x", "y")]);
    expect([...sel].sort()).toEqual(["b", "c", "y"]);
  });
});

describe("canSelect", () => {
  const group = g("1", "a", "b");
  it("grupta baska tutulan dosya varsa secilebilir", () => expect(canSelect(group, new Set(), "a")).toBe(true));
  it("son tutulan dosya secilemez", () => expect(canSelect(group, new Set(["b"]), "a")).toBe(false));
});

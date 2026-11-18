import { describe, expect, it } from "vitest";
import { squarify } from "./treemap";

const BOX = { x: 0, y: 0, w: 400, h: 300 };
const items = [{ id: "a", value: 50 }, { id: "b", value: 30 }, { id: "c", value: 15 }, { id: "d", value: 5 }, { id: "z", value: 0 }];

describe("squarify", () => {
  it("bos ve sifir degerli girdide bos doner", () => {
    expect(squarify([], BOX)).toEqual([]);
    expect(squarify([{ value: 0 }], BOX)).toEqual([]);
  });
  it("sifir degerliler atlanir, geri kalan buyukten kucuge gelir", () => {
    const out = squarify(items, BOX);
    expect(out.map(b => b.id)).toEqual(["a", "b", "c", "d"]);
  });
  it("alanlar degerle orantili ve toplami kutu alanina esit", () => {
    const out = squarify(items, BOX);
    const total = out.reduce((s, b) => s + b.w * b.h, 0);
    expect(total).toBeCloseTo(BOX.w * BOX.h, 3);
    for (const b of out) expect(b.w * b.h).toBeCloseTo((b.value / 100) * BOX.w * BOX.h, 3);
  });
  it("kutular kutunun disina tasmaz ve birbirinin ustune binmez", () => {
    const out = squarify(items, BOX);
    const eps = 1e-6;
    for (const b of out) {
      expect(b.x).toBeGreaterThanOrEqual(-eps);
      expect(b.y).toBeGreaterThanOrEqual(-eps);
      expect(b.x + b.w).toBeLessThanOrEqual(BOX.w + eps);
      expect(b.y + b.h).toBeLessThanOrEqual(BOX.h + eps);
    }
    for (let i = 0; i < out.length; i++) for (let j = i + 1; j < out.length; j++) {
      const [p, q] = [out[i], out[j]];
      const overlapW = Math.min(p.x + p.w, q.x + q.w) - Math.max(p.x, q.x);
      const overlapH = Math.min(p.y + p.h, q.y + q.h) - Math.max(p.y, q.y);
      expect(overlapW > eps && overlapH > eps).toBe(false);
    }
  });
});

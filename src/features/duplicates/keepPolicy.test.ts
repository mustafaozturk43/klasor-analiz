import { describe, expect, it } from "vitest";
import type { FileItem } from "../../types/file";
import { keptIndex } from "./keepPolicy";

const f = (path: string, mtime: number): FileItem => ({ path, name: path, stem: path, extension: "", size: 1, mtime, category: "Other", hashId: "h" });
const files = [f("C:\\a\\uzun_yol\\x.txt", 20), f("C:\\x.txt", 10), f("C:\\b\\x.txt", 30)];

describe("keptIndex", () => {
  it("bos listede 0", () => expect(keptIndex([], "newest")).toBe(0));
  it("first ilk dosyayi tutar", () => expect(keptIndex(files, "first")).toBe(0));
  it("newest en yeni, oldest en eski dosyayi tutar", () => {
    expect(keptIndex(files, "newest")).toBe(2);
    expect(keptIndex(files, "oldest")).toBe(1);
  });
  it("shortestPath en kisa yolu tutar", () => expect(keptIndex(files, "shortestPath")).toBe(1));
  it("esit tarihte newest sonuncuyu, oldest ilki secer (backend ile ayni)", () => {
    const tie = [f("a", 5), f("b", 5)];
    expect(keptIndex(tie, "newest")).toBe(1);
    expect(keptIndex(tie, "oldest")).toBe(0);
  });
});

import { describe, expect, it } from "vitest";
import { baseName, formatAge, formatBytes, formatCount, parentPath, skippedNote } from "./format";

describe("formatBytes", () => {
  it("sifir ve kucuk degerler", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(512)).toBe("512 B");
  });
  it("birimleri ve virgul ayiracini kullanir", () => {
    expect(formatBytes(1536)).toBe("1,50 KB");
    expect(formatBytes(10 * 1024 * 1024)).toBe("10,0 MB");
    expect(formatBytes(5 * 1024 ** 3)).toBe("5,00 GB");
  });
});

describe("formatCount", () => {
  it("binlik ayirac olarak nokta kullanir", () => expect(formatCount(1234567)).toBe("1.234.567"));
});

describe("formatAge", () => {
  const now = Date.UTC(2026, 10, 18);
  const ago = (days: number) => now / 1000 - days * 86400;
  it("tarih yoksa ve bugun", () => {
    expect(formatAge(0, now)).toBe("tarih yok");
    expect(formatAge(ago(0), now)).toBe("bugün");
  });
  it("gun, ay ve yil", () => {
    expect(formatAge(ago(12), now)).toBe("12 gün önce");
    expect(formatAge(ago(65), now)).toBe("2 ay önce");
    expect(formatAge(ago(800), now)).toBe("2 yıl önce");
  });
});

describe("yol yardimcilari", () => {
  it("parentPath ust klasoru verir, kokte null", () => {
    expect(parentPath("C:\\a\\b")).toBe("C:\\a\\");
    expect(parentPath("C:\\a\\b\\")).toBe("C:\\a\\");
    expect(parentPath("C:\\")).toBeNull();
  });
  it("baseName son parcayi verir", () => {
    expect(baseName("C:\\a\\b.txt")).toBe("b.txt");
    expect(baseName("C:\\a\\b\\")).toBe("b");
    expect(baseName("duz")).toBe("duz");
  });
  it("skippedNote bos listede bos metin", () => {
    expect(skippedNote([])).toBe("");
    expect(skippedNote(["x"])).toContain("1 dosya/klasör atlandı (ilk: x)");
  });
});

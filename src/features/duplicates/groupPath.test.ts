import { describe, expect, it } from "vitest";
import { joinPath, leafName, resolveGroupTarget, splitGroup, suggestGroupName } from "./groupPath";

describe("splitGroup", () => {
  it("bos ad bos liste, alt klasor ayiraclari parcalanir", () => {
    expect(splitGroup("  ")).toEqual([]);
    expect(splitGroup("Faturalar/2026\\Mart")).toEqual(["Faturalar", "2026", "Mart"]);
  });
  it("sondaki noktalar kirpilir", () => expect(splitGroup("Arsiv..")).toEqual(["Arsiv"]));
  it("gecersiz adlari metinle reddeder", () => {
    expect(splitGroup("a//b")).toContain("boş bölüm");
    expect(splitGroup("a:b")).toContain("kullanılamayan karakter");
    expect(splitGroup("CON")).toContain("ayrılmış");
    expect(splitGroup("_kopyalar_karantina")).toContain("kullanılamaz");
    expect(splitGroup("..")).toContain("kullanılamaz");
  });
});

describe("yol birlestirme", () => {
  it("joinPath ve leafName", () => {
    expect(joinPath("C:\\Kok\\", "Ay")).toBe("C:\\Kok\\Ay");
    expect(leafName("C:\\Kok\\Ay\\")).toBe("Ay");
  });
});

describe("resolveGroupTarget", () => {
  it("var olan klasoru buyuk/kucuk harf duyarsiz bulur, gerisini olusturur", () => {
    const r = resolveGroupTarget("C:\\Kok", "faturalar\\Mart", ["C:\\Kok\\Faturalar"], "C:\\Baska");
    expect(r).toMatchObject({ ok: true, target: "C:\\Kok\\Faturalar\\Mart", created: ["C:\\Kok\\Faturalar\\Mart"] });
    if (r.ok) expect(r.parts).toEqual([{ name: "Faturalar", exists: true }, { name: "Mart", exists: false }]);
  });
  it("hata varsa error true, dosyalar zaten hedefteyse error false", () => {
    expect(resolveGroupTarget("C:\\Kok", "a:b", [], "C:\\X")).toMatchObject({ ok: false, error: true });
    expect(resolveGroupTarget("C:\\Kok", "", [], "C:\\KOK")).toMatchObject({ ok: false, error: false });
  });
});

describe("suggestGroupName", () => {
  const day = (m: number) => new Date(2024, m, 10, 12).getTime() / 1000;
  it("ayni ay: hedef varsa yalniz ay, yoksa ay ve yil", () => {
    expect(suggestGroupName([day(4), day(4)], true)).toBe("Mayıs");
    expect(suggestGroupName([day(4)], false)).toBe("Mayıs 2024");
  });
  it("farkli ay, bos liste veya tarihsiz dosyada null", () => {
    expect(suggestGroupName([day(4), day(5)], true)).toBeNull();
    expect(suggestGroupName([], true)).toBeNull();
    expect(suggestGroupName([day(4), null], true)).toBeNull();
  });
});

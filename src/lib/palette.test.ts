import { describe, expect, it } from "vitest";
import { filterItems, fold, nextIndex, type PaletteItem } from "./palette";

const item = (label: string, extra: Partial<PaletteItem> = {}): PaletteItem => ({ id: label, label, group: "Ekran", run: () => undefined, ...extra });

describe("fold", () => {
  it("Turkce harfleri ve buyuk/kucuk farkini yok sayar", () => {
    expect(fold("Geçmiş")).toBe("gecmis");
    expect(fold("İŞLEM")).toBe("islem");
    expect(fold("IŞIK")).toBe("isik");
    expect(fold("Öğrenci Ğ")).toBe("ogrenci g");
  });
});

describe("filterItems", () => {
  const items = [item("Kopyalar › Dosya grupları"), item("Geçmiş › İşlem geçmişi"), item("Tarihe Göre"), item("Kural motoru", { keywords: "otomasyon filtre" })];
  it("bos sorgu hepsini sirayla verir", () => expect(filterItems(items, "  ").map(i => i.label)).toEqual(items.map(i => i.label)));
  it("aksansiz yazim aksanli ada eslesir", () => {
    expect(filterItems(items, "gecmis").map(i => i.label)).toEqual(["Geçmiş › İşlem geçmişi"]);
    expect(filterItems(items, "tarihe gore")[0].label).toBe("Tarihe Göre");
  });
  it("tum sozcukler eslesmeli; anahtar sozcuklerde de aranir", () => {
    expect(filterItems(items, "kopya zzz")).toEqual([]);
    expect(filterItems(items, "otomasyon").map(i => i.label)).toEqual(["Kural motoru"]);
  });
  it("basta eslesen, sozcuk basinda eslesenden once gelir", () => {
    const list = [item("Yeni islem"), item("Islem gecmisi"), item("Son islem")];
    expect(filterItems(list, "islem").map(i => i.label)).toEqual(["Islem gecmisi", "Yeni islem", "Son islem"]);
  });
  it("sinira uyar", () => expect(filterItems(Array.from({ length: 100 }, (_, i) => item(`a${i}`)), "", 5)).toHaveLength(5));
});

describe("nextIndex", () => {
  const list = [item("a"), item("b", { disabled: true }), item("c")];
  it("devre disi satiri atlar ve donguseldir", () => {
    expect(nextIndex(list, 0, 1)).toBe(2);
    expect(nextIndex(list, 2, 1)).toBe(0);
    expect(nextIndex(list, 0, -1)).toBe(2);
  });
  it("bos ya da hepsi devre disiyken -1", () => {
    expect(nextIndex([], 0, 1)).toBe(-1);
    expect(nextIndex([item("x", { disabled: true })], 0, 1)).toBe(-1);
  });
});

import { describe, expect, it } from "vitest";
import { effectiveDate, extOf, formatDay, kindOf, ymd } from "./fileKinds";

describe("fileKinds", () => {
  it("extOf uzantiyi kucuk harfle verir; gizli dosya ve uzantisizda bos", () => {
    expect(extOf("Foto.JPG")).toBe("jpg");
    expect(extOf(".gitignore")).toBe("");
    expect(extOf("README")).toBe("");
  });
  it("kindOf uzantidan tur bulur", () => {
    expect(kindOf("a.HEIC")).toBe("image");
    expect(kindOf("a.mkv")).toBe("video");
    expect(kindOf("a.pdf")).toBe("pdf");
    expect(kindOf("a.xlsx")).toBe("doc");
    expect(kindOf("a.bin")).toBe("other");
  });
  it("effectiveDate cekim tarihini once alir", () => {
    expect(effectiveDate(100, 200)).toBe(200);
    expect(effectiveDate(100, null)).toBe(100);
    expect(effectiveDate(null, undefined)).toBeNull();
  });
  it("ymd ve formatDay yerel saatle iki haneli yazar", () => {
    const secs = new Date(2024, 5, 4, 12).getTime() / 1000;
    expect(ymd(secs)).toEqual({ y: "2024", m: "06", d: "04" });
    expect(formatDay(secs)).toBe("04.06.2024");
  });
});

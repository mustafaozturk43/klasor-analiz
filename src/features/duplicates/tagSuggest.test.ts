import { describe, expect, it } from "vitest";
import type { FileCategory, FileItem } from "../../types/file";
import { suggestTagName } from "./tagSuggest";

const f = (stem: string, category: FileCategory = "Document"): FileItem => ({ path: `C:\\${stem}.pdf`, name: `${stem}.pdf`, stem, extension: "pdf", size: 1, mtime: 1, category, hashId: "h" });

describe("suggestTagName", () => {
  it("dosya yoksa Etiket", () => expect(suggestTagName([])).toBe("Etiket"));
  it("en sik anlamli sozcuk ve en sik yil", () => {
    expect(suggestTagName([f("fatura_ocak"), f("fatura_2026_mart"), f("Fatura 2026")])).toBe("2026_Fatura");
  });
  it("gurultu sozcukleri ve sayilar atlanir", () => {
    expect(suggestTagName([f("final_v2_rapor"), f("rapor_12345")])).toBe("Rapor");
  });
  it("anlamli sozcuk yoksa kategori adina duser", () => {
    expect(suggestTagName([f("img_001", "Image")])).toBe("Resim");
    expect(suggestTagName([f("v1")])).toBe("Doküman");
  });
  it("Turkce buyuk harf kuralini kullanir", () => {
    expect(suggestTagName([f("ılık_liste"), f("ılık_not")])).toBe("Ilık");
  });
});

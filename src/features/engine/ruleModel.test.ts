import { describe, expect, it } from "vitest";
import { describeConditions, newRule, parseOptionalNumber } from "./ruleModel";

describe("describeConditions", () => {
  it("kosulsuz kuralda bos metin", () => expect(describeConditions(newRule(1))).toBe(""));
  it("tum kosullari ozetler", () => {
    const r = { ...newRule(1), exts: ["pdf", "docx"], nameContains: " fatura ", minMb: 1, minAgeDays: 30 };
    expect(describeConditions(r)).toBe("pdf, docx · adı “fatura” içerir · ≥ 1 MB · 30+ gün eski");
  });
  it("boyut araligi, yalniz ust sinir", () => {
    expect(describeConditions({ ...newRule(1), minMb: 1, maxMb: 5 })).toBe("1–5 MB");
    expect(describeConditions({ ...newRule(1), maxMb: 2.5 })).toBe("≤ 2.5 MB");
  });
});

describe("parseOptionalNumber", () => {
  it("bos ve gecersiz null, virgul kabul, eksi 0", () => {
    expect(parseOptionalNumber("")).toBeNull();
    expect(parseOptionalNumber("abc")).toBeNull();
    expect(parseOptionalNumber("1,5")).toBe(1.5);
    expect(parseOptionalNumber("-3")).toBe(0);
  });
});

describe("newRule", () => {
  it("etkin, koşulsuz ve taşıma eylemiyle başlar", () => {
    expect(newRule(2)).toMatchObject({ name: "Kural 2", enabled: true, action: "move", exts: [], target: "" });
  });
});

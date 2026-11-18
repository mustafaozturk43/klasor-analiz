import { describe, expect, it } from "vitest";
import { describeUpdate, downloadText } from "./updater";

describe("describeUpdate", () => {
  it("surumleri ve guvence cumlesini yazar; not yoksa not bolumu yok", () => {
    const t = describeUpdate({ version: "0.2.0", currentVersion: "0.1.0" });
    expect(t).toContain("0.2.0");
    expect(t).toContain("şu an 0.1.0");
    expect(t).toContain("dokunulmaz");
    expect(t).not.toContain("Sürüm notları");
  });
  it("notlari ekler, cok uzunsa kirpar", () => {
    expect(describeUpdate({ version: "1", currentVersion: "0", body: "  Yenilik  " })).toContain("Sürüm notları:\nYenilik");
    const long = describeUpdate({ version: "1", currentVersion: "0", body: "x".repeat(700) });
    expect(long.endsWith("…")).toBe(true);
    expect(long.split("\n\nSürüm notları:\n")[1]).toHaveLength(601);
  });
});

describe("downloadText", () => {
  it("toplam bilinmiyorsa yalniz MB", () => expect(downloadText(5_242_880, null)).toBe("Güncelleme indiriliyor · 5,0 MB"));
  it("yuzde 10'luk adimlarla yazilir ve 100'u asmaz", () => {
    expect(downloadText(0, 100)).toContain("%0");
    expect(downloadText(37, 100)).toContain("%30");
    expect(downloadText(250, 100)).toContain("%100");
  });
});

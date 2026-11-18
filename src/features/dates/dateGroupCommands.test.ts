import { describe, expect, it } from "vitest";
import { DATE_INDEX_FILE, LAYOUTS, dateIndexPath } from "./dateGroupCommands";

describe("dateIndexPath", () => {
  it("kok klasore dizin dosya adini ekler, sondaki ayiraci kirpar", () => {
    expect(dateIndexPath("C:\\Fotograflar")).toBe("C:\\Fotograflar\\Tarih_Dizini.html");
    expect(dateIndexPath("C:\\Fotograflar\\")).toBe("C:\\Fotograflar\\Tarih_Dizini.html");
    expect(dateIndexPath("D:/Arsiv/")).toBe("D:/Arsiv\\Tarih_Dizini.html");
  });
  it("dosya adi sabitiyle tutarli", () => expect(DATE_INDEX_FILE).toBe("Tarih_Dizini.html"));
});

describe("LAYOUTS", () => {
  it("uc duzen var ve kimlikleri Rust DateLayout ile ayni", () => {
    expect(LAYOUTS.map(l => l.id)).toEqual(["yearMonth", "month", "yearNumbered"]);
  });
});

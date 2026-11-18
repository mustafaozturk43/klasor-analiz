import { describe, expect, it } from "vitest";
import { buildLatest } from "./make-latest-json.mjs";

const base = { version: "0.2.0", pubDate: "2026-11-19T10:00:00.000Z", signature: "  dW50cnVzdGVk\n", repo: "sahip/depo", tag: "v0.2.0", fileName: "SmartFileOrganizer-0.2.0-windows-x64-setup-20261119T100000Z.exe" };

describe("buildLatest", () => {
  it("guncelleyicinin bekledigi alanlari uretir; imza kirpilir", () => {
    const l = buildLatest(base);
    expect(l.version).toBe("0.2.0");
    expect(l.pub_date).toBe(base.pubDate);
    expect(l.platforms["windows-x86_64"].signature).toBe("dW50cnVzdGVk");
    expect(l.platforms["windows-x86_64"].url).toBe("https://github.com/sahip/depo/releases/download/v0.2.0/SmartFileOrganizer-0.2.0-windows-x64-setup-20261119T100000Z.exe");
  });
  it("notlar bos birakilabilir", () => expect(buildLatest(base).notes).toBe(""));
  it("dosya adi ve etiket URL icin kodlanir", () => {
    const url = buildLatest({ ...base, fileName: "Smart File Setup.exe" }).platforms["windows-x86_64"].url;
    expect(url.endsWith("/Smart%20File%20Setup.exe")).toBe(true);
  });
  it("gecersiz surum, bos imza ve gecersiz depo reddedilir", () => {
    expect(() => buildLatest({ ...base, version: "abc" })).toThrow("Geçersiz sürüm");
    expect(() => buildLatest({ ...base, signature: "  " })).toThrow("İmza boş");
    expect(() => buildLatest({ ...base, repo: "depo" })).toThrow("Geçersiz depo");
  });
});

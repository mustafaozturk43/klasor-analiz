// Tauri'nin urettigi NSIS kurulum dosyasini artifacts/ altina kopyalar ve SHA256 yazar.
// Yalnizca bundle/nsis klasorunun dogrudan icine bakilir; target/ taranmaz.
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";

if (process.platform !== "win32") throw new Error("Kurulum paketi Windows üzerinde hazırlanmalıdır.");
const { version } = JSON.parse(fs.readFileSync("package.json", "utf8"));
const nsisDir = path.resolve("src-tauri/target/release/bundle/nsis");
const setups = fs.existsSync(nsisDir)
  ? fs.readdirSync(nsisDir).filter(f => f.endsWith("-setup.exe") && f.includes(`_${version}_`))
  : [];
if (setups.length !== 1) throw new Error(`Beklenen tek NSIS kurulum dosyası bulunamadı (${nsisDir}): ${setups.join(", ") || "yok"}`);

const source = path.join(nsisDir, setups[0]);
const bytes = fs.readFileSync(source);
if (bytes.toString("ascii", 0, 2) !== "MZ") throw new Error("Kurulum dosyası geçerli bir Windows EXE değil.");

const stamp = new Date().toISOString().replace(/[-:]/g, "").replace(/\.\d+Z$/, "Z");
fs.mkdirSync("artifacts", { recursive: true });
const target = path.resolve("artifacts", `SmartFileOrganizer-${version}-windows-x64-setup-${stamp}.exe`);
fs.copyFileSync(source, target, fs.constants.COPYFILE_EXCL);
// Guncelleyici imzasi (`createUpdaterArtifacts`): TAURI_SIGNING_PRIVATE_KEY olmadan uretilmez.
// Yerel derlemede (SFO_LOCAL_BUILD=1, createUpdaterArtifacts kapali) imza aranmaz; boyle bir kurulum dosyasi
// guncelleme olarak yayinlanamaz (latest.json uretimi .sig ister, surum is akisi anahtar olmadan durur).
const sigSource = `${source}.sig`;
if (fs.existsSync(sigSource)) {
  fs.copyFileSync(sigSource, `${target}.sig`, fs.constants.COPYFILE_EXCL);
} else if (process.env.SFO_LOCAL_BUILD === "1") {
  console.warn("Uyarı: güncelleme imzası (.sig) yok; bu kurulum dosyası yalnızca elle kurulum içindir, güncelleme olarak yayınlanamaz.");
} else {
  throw new Error(`Güncelleme imzası yok (${sigSource}). TAURI_SIGNING_PRIVATE_KEY ve TAURI_SIGNING_PRIVATE_KEY_PASSWORD ortam değişkenlerini ayarlayıp derleyin (yalnızca elle kurulum için: SFO_LOCAL_BUILD=1 ve TAURI_CONFIG ile createUpdaterArtifacts kapatın).`);
}
const hash = createHash("sha256").update(bytes).digest("hex");
fs.writeFileSync(`${target}.sha256`, `${hash}  ${path.basename(target)}\n`, "utf8");
console.log(JSON.stringify({ setup: target, bytes: bytes.length, sha256: hash }, null, 2));

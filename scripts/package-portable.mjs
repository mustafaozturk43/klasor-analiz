// Package only the exact Tauri build output; do not traverse build directories.
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";

if (process.platform !== "win32") throw new Error("Portable paket Windows üzerinde hazırlanmalıdır.");
const { version } = JSON.parse(fs.readFileSync("package.json", "utf8"));
const stamp = new Date().toISOString().replace(/[-:]/g, "").replace(/\.\d+Z$/, "Z");
const name = `SmartFileOrganizer-${version}-windows-x64-portable-${stamp}`;
const folder = path.resolve("artifacts", name);
fs.mkdirSync(folder, { recursive: true });
const exe = path.join(folder, "SmartFileOrganizer.exe");
fs.copyFileSync(path.resolve("src-tauri/target/release/smart-file-organizer.exe"), exe, fs.constants.COPYFILE_EXCL);
// Validate the copied deliverable's PE header, not files under target/.
const bytes = fs.readFileSync(exe);
if (bytes.toString("ascii", 0, 2) !== "MZ") throw new Error("Geçerli Windows EXE dosyası değil.");
const pe = bytes.readUInt32LE(0x3c);
if (bytes.toString("ascii", pe, pe + 4) !== "PE\0\0" || bytes.readUInt16LE(pe + 4) !== 0x8664) {
  throw new Error("Beklenen Windows x64 PE çıktısı bulunamadı.");
}
fs.writeFileSync(path.join(folder, "BASLANGIC.txt"), `Smart File Organizer ${version} - Windows x64 Portable

KULLANIM
1. ZIP dosyasını bir klasöre çıkarın.
2. SmartFileOrganizer.exe dosyasını çalıştırın. Kurulum gerekmez.

GEREKSİNİMLER
- Windows 10/11 x64.
- Microsoft Edge WebView2 Runtime (çoğu güncel Windows bilgisayarda bulunur).
  Resmi indirme: https://developer.microsoft.com/microsoft-edge/webview2/
- VCRUNTIME/MSVCP eksik uyarısı alırsanız Microsoft Visual C++ x64
  Redistributable gerekebilir. Rastgele sitelerden DLL indirmeyin.

VERİ SAKLAMA
Bu paket kurulum gerektirmez; tamamen USB içinde veri saklayan bir mod değildir.
Ayarlar ve işlem geçmişi Windows kullanıcı profilinde/AppData'da saklanır.
Başka bir bilgisayara EXE kopyalandığında bu ayarlar kendiliğinden taşınmaz.
Kullanıcıya ait dosyalar bu pakete dahil edilmemiştir.

GÜVENLİK
Bu derleme dijital olarak imzalanmamıştır; Windows yayıncı uyarısı gösterebilir.
Paketi yalnızca güvendiğiniz kaynaktan kullanın ve SHA256 değerini doğrulayın.
Önemli dosyalarda kullanmadan önce yedek alın ve deneme klasöründe test edin.
`, "utf8");
const exeHash = createHash("sha256").update(bytes).digest("hex");
fs.writeFileSync(path.join(folder, "SHA256SUMS.txt"), `${exeHash}  SmartFileOrganizer.exe\n`, "utf8");
const zip = `${folder}.zip`;
const quote = s => `'${s.replaceAll("'", "''")}'`;
execFileSync("powershell.exe", ["-NoProfile", "-Command",
  `Compress-Archive -LiteralPath ${quote(folder)} -DestinationPath ${quote(zip)} -CompressionLevel Optimal -ErrorAction Stop`], { stdio: "inherit" });
const zipHash = createHash("sha256").update(fs.readFileSync(zip)).digest("hex");
fs.writeFileSync(`${zip}.sha256`, `${zipHash}  ${path.basename(zip)}\n`, "utf8");
console.log(JSON.stringify({ folder, zip, exe, architecture: "Windows x64", exeBytes: bytes.length,
  zipBytes: fs.statSync(zip).size, exeSHA256: exeHash, zipSHA256: zipHash }, null, 2));

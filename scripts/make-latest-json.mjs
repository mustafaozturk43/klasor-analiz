// Guncelleyicinin okudugu `latest.json` dosyasini uretir (artifacts/latest.json).
// Kurulum dosyasinin yanindaki `.sig` (minisign imzasi) tauri build tarafindan uretilir; imza icerigi
// buraya yazilir, uygulama bunu tauri.conf.json'daki genel anahtarla dogrular.
//   node scripts/make-latest-json.mjs <etiket>      (ornek: v0.2.0; GITHUB_REPOSITORY ve RELEASE_NOTES ortam degiskenleri)
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

/** Saf: latest.json icerigini olusturur. `fileName`: surumde yuklenen kurulum dosyasinin adi. */
export function buildLatest({ version, notes = "", pubDate, signature, repo, tag, fileName }) {
  if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) throw new Error(`Geçersiz sürüm: ${version}`);
  if (!signature?.trim()) throw new Error("İmza boş: kurulum dosyasının .sig dosyası okunamadı.");
  if (!/^[\w.-]+\/[\w.-]+$/.test(repo)) throw new Error(`Geçersiz depo adı: ${repo} (örnek: sahip/depo)`);
  return {
    version,
    notes,
    pub_date: pubDate,
    platforms: {
      "windows-x86_64": {
        signature: signature.trim(),
        url: `https://github.com/${repo}/releases/download/${encodeURIComponent(tag)}/${encodeURIComponent(fileName)}`,
      },
    },
  };
}

function main() {
  const tag = process.argv[2];
  const repo = process.env.GITHUB_REPOSITORY;
  if (!tag || !repo) throw new Error("Kullanım: GITHUB_REPOSITORY=sahip/depo node scripts/make-latest-json.mjs vX.Y.Z");
  const { version } = JSON.parse(fs.readFileSync("package.json", "utf8"));
  if (tag !== `v${version}`) throw new Error(`Etiket (${tag}) package.json sürümüyle (v${version}) eşleşmiyor.`);
  const dir = path.resolve("artifacts");
  const sigs = fs.readdirSync(dir).filter(f => f.includes(`-${version}-windows-x64-setup-`) && f.endsWith(".exe.sig"));
  if (sigs.length !== 1) throw new Error(`Beklenen tek kurulum imzası bulunamadı (${dir}): ${sigs.join(", ") || "yok"}`);
  const fileName = sigs[0].slice(0, -".sig".length);
  if (!fs.existsSync(path.join(dir, fileName))) throw new Error(`İmzanın kurulum dosyası yok: ${fileName}`);
  const latest = buildLatest({
    version, notes: process.env.RELEASE_NOTES ?? "", pubDate: new Date().toISOString(),
    signature: fs.readFileSync(path.join(dir, sigs[0]), "utf8"), repo, tag, fileName,
  });
  const target = path.join(dir, "latest.json");
  fs.writeFileSync(target, `${JSON.stringify(latest, null, 2)}\n`, "utf8");
  console.log(`${target}\n${latest.platforms["windows-x86_64"].url}`);
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) main();

// Surum numarasinin tek kaynagi package.json'dur; bu betik Cargo.toml, Cargo.lock,
// tauri.conf.json ve package-lock.json'u ona esitler.
//   node scripts/sync-version.mjs            -> dosyalari package.json surumune esitler
//   node scripts/sync-version.mjs 0.2.0      -> once package.json'u 0.2.0 yapar, sonra esitler
//   node scripts/sync-version.mjs --check    -> yazmaz; uyumsuzluk varsa hata ile cikar (npm run check / CI)
import fs from "node:fs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const requested = args.find(a => !a.startsWith("--"));
const SEMVER = /^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/;

const read = f => fs.readFileSync(f, "utf8");
const pkgText = read("package.json");
const current = JSON.parse(pkgText).version;
if (requested && !SEMVER.test(requested)) throw new Error(`Geçersiz sürüm: ${requested} (örnek: 0.2.0)`);
if (requested && check) throw new Error("--check ile yeni sürüm verilemez.");
const version = requested ?? current;
if (!SEMVER.test(version)) throw new Error(`package.json sürümü geçersiz: ${version}`);

const eol = text => (text.includes("\r\n") ? "\r\n" : "\n");
/** Bir dosyada tek bir eslesmeyi degistirir; eslesme yoksa hata verir (sessizce atlanmaz). */
const replaceOnce = (text, pattern, replacement, file) => {
  if (!pattern.test(text)) throw new Error(`${file}: sürüm satırı bulunamadı.`);
  return text.replace(pattern, replacement);
};

const targets = [
  ["package.json", t => replaceOnce(t, /("version":\s*")[^"]+(")/, `$1${version}$2`, "package.json")],
  ["src-tauri/Cargo.toml", t => replaceOnce(t, /^(version\s*=\s*")[^"]+(")/m, `$1${version}$2`, "Cargo.toml")],
  ["src-tauri/tauri.conf.json", t => replaceOnce(t, /("version":\s*")[^"]+(")/, `$1${version}$2`, "tauri.conf.json")],
  ["src-tauri/Cargo.lock", t => replaceOnce(t, /(name = "smart-file-organizer"\r?\nversion = ")[^"]+(")/, `$1${version}$2`, "Cargo.lock")],
  ["package-lock.json", t => {
    const lock = JSON.parse(t);
    lock.version = version;
    if (lock.packages?.[""]) lock.packages[""].version = version;
    return `${JSON.stringify(lock, null, 2).replace(/\n/g, eol(t))}${t.endsWith("\n") ? eol(t) : ""}`;
  }],
];

const drift = [];
for (const [file, update] of targets) {
  const before = read(file);
  const after = update(before);
  if (after === before) continue;
  drift.push(file);
  if (!check) fs.writeFileSync(file, after);
}

if (check) {
  if (drift.length) {
    console.error(`Sürüm uyumsuz (package.json = ${version}): ${drift.join(", ")}\nDüzeltmek için: npm run version:set`);
    process.exit(1);
  }
  console.log(`Sürümler uyumlu: ${version}`);
} else {
  console.log(drift.length ? `Sürüm ${version} olarak güncellendi: ${drift.join(", ")}` : `Sürümler zaten ${version}.`);
}

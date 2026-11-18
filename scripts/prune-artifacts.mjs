// artifacts/ altindaki eski surum paketlerini temizler (yalnizca derleme ciktilari; kullanici dosyasi degil).
// Varsayilan: kuru calistirma (hicbir sey silinmez). Silmek icin: npm run prune:artifacts -- --yes
// En yeni N tarih damgasini tutar (varsayilan 3): npm run prune:artifacts -- --keep=5
import fs from "node:fs";
import path from "node:path";

const args = process.argv.slice(2);
const apply = args.includes("--yes");
const keepArg = args.find(a => a.startsWith("--keep="));
const keep = keepArg ? Number(keepArg.slice(7)) : 3;
if (!Number.isInteger(keep) || keep < 1) throw new Error("--keep en az 1 olan bir tam sayi olmali.");

const dir = path.resolve("artifacts");
if (!fs.existsSync(dir)) {
  console.log("artifacts/ yok; yapilacak bir sey yok.");
  process.exit(0);
}

const STAMP = /\d{8}T\d{6}Z/;
const byStamp = new Map();
for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
  const stamp = entry.name.match(STAMP)?.[0];
  if (!stamp || !entry.name.startsWith("SmartFileOrganizer-")) continue;
  byStamp.set(stamp, [...(byStamp.get(stamp) ?? []), entry]);
}

const stamps = [...byStamp.keys()].sort().reverse();
const old = stamps.slice(keep);
const size = e => {
  const p = path.join(dir, e.name);
  const walk = q => (fs.statSync(q).isDirectory() ? fs.readdirSync(q).reduce((n, f) => n + walk(path.join(q, f)), 0) : fs.statSync(q).size);
  return walk(p);
};

let total = 0;
for (const stamp of old) {
  for (const e of byStamp.get(stamp)) {
    total += size(e);
    console.log(`${apply ? "siliniyor" : "silinecek"}: ${e.name}`);
    if (apply) fs.rmSync(path.join(dir, e.name), { recursive: true, force: true });
  }
}
const mb = (total / 1024 / 1024).toFixed(1);
console.log(`${stamps.length} surum bulundu, ${Math.min(keep, stamps.length)} tutuldu, ${old.length} eski surum ${apply ? "silindi" : "silinecek"} (${mb} MB).`);
if (!apply && old.length) console.log("Silmek icin: npm run prune:artifacts -- --yes");

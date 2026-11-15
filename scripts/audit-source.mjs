// Read source directories only; never traverse build/dependency directories.
import fs from "node:fs";
import path from "node:path";
const files = [];
function walk(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const file = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(file);
    else if (/\.(ts|tsx)$/.test(file)) files.push(path.resolve(file));
  }
}
walk("src");
const seen = new Set();
function visit(file) {
  if (seen.has(file)) return;
  seen.add(file);
  const text = fs.readFileSync(file, "utf8");
  for (const match of text.matchAll(/(?:from\s*|import\s*\(?\s*)["']((?:\.|@\/)[^"']+)["']/g)) {
    // "@/" takma adi src/ klasorunu gosterir (vite.config.ts + tsconfig paths).
    const spec = match[1];
    const base = spec.startsWith("@/") ? path.resolve("src", spec.slice(2)) : path.resolve(path.dirname(file), spec);
    const next = [base + ".tsx", base + ".ts", path.join(base, "index.tsx"), path.join(base, "index.ts")].find(f => fs.existsSync(f));
    if (next) visit(next);
  }
}
visit(path.resolve("src/main.tsx"));
console.log("Unreachable source modules:");
console.log(files.filter(f => !seen.has(f) && !f.endsWith(".d.ts")).map(f => path.relative(process.cwd(), f)).join("\n") || "None");
const lib = fs.readFileSync("src-tauri/src/lib.rs", "utf8");
const handlers = lib.split("tauri::generate_handler![")[1]?.split("])")[0] ?? "";
const registered = new Set(handlers.split(",").map(s => s.trim().split("::").at(-1)).filter(Boolean));
const commands = new Set();
for (const file of files) {
  for (const match of fs.readFileSync(file, "utf8").matchAll(/invoke(?:<[^;]*?>)?\(\s*["']([^"']+)["']/g)) commands.add(match[1]);
}
const missing = [...commands].filter(command => !registered.has(command));
console.log("Unregistered invoke commands:", missing.length ? missing.join(", ") : "None");
if (missing.length) process.exitCode = 1;
const overLimit = [];
// 250 satir kurali uzun satirlarla asilmasin: tek satir en fazla 300 karakter.
const MAX_LINE = 300;
const longLines = [];
function checkLength(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const file = path.join(dir, entry.name);
    if (entry.isDirectory()) checkLength(file);
    else if (/\.(ts|tsx|rs|css)$/.test(file)) {
      const text = fs.readFileSync(file, "utf8");
      const lines = text.split(/\r?\n/).length - (text.endsWith("\n") ? 1 : 0);
      if (lines > 250) overLimit.push(`${file}: ${lines}`);
      text.split(/\r?\n/).forEach((line, i) => { if (line.length > MAX_LINE) longLines.push(`${file}:${i + 1} (${line.length})`); });
    }
  }
}
for (const dir of ["src", "src-tauri/src", "src-tauri/tests"]) checkLength(dir);
console.log("Source files over 250 lines:", overLimit.length ? overLimit.join(", ") : "None");
if (overLimit.length) process.exitCode = 1;
console.log(`Lines over ${MAX_LINE} characters:`, longLines.length ? longLines.join(", ") : "None");
if (longLines.length) process.exitCode = 1;

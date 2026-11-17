/**
 * Yalnizca gelistirme: Tauri disinda (tarayicida) arayuzu ornek veriyle acmak icin.
 * Gercek kullanici verisi icermez; paketlere girmez (bkz. main.tsx).
 */
import type { FileGroup, FileItem, FolderFile, FolderStatsResult, ScanResult, UndoRecord } from "../types/file";
import type { HealthReport, QuarantineListing } from "../types/health";
import type { InboxListing } from "../types/inbox";
import type { AppSettings } from "../types/settings";

export const ROOT = "C:\\Kullanicilar\\Ornek\\Belgeler";
export const BACKUP = "D:\\Yedek";
const now = Math.floor(Date.now() / 1000);
const day = 86400;

const item = (dir: string, name: string, size: number, age: number, hashId: string): FileItem => {
  const [stem, ext] = [name.replace(/\.[^.]+$/, ""), name.split(".").pop()!.toLowerCase()];
  const category = ["pdf", "docx", "xlsx", "txt", "pptx"].includes(ext) ? "Document" : ["jpg", "png", "webp"].includes(ext) ? "Image" : "Archive";
  return { path: `${ROOT}\\${dir}\\${name}`, name, stem, extension: ext, size, mtime: now - age * day, category, hashId };
};

const groups: FileGroup[] = [
  { id: "GRP-0001", groupType: "Exact", category: "Document", wastedBytes: 2_480_000, files: [item("Faturalar", "Elektrik_Kasim.pdf", 1_240_000, 3, "a1"), item("Indirilenler", "Elektrik_Kasim (1).pdf", 1_240_000, 2, "a1"), item("Masaustu", "Elektrik_Kasim.pdf", 1_240_000, 1, "a1")] },
  { id: "GRP-0002", groupType: "Exact", category: "Image", wastedBytes: 4_200_000, files: [item("Fotograflar", "IMG_2041.jpg", 4_200_000, 40, "b2"), item("Yedek", "IMG_2041.jpg", 4_200_000, 12, "b2")] },
  { id: "GRP-0003", groupType: "Similar", category: "Document", wastedBytes: 0, files: [item("Projeler", "Teklif_Q4.xlsx", 84_000, 9, "c3"), item("Projeler", "Teklif_Q4_son.xlsx", 91_000, 4, "c4"), item("Indirilenler", "Teklif_Q4_son2.xlsx", 92_500, 1, "c5")] },
  { id: "GRP-0004", groupType: "Exact", category: "Archive", wastedBytes: 58_000_000, files: [item("Indirilenler", "yedek_2026.zip", 58_000_000, 20, "d4"), item("Arsiv", "yedek_2026.zip", 58_000_000, 18, "d4")] },
  { id: "GRP-0005", groupType: "Similar", category: "Image", wastedBytes: 0, files: [item("Tasarim", "logo_v2.png", 220_000, 30, "e1"), item("Tasarim", "logo_v3.png", 236_000, 21, "e2")] },
];

export const scanResult = (rootPath: string): ScanResult => ({ rootPath, totalFiles: 1284, groups, comparison: null });

const folderFile = (dir: string, name: string, size: number, cat: FolderFile["category"]): FolderFile => ({ name, path: `${ROOT}\\${dir}\\${name}`, size, category: cat, createdAt: (now - 5 * day) * 1000, modifiedAt: 1718366400 });

export const folderFiles: FolderFile[] = [
  folderFile("Faturalar", "Elektrik_Kasim.pdf", 1_240_000, "Document"),
  folderFile("Faturalar", "Su_Ekim.pdf", 640_000, "Document"),
  folderFile("Projeler", "Teklif_Q4.xlsx", 84_000, "Document"),
  folderFile("Projeler", "Sunum_Final.pptx", 6_400_000, "Document"),
  folderFile("Fotograflar", "IMG_2041.jpg", 4_200_000, "Image"),
  folderFile("Fotograflar", "IMG_2042.jpg", 3_900_000, "Image"),
  folderFile("Arsiv", "yedek_2026.zip", 58_000_000, "Archive"),
  folderFile("Notlar", "toplanti.txt", 3_200, "Document"),
];

const stat = (name: string, d: number, i: number, a: number, o: number, n: number) => ({ name, path: `${ROOT}\\${name}`, totalBytes: d + i + a + o, fileCount: n, documentBytes: d, imageBytes: i, archiveBytes: a, otherBytes: o, files: folderFiles.filter(f => f.path.includes(`\\${name}\\`)) });

export const folderStats = (rootPath: string): FolderStatsResult => {
  const folders = [stat("Fotograflar", 0, 1_820_000_000, 0, 12_000_000, 612), stat("Arsiv", 4_000_000, 0, 940_000_000, 0, 38), stat("Projeler", 210_000_000, 46_000_000, 18_000_000, 9_000_000, 244), stat("Faturalar", 38_000_000, 0, 0, 0, 96), stat("Notlar", 2_400_000, 0, 0, 300_000, 120)];
  const sum = (k: "documentBytes" | "imageBytes" | "archiveBytes" | "otherBytes") => folders.reduce((s, f) => s + f[k], 0);
  return { rootPath, rootTotalBytes: folders.reduce((s, f) => s + f.totalBytes, 0), rootFileCount: 1284, folders, categoryBytes: [sum("documentBytes"), sum("imageBytes"), sum("archiveBytes"), sum("otherBytes")], warnings: [] };
};

export const history: UndoRecord[] = [
  { id: "H-3", timestamp: (now - 3600) * 1000, tag: "Faturalar", moves: [{ from: `${ROOT}\\Indirilenler\\Su_Ekim.pdf`, to: `${ROOT}\\Faturalar\\Su_Ekim.pdf` }] },
  { id: "H-2", timestamp: (now - day) * 1000, tag: "Türüne göre düzenleme", moves: [{ from: `${ROOT}\\a.jpg`, to: `${ROOT}\\Resimler\\JPG\\a.jpg` }, { from: `${ROOT}\\b.pdf`, to: `${ROOT}\\Dokümanlar\\PDF\\b.pdf` }] },
];

export const health = (rootPath: string): HealthReport => ({
  rootPath, totalFiles: 1284, totalBytes: 3_105_700_000,
  largest: [{ path: `${ROOT}\\Arsiv\\yedek_2026.zip`, size: 58_000_000, mtime: now - 20 * day }, { path: `${ROOT}\\Projeler\\Sunum_Final.pptx`, size: 6_400_000, mtime: now - 8 * day }],
  oldest: [{ path: `${ROOT}\\Notlar\\eski_plan.txt`, size: 2_100, mtime: now - 2100 * day }],
  emptyFiles: [{ path: `${ROOT}\\Notlar\\bos.txt`, size: 0, mtime: now - 90 * day }],
  emptyDirs: [`${ROOT}\\Eski\\Gecici`], warnings: [],
});

export const quarantine = (rootPath: string): QuarantineListing => ({
  quarantinePath: `${rootPath}\\_Kopyalar_Karantina`, totalBytes: 1_240_000,
  items: [{ path: `${rootPath}\\_Kopyalar_Karantina\\Faturalar\\Elektrik_Kasim.pdf`, group: "Faturalar", size: 1_240_000, mtime: now - 2 * day, original: `${ROOT}\\Indirilenler\\Elektrik_Kasim (1).pdf`,
    quarantinedMs: Date.now() - 45 * 86_400_000 }],
});

export const settings: AppSettings = {
  sourceFolders: ["C:\\Kullanicilar\\Ornek\\Indirilenler"], backupRoot: BACKUP, automationEnabled: true, autoMove: true, includeOther: false, scanPendingOnStart: true,
  startWithWindows: false, notifyMoves: true,
  extGroups: [
    { name: "Dokümanlar", extensions: ["pdf", "docx", "xlsx", "pptx", "txt"] }, { name: "Görseller", extensions: ["jpg", "png", "webp", "bmp"] },
    { name: "Videolar", extensions: ["mp4", "mkv", "mov"] }, { name: "Arşivler", extensions: ["zip", "rar", "7z"] }, { name: "Kurulum", extensions: ["exe", "msi"] },
  ],
};

export const inbox = (date: string, withPending = false): InboxListing => {
  const dir = `${BACKUP}\\_GelenKutusu\\${date}_Bakilacaklar`;
  const ms = Date.now();
  const f = (group: string, name: string, size: number, extra: Partial<InboxListing["groups"][number]["files"][number]> = {}) => ({ path: `${dir}\\${group}\\${name}`, name, ext: name.split(".").pop()!, size, group, arrivedMs: ms - 3_600_000, createdMs: ms - day * 1000, modifiedMs: ms - 7_200_000,
    accessedMs: ms - 600_000, contentPreview: null, thumbnailPath: null, similarTo: null, similarity: null, duplicateOf: null, duplicateIn: null,
    suggestion: { verdict: "keep" as const, target: null, reason: "Yeni bir dosya, benzeri yok. Yerinde kalması yeterli." }, pending: false, ...extra });
  const groupsIn = [
    { name: "Dokümanlar", files: [f("Dokümanlar", "toplanti_notlari.txt", 812, { contentPreview: "Toplantı notları: Q4 bütçe görüşmesi ve süreç güncellemesi." }),
      f("Dokümanlar", "Teklif_Q4.xlsx", 22_011, { similarTo: "Teklif_Q4_son.xlsx", similarity: 95, suggestion: { verdict: "move" as const, target: "Projeler", reason: "Projeler klasöründeki Teklif_Q3.xlsx ile adı %89 benzer." } }),
      f("Dokümanlar", "Teklif_Q4_son.xlsx", 23_100, { similarTo: "Teklif_Q4.xlsx", similarity: 95, suggestion: { verdict: "move" as const, target: "Projeler", reason: "Adı Teklif_Q4.xlsx ile benzer, ikisi birlikte gitmeli." } })] },
    { name: "Görseller", files: [f("Görseller", "logo.png", 5_120), f("Görseller", "tarama.bmp", 7_654_321)] },
    { name: "Arşivler", files: [f("Arşivler", "yedek_2026.zip", 120_000, { duplicateOf: "D:\\Yedek\\Arşiv\\yedek_2026.zip", duplicateIn: "Arşiv", suggestion: { verdict: "bin" as const, target: null, reason: "İçeriği Arşiv klasöründeki yedek_2026.zip ile birebir aynı." } })] },
    ...(withPending ? [{ name: "İzlenen · Indirilenler", files: ["teknik_sartname_son.docx", "fatura_kasim.pdf"].map((n, i) =>
      f("x", n, 48_200 + i * 40_000, { path: `${settings.sourceFolders[0]}\\${n}`, group: "İzlenen · Indirilenler", pending: true })) }] : []),
  ].map(g => ({ ...g, bytes: g.files.reduce((s, x) => s + x.size, 0) }));
  const total = groupsIn.reduce((n, g) => n + g.files.length, 0);
  return { date, dir, exists: true, days: [{ date, files: total }], groups: groupsIn, totalFiles: total, totalBytes: groupsIn.reduce((s, g) => s + g.bytes, 0), truncated: false };
};

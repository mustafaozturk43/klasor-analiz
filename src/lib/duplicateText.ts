import type { FileDetail } from "../types/inbox";

const baseName = (p: string) => p.split(/[\\/]/).pop() ?? p;

/** Birebir kopya rozetinin metni: yedek klasorunde ya da gelen kutusunda baska bir dosya ile ayni icerik. */
export function duplicateText(f: FileDetail): string {
  if (!f.duplicateOf) return "";
  return f.duplicateIn ? `Aynısı ${f.duplicateIn} klasöründe var` : `Birebir kopya: ${baseName(f.duplicateOf)}`;
}

/** Ayrinti panelindeki uzun aciklama. */
export function duplicateNote(f: FileDetail): string {
  if (!f.duplicateOf) return "";
  return f.duplicateIn
    ? `İçeriği ${f.duplicateIn} klasöründeki ${baseName(f.duplicateOf)} ile birebir aynı. Bu kopyayı Geri Dönüşüm Kutusu’na atabilirsin.`
    : `İçeriği ${baseName(f.duplicateOf)} ile birebir aynı. Bu kopyayı Geri Dönüşüm Kutusu’na atabilirsin.`;
}

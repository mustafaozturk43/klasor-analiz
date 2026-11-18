/** Komut paleti (Ctrl+K): Turkce harfe duyarsiz arama ve siralama. Saf fonksiyonlar. */

export interface PaletteItem {
  id: string;
  /** Satirda gorunen ad. */
  label: string;
  group: "Ekran" | "Klasör" | "İşlem";
  /** Soluk ikinci satir (ornek: tam klasor yolu). */
  hint?: string;
  /** Aramada eslesen ama gorunmeyen ek sozcukler. */
  keywords?: string;
  /** true ise listelenir ama secilemez. */
  disabled?: boolean;
  run: () => void;
}

/** Kucuk harfe cevirir; ı/İ ve aksan farklarini (ç ğ ö ş ü) yok sayar: "Ayarlar" ~ "ayarlar", "Gecmis" ~ "Geçmiş". */
export function fold(text: string): string {
  return text.toLocaleLowerCase("tr").replace(/ı/g, "i").normalize("NFD").replace(/[̀-ͯ]/g, "");
}

/** Her sorgu sozcugu ad + anahtar sozcuklerde gecmeli; ada yakin eslesme (basta, sozcuk basinda) one gecer. */
export function filterItems<T extends PaletteItem>(items: T[], query: string, limit = 50): T[] {
  const tokens = fold(query).split(/\s+/).filter(Boolean);
  if (!tokens.length) return items.slice(0, limit);
  const scored: { item: T; score: number; order: number }[] = [];
  items.forEach((item, order) => {
    const label = fold(item.label);
    const haystack = `${label} ${fold(item.keywords ?? "")} ${fold(item.hint ?? "")}`;
    let score = 0;
    for (const t of tokens) {
      if (!haystack.includes(t)) return;
      score += label.startsWith(t) ? 0 : label.split(/[\s›/\\]+/).some(w => w.startsWith(t)) ? 1 : label.includes(t) ? 2 : 3;
    }
    scored.push({ item, score, order });
  });
  return scored.sort((a, b) => a.score - b.score || a.order - b.order).slice(0, limit).map(s => s.item);
}

/** Ok tuslariyla gezinirken devre disi satirlari atlar; yon: +1 asagi, -1 yukari. Dongusel. */
export function nextIndex(items: PaletteItem[], from: number, dir: 1 | -1): number {
  if (!items.length) return -1;
  for (let step = 1; step <= items.length; step++) {
    const i = (from + dir * step + items.length * step) % items.length;
    if (!items[i].disabled) return i;
  }
  return -1;
}

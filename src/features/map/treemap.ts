/** Kutu haritası yerleşimi (squarified treemap): değerle orantılı alanlar, kutular mümkün olduğunca kareye yakın. */
export interface Box { x: number; y: number; w: number; h: number }
export type Placed<T> = T & Box;

interface Node<T> { item: T; area: number }

/** Değeri sıfır olanlar atlanır. `box` içindeki alan toplamı, değerlerle orantılı bölünür. */
export function squarify<T extends { value: number }>(items: T[], box: Box): Placed<T>[] {
  const sorted = items.filter(i => i.value > 0).sort((a, b) => b.value - a.value);
  const total = sorted.reduce((s, i) => s + i.value, 0);
  if (!total) return [];
  const scale = (box.w * box.h) / total;
  const nodes: Node<T>[] = sorted.map(item => ({ item, area: item.value * scale }));
  const out: Placed<T>[] = [];
  let { x, y, w, h } = box;

  const worst = (row: Node<T>[], side: number) => {
    const sum = row.reduce((s, n) => s + n.area, 0);
    const areas = row.map(n => n.area);
    return Math.max((side * side * Math.max(...areas)) / (sum * sum), (sum * sum) / (side * side * Math.min(...areas)));
  };
  const place = (row: Node<T>[]) => {
    const sum = row.reduce((s, n) => s + n.area, 0);
    if (w >= h) {
      const colW = sum / h;
      let cy = y;
      for (const n of row) { const nh = n.area / colW; out.push({ ...n.item, x, y: cy, w: colW, h: nh }); cy += nh; }
      x += colW; w -= colW;
    } else {
      const rowH = sum / w;
      let cx = x;
      for (const n of row) { const nw = n.area / rowH; out.push({ ...n.item, x: cx, y, w: nw, h: rowH }); cx += nw; }
      y += rowH; h -= rowH;
    }
  };

  let row: Node<T>[] = [];
  for (const node of nodes) {
    const side = Math.min(w, h);
    if (!row.length || worst([...row, node], side) <= worst(row, side)) row.push(node);
    else { place(row); row = [node]; }
  }
  if (row.length) place(row);
  return out;
}

import { ChevronLeft, ChevronRight } from "lucide-react";

interface Props {
  page: number; pages: number; size: number; total: number;
  setPage: (page: number) => void; setSize: (size: number) => void;
}
export default function Pagination({ page, pages, size, total, setPage, setSize }: Props) {
  return <div className="pagination" aria-label="Sayfalama">
    <span>{total ? (page - 1) * size + 1 : 0} ile {Math.min(page * size, total)} arası / {total.toLocaleString("tr-TR")} kayıt</span>
    <div className="inline-actions">
      <button className="btn" aria-label="Önceki sayfa" disabled={page <= 1} onClick={() => setPage(page - 1)}><ChevronLeft size={16} /></button>
      <span aria-live="polite">{page} / {pages}</span>
      <button className="btn" aria-label="Sonraki sayfa" disabled={page >= pages} onClick={() => setPage(page + 1)}><ChevronRight size={16} /></button>
    </div>
    <select aria-label="Sayfa başına kayıt" value={size} onChange={e => setSize(Number(e.target.value))}>
      {[25, 50, 100].map(n => <option key={n} value={n}>{n} / sayfa</option>)}
    </select>
  </div>;
}

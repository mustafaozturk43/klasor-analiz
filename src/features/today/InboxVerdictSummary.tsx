import type { ReactNode } from "react";
import { ArrowRight, Check, Trash2 } from "lucide-react";
import { formatCount } from "../../lib/format";
import type { InboxState } from "./useInbox";

interface Props { inbox: InboxState; dateLabel: string; actions?: ReactNode }

/** Baslik ve karar ozeti: kac dosya bekliyor, kacinin kalacagi, tasinacagi ve atilacagi. */
export default function InboxVerdictSummary({ inbox, dateLabel, actions }: Props) {
  const { keep, move, bin } = inbox.counts;
  const total = keep + move + bin;
  const items = [
    { id: "keep", n: keep, Icon: Check, text: "yerinde kalır" },
    { id: "move", n: move, Icon: ArrowRight, text: "klasörüne taşınır" },
    { id: "bin", n: bin, Icon: Trash2, text: "Geri Dönüşüm Kutusu’na" },
  ] as const;
  return <section className="inbox-summary" aria-label="Karar özeti">
    <div className="inbox-summary-head">
      <div>
        <h1>{formatCount(total)} dosya karar bekliyor</h1>
        <p>{dateLabel}. Önerileri değiştir, sonra hepsini tek dokunuşla uygula.</p>
      </div>
      {actions}
    </div>
    <div className="verdict-bar" aria-hidden="true">
      {items.map(i => <span key={i.id} data-verdict={i.id} style={{ flexGrow: i.n }} />)}
    </div>
    <div className="verdict-counts">
      {items.map(({ id, n, Icon, text }) => <div key={id} data-verdict={id}>
        <span className="verdict-icon"><Icon size={22} aria-hidden="true" /></span>
        <strong>{formatCount(n)}</strong><span>{text}</span>
      </div>)}
    </div>
  </section>;
}

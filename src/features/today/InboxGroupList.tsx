import { useCallback, useMemo } from "react";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "@/components/ui/accordion";
import { formatBytes, formatCount } from "../../lib/format";
import type { Verdict } from "./useInbox";
import type { FileDetail, InboxGroup } from "../../types/inbox";
import InboxFileRow from "./InboxFileRow";
import VerdictToggle from "./VerdictToggle";
import VirtualRows from "../../components/VirtualRows";

export const SIMILAR_GROUP = "Benzer isimliler";

interface Props {
  groups: InboxGroup[];
  verdicts: Record<string, "move" | "bin">;
  setVerdict: (paths: string[], v: Verdict) => void;
  onReveal: (path: string) => void;
  selected: string | null;
  onSelect: (path: string) => void;
  /** "Tasi" kararindaki dosyanin gidecegi klasorun adi (yoksa null). */
  targetNameOf: (path: string) => string | null;
}

/** Bir grubun dosyalari ayni kararda mi? Karisiksa `null`. */
function groupVerdict(paths: string[], verdicts: Props["verdicts"]): Verdict | null {
  const first = verdicts[paths[0]] ?? "keep";
  return paths.every(p => (verdicts[p] ?? "keep") === first) ? first : null;
}

/** Gruplar akordeonu: her grubun tek hamlede karar veren dugmeleri ve sanallastirilmis satirlari. */
export default function InboxGroupList({ groups, verdicts, setVerdict, onReveal, selected, onSelect, targetNameOf }: Props) {
  // Benzer adli dosyalar kendi gruplarinda da kalir; burada ayrica bir arada gorunur.
  const all = useMemo(() => {
    const similar = groups.flatMap(g => g.files).filter(f => f.similarTo);
    return similar.length ? [...groups, { name: SIMILAR_GROUP, bytes: similar.reduce((s, f) => s + f.size, 0), files: similar }] : groups;
  }, [groups]);
  const onVerdict = useCallback((path: string, v: Verdict) => setVerdict([path], v), [setVerdict]);
  const open = useMemo(() => all.filter(g => g.name !== SIMILAR_GROUP).map(g => g.name), [all]);

  return <Accordion type="multiple" defaultValue={open} className="flex flex-col gap-3">
    {all.map(g => {
      const paths = g.files.map(f => f.path);
      return <AccordionItem key={g.name} value={g.name}>
        <AccordionTrigger before={<VerdictToggle compact value={groupVerdict(paths, verdicts)} onChange={v => setVerdict(paths, v)} label={`${g.name} grubundaki tüm dosyaların kararı`} />}>
          {g.name}
          <span className="ml-auto text-[13px] font-medium tabular-nums text-[var(--color-muted)]">
            {formatCount(g.files.length)} dosya, {formatBytes(g.bytes)}
            {g.name === SIMILAR_GROUP ? ", içerik farklı olabilir" : ""}
          </span>
        </AccordionTrigger>
        <AccordionContent>
          <VirtualRows<FileDetail> items={g.files} estimate={84} maxHeight={520} itemKey={f => f.path}
            render={f => <InboxFileRow file={f} verdict={verdicts[f.path] ?? "keep"} onVerdict={onVerdict} onReveal={onReveal} selected={selected === f.path} onSelect={onSelect} target={verdicts[f.path] === "move" ? targetNameOf(f.path) : null} />} />
        </AccordionContent>
      </AccordionItem>;
    })}
  </Accordion>;
}

import { useEffect, useState, type ReactNode } from "react";
import { ArrowRight, PenLine } from "lucide-react";
import { Dialog, DialogClose, DialogContent } from "@/components/ui/dialog";
import type { RenamePlan } from "../lib/fileOperations";
import type { ListRenameOptions } from "../lib/inboxCommands";
import { baseName, formatCount } from "../lib/format";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  count: number;
  busy: boolean;
  tokens: string[];
  defaultTemplate: string;
  defaultPad?: number;
  description: string;
  hint?: ReactNode;
  /** Kararli (useCallback) olmali: degisince on izleme yenilenir. */
  preview: (options: ListRenameOptions) => Promise<RenamePlan>;
  apply: (plan: RenamePlan) => Promise<void>;
  /** Ikinci eylem (ör. "Adlandır, sonra taşı…"). */
  secondary?: { label: string; disabled?: boolean; title?: string; run: (plan: RenamePlan) => Promise<void> };
}

/** Sablonla adlandirma: yazdikca sunucu tarafli on izleme (eski → yeni), sonra uygula. Gelen kutusu ve Klasordeki dosyalar kullanir. */
export default function TemplateRenameDialog({ open, onOpenChange, count, busy, tokens, defaultTemplate, defaultPad = 2, description, hint, preview, apply, secondary }: Props) {
  const [template, setTemplate] = useState(defaultTemplate);
  const [customText, setCustomText] = useState("");
  const [start, setStart] = useState(1);
  const [pad, setPad] = useState(defaultPad);
  const [plan, setPlan] = useState<RenamePlan | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open || !count) return;
    let alive = true;
    const id = window.setTimeout(() => {
      preview({ template, customText, start, pad })
        .then(p => { if (alive) { setPlan(p); setError(null); } })
        .catch(e => { if (alive) { setPlan(null); setError(String(e)); } });
    }, 250);
    return () => { alive = false; window.clearTimeout(id); };
  }, [open, count, template, customText, start, pad, preview]);

  const ready = plan?.entries.filter(e => e.to).length ?? 0;
  const run = async (fn: (p: RenamePlan) => Promise<void>) => {
    if (!plan) return;
    await fn(plan);
    onOpenChange(false);
  };

  return <Dialog open={open} onOpenChange={onOpenChange}>
    <DialogContent title={`${formatCount(count)} dosyayı adlandır`} description={description} className="max-w-[620px]">
      <label className="flex flex-col gap-1 text-[12px] font-semibold text-[var(--color-ink-2)]">Şablon
        <input id="rename-template" className="settings-path font-mono" value={template} spellCheck={false} onChange={e => setTemplate(e.target.value)} />
      </label>
      <div className="flex flex-wrap gap-1.5">{tokens.map(t => <button key={t} type="button" className={`inbox-token ${template.includes(t) ? "on" : ""}`} onClick={() => setTemplate(v => `${v}${t}`)}>{t}</button>)}</div>
      <div className="grid grid-cols-[minmax(0,1fr)_90px_90px] gap-2 max-[520px]:grid-cols-2">
        <label className="flex flex-col gap-1 text-[12px] text-[var(--color-muted)] max-[520px]:col-span-2">Özel metin
          <input id="rename-custom" className="settings-group-name font-normal" value={customText} onChange={e => setCustomText(e.target.value)} placeholder="ör. Kasim_Faturalari" />
        </label>
        <label className="flex flex-col gap-1 text-[12px] text-[var(--color-muted)]">Başlangıç
          <input id="rename-start" type="number" min={0} className="settings-group-name font-normal" value={start} onChange={e => setStart(Math.max(0, Number(e.target.value) || 0))} />
        </label>
        <label className="flex flex-col gap-1 text-[12px] text-[var(--color-muted)]">Basamak
          <select id="rename-pad" value={pad} onChange={e => setPad(Number(e.target.value))}>{[1, 2, 3, 4].map(n => <option key={n} value={n}>{"0".repeat(n - 1)}1</option>)}</select>
        </label>
      </div>
      {hint && <p className="hint">{hint}</p>}
      {error && <p className="settings-error" role="alert">{error}</p>}
      {plan && <div className="inbox-preview" aria-live="polite">
        {plan.entries.map(e => <div key={e.from}>
          <span className="truncate">{baseName(e.from)}</span>
          <ArrowRight size={13} className="text-[var(--color-muted)]" />
          <span className={`truncate ${e.to ? "font-medium text-[var(--color-ink-2)]" : "text-[var(--color-muted)]"}`}>{e.to ? baseName(e.to) : "atlanacak"}</span>
          {e.warning && <small className="col-span-3 text-[var(--color-warn-ink)]">{e.warning}</small>}
        </div>)}
      </div>}
      <div className="flex flex-wrap justify-end gap-2">
        <DialogClose className="btn">Vazgeç</DialogClose>
        <button type="button" className={`btn${secondary ? "" : " primary"}`} disabled={!ready || busy} onClick={() => void run(apply)}><PenLine size={14} /> {formatCount(ready)} dosyayı adlandır</button>
        {secondary && <button type="button" className="btn primary" disabled={!ready || busy || secondary.disabled} title={secondary.title} onClick={() => void run(secondary.run)}>{secondary.label}</button>}
      </div>
    </DialogContent>
  </Dialog>;
}

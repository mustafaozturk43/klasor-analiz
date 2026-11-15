import { memo } from "react";
import { ArrowRight, Check, Trash2 } from "lucide-react";
import type { Verdict } from "../hooks/useInbox";

const OPTIONS: { id: Verdict; label: string; Icon: typeof Check }[] = [
  { id: "keep", label: "Tut", Icon: Check },
  { id: "move", label: "Taşı", Icon: ArrowRight },
  { id: "bin", label: "At", Icon: Trash2 },
];

interface Props {
  /** Secili karar; grup satirinda karisiksa `null`. */
  value: Verdict | null;
  onChange: (v: Verdict) => void;
  label: string;
  compact?: boolean;
}

/** Tut / Tasi / At: renk, yazi ve simge birlikte tasir; renk tek basina anlam yuklemez. */
function VerdictToggle({ value, onChange, label, compact }: Props) {
  return <div role="group" aria-label={label} className={`verdict ${compact ? "compact" : ""}`}>
    {OPTIONS.map(({ id, label: text, Icon }) => <button key={id} type="button" data-verdict={id} aria-pressed={value === id} onClick={() => onChange(id)}>
      {compact && <Icon size={13} aria-hidden="true" />}{text}
    </button>)}
  </div>;
}

export default memo(VerdictToggle);

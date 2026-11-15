interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  /** Erisilebilirlik icin etiket. */
  label?: string;
}

/** Ac-kapa anahtari: kapaliyken cizgili, aciyken dolu; renkler tokens.css'ten (`.switch`, rules.css). */
export default function Switch({ checked, onChange, label }: SwitchProps) {
  return <button type="button" role="switch" aria-checked={checked} aria-label={label} className="switch" onClick={() => onChange(!checked)}><span /></button>;
}

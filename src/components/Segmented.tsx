import type { LucideIcon } from "lucide-react";

interface SegmentedProps<T extends string> {
  value: T;
  options: { value: T; label: string; icon?: LucideIcon }[];
  onChange: (value: T) => void;
  /** Grubu adlandiran etiketin id'si. */
  labelledBy?: string;
}

/** Bolunmus kontrol (segmented control); renkler menus.css'teki `.segmented` token'larindan. */
export default function Segmented<T extends string>({
  value,
  options,
  onChange,
  labelledBy,
}: SegmentedProps<T>) {
  return (
    <div className="segmented" role="group" aria-labelledby={labelledBy}>
      {options.map(({ value: v, label, icon: Icon }) => (
        <button key={v} type="button" aria-pressed={v === value} onClick={() => onChange(v)}>
          {Icon && <Icon size={14} aria-hidden />}
          {label}
        </button>
      ))}
    </div>
  );
}

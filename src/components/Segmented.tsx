import type { LucideIcon } from "lucide-react";

interface SegmentedProps<T extends string> {
  value: T;
  options: { value: T; label: string; icon?: LucideIcon }[];
  onChange: (value: T) => void;
}

/** Apple HIG tarzi bolunmus kontrol (segmented control). */
export default function Segmented<T extends string>({
  value,
  options,
  onChange,
}: SegmentedProps<T>) {
  return (
    <div className="inline-flex items-center gap-1 rounded-full bg-zinc-200/80 p-1 dark:bg-zinc-800/80">
      {options.map(({ value: v, label, icon: Icon }) => {
        const active = v === value;
        return (
          <button
            key={v}
            type="button"
            onClick={() => onChange(v)}
            className={`flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-medium transition-colors ${
              active
                ? "bg-white text-zinc-900 shadow-sm dark:bg-zinc-600 dark:text-zinc-50"
                : "text-zinc-600 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-100"
            }`}
          >
            {Icon && <Icon className="h-3.5 w-3.5" />}
            {label}
          </button>
        );
      })}
    </div>
  );
}

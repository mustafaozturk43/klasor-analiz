import { useRef, type ReactNode } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

export default function VirtualRows<T>({ items, itemKey, render, estimate = 78, maxHeight = 520 }: {
  items: T[]; itemKey: (item: T) => string; render: (item: T, index: number) => ReactNode;
  estimate?: number; maxHeight?: number;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const virtual = useVirtualizer({ count: items.length, getScrollElement: () => ref.current,
    getItemKey: i => itemKey(items[i]), estimateSize: () => estimate, overscan: 4 });
  return <div ref={ref} className="overflow-y-auto" style={{ height: Math.min(items.length * estimate, maxHeight) }}>
    <div style={{ height: virtual.getTotalSize(), position: "relative" }}>
      {virtual.getVirtualItems().map(item => <div key={item.key} ref={virtual.measureElement} data-index={item.index}
        style={{ position: "absolute", top: 0, left: 0, width: "100%", transform: `translateY(${item.start}px)` }}>
        {render(items[item.index], item.index)}
      </div>)}
    </div>
  </div>;
}

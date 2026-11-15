import * as PopoverPrimitive from "@radix-ui/react-popover";
import type { ComponentProps } from "react";
import { cn } from "@/lib/utils";

/** shadcn Popover; renkler workspace.css token'larindan. */
export const Popover = PopoverPrimitive.Root;
export const PopoverTrigger = PopoverPrimitive.Trigger;
export const PopoverAnchor = PopoverPrimitive.Anchor;

export function PopoverContent({ className, align = "start", sideOffset = 6, ...props }: ComponentProps<typeof PopoverPrimitive.Content>) {
  return <PopoverPrimitive.Portal>
    <PopoverPrimitive.Content align={align} sideOffset={sideOffset}
      className={cn("z-50 rounded-[10px] border border-[var(--color-rule)] bg-[var(--color-paper-2)] p-1.5 text-[12px] text-[var(--color-ink)] shadow-[var(--shadow-raised)] outline-none", className)}
      {...props} />
  </PopoverPrimitive.Portal>;
}

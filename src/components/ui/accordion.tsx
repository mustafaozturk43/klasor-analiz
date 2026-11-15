import * as AccordionPrimitive from "@radix-ui/react-accordion";
import { ChevronDown } from "lucide-react";
import type { ComponentProps, ReactNode } from "react";
import { cn } from "@/lib/utils";

/** shadcn Accordion; renkler workspace.css token'larindan. */
export const Accordion = AccordionPrimitive.Root;

export function AccordionItem({ className, ...props }: ComponentProps<typeof AccordionPrimitive.Item>) {
  return <AccordionPrimitive.Item className={cn("overflow-hidden rounded-[10px] border border-[var(--color-rule)] bg-[var(--color-paper-2)]", className)} {...props} />;
}

/** Baslik: `before` (orn. grup checkbox'i) tetikleyicinin disinda tutulur ki tiklamasi akordeonu acip kapatmasin. */
export function AccordionTrigger({ className, children, before, ...props }: ComponentProps<typeof AccordionPrimitive.Trigger> & { before?: ReactNode }) {
  return <AccordionPrimitive.Header className="flex items-center gap-2.5 bg-[var(--color-paper)] pl-3 data-[state=open]:border-b data-[state=open]:border-[var(--color-rule)]">
    {before}
    <AccordionPrimitive.Trigger
      className={cn("group flex flex-1 items-center gap-2 py-2.5 pr-3 text-left text-[12.5px] font-semibold text-[var(--color-ink-2)] outline-none focus-visible:underline", className)}
      {...props}>
      {children}
      <ChevronDown size={15} className="ml-1 shrink-0 text-[var(--color-muted)] transition-transform group-data-[state=open]:rotate-180" />
    </AccordionPrimitive.Trigger>
  </AccordionPrimitive.Header>;
}

export function AccordionContent({ className, ...props }: ComponentProps<typeof AccordionPrimitive.Content>) {
  return <AccordionPrimitive.Content className={cn("overflow-hidden", className)} {...props} />;
}

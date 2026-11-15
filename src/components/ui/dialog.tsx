import * as DialogPrimitive from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import type { ComponentProps, ReactNode } from "react";
import { cn } from "@/lib/utils";

/** shadcn Dialog; renkler workspace.css token'larindan. Esc ve dis tiklama kapatir. */
export const Dialog = DialogPrimitive.Root;
export const DialogClose = DialogPrimitive.Close;

export function DialogContent({ className, children, title, description, ...props }: ComponentProps<typeof DialogPrimitive.Content> & { title: string; description?: ReactNode }) {
  return <DialogPrimitive.Portal>
    <DialogPrimitive.Overlay className="fixed inset-0 z-50 bg-[var(--color-scrim)]" />
    <DialogPrimitive.Content
      className={cn("fixed left-1/2 top-1/2 z-50 flex max-h-[88vh] w-[calc(100vw-32px)] max-w-[520px] -translate-x-1/2 -translate-y-1/2 flex-col gap-3 overflow-y-auto",
        "rounded-[var(--radius-card)] border border-[var(--color-rule)] bg-[var(--color-paper-2)] p-5 text-[13px] text-[var(--color-ink)] shadow-[var(--shadow-overlay)] outline-none", className)}
      {...props}>
      <div className="flex items-start gap-3">
        <div className="min-w-0 flex-1">
          <DialogPrimitive.Title className="m-0 text-[15px] font-semibold text-[var(--color-ink-2)]">{title}</DialogPrimitive.Title>
          {description ? <DialogPrimitive.Description className="mt-1 text-[12px] text-[var(--color-muted)]">{description}</DialogPrimitive.Description>
            : <DialogPrimitive.Description className="sr-only">{title}</DialogPrimitive.Description>}
        </div>
        <DialogPrimitive.Close className="grid size-7 place-items-center rounded-md text-[var(--color-muted)] hover:bg-[var(--color-paper)]" aria-label="Kapat"><X size={15} /></DialogPrimitive.Close>
      </div>
      {children}
    </DialogPrimitive.Content>
  </DialogPrimitive.Portal>;
}

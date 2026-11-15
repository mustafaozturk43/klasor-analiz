import type { OrganizerState } from "../types/app";
import InboxStatusPanel from "./InboxStatusPanel";
import TemplateRenamePanel from "./TemplateRenamePanel";

/** Otomasyon sekmesi: gelen kutusu izleme durumu ve sablonla toplu adlandirma. */
export default function AutomationView({ app }: { app: OrganizerState }) {
  return <div className="organize-stack">
    <InboxStatusPanel app={app} />
    <TemplateRenamePanel app={app} />
  </div>;
}

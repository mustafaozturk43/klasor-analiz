import type { OrganizerState } from "../../types/app";
import { useNaming } from "./useNaming";
import { NAMING_DEFAULTS } from "./namingCommands";
import NamingOptions from "./NamingOptions";
import NamingPreview from "./NamingPreview";

/** Adlandirma ekrani: solda kurallar, sagda canli onizleme. Onaylanmadan hicbir ad degismez. */
export default function NamingView({ app, active }: { app: OrganizerState; active: boolean }) {
  const naming = useNaming(app, active);
  const locked = app.isScanning || app.isMoving || naming.busy;
  return <div className="naming-layout">
    <NamingOptions options={naming.options} onChange={naming.setOptions} disabled={locked} />
    <NamingPreview naming={naming} rootPath={app.rootPath} locked={locked} onReset={() => naming.setOptions(NAMING_DEFAULTS)} />
  </div>;
}

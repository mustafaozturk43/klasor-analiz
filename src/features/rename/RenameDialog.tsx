import { useEffect, useRef, useState } from "react";
import { applyRenamePlan } from "../rules/automationCommands";
import type { RenamePlan } from "../../lib/fileOperations";
import { baseName } from "../../lib/format";
import VirtualRows from "../../components/VirtualRows";

export default function RenameDialog({ plan, onClose, onApplied }: {
  plan: RenamePlan; onClose: () => void; onApplied: () => Promise<void>;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  useEffect(() => { ref.current?.showModal(); }, []);
  async function apply() {
    setBusy(true); setError("");
    try { await applyRenamePlan(plan.id); await onApplied(); onClose(); }
    catch (e) { setError(String(e)); await onApplied(); }
    finally { setBusy(false); }
  }
  return <dialog ref={ref} className="modal-card m-auto p-0 text-inherit" aria-labelledby="rename-title" onCancel={e => { e.preventDefault(); if (!busy) onClose(); }}>
    <div className="panel-head"><div><h2 id="rename-title">Tarihle isimlendirme ön izlemesi</h2><p>Oluşturulma tarihi (yerel saat) kullanılır. Tarihi bulunamayanlar atlanır; uzantılar korunur.</p></div></div>
    <VirtualRows items={plan.entries} itemKey={e => e.from} estimate={90} render={entry => <div className="data-row"><div className="file-description">
      <small title={entry.from}>{baseName(entry.from)}</small><strong className="break-all">{entry.to ? baseName(entry.to) : "Atlanacak"}</strong>
      <small>{entry.warning || "Hedef çakışmaları numaralandırılır. İşlem geri alınabilir."}</small>
    </div></div>} />
    {error && <div role="alert" className="error-state">{error}</div>}
    <div className="panel-head"><button className="btn" disabled={busy} onClick={onClose}>Kapat</button>
      <button className="btn primary" disabled={busy || !!error || !plan.entries.some(e => e.to)} onClick={() => void apply()}>{busy ? "İşleniyor…" : "Onayla ve isimlendir"}</button></div>
  </dialog>;
}

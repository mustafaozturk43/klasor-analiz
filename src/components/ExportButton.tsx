import { Download } from "lucide-react";
import { formatCount } from "../lib/format";
import { exportTableCsv, type TableRows } from "../lib/tableExport";
import type { OrganizerState } from "../types/app";

interface Props {
  app: OrganizerState;
  /** Kaydet diyalogundaki varsayilan dosya adi. */
  fileName: string;
  header: string[];
  /** Tikaninca cagrilir; satirlar o ana kadarki ekran verisinden uretilir. */
  rows: () => TableRows;
  disabled?: boolean;
}

/** "CSV'ye aktar": kaydet diyalogu acar, Excel uyumlu CSV yazar ve sonucu bildirir. */
export default function ExportButton({ app, fileName, header, rows, disabled }: Props) {
  const run = async () => {
    try {
      const out = await exportTableCsv(fileName, header, rows());
      if (out) app.setStatus({ type: "success", text: `Rapor kaydedildi · ${formatCount(out.rows)} satır → ${out.path}` });
    } catch (e) {
      app.setStatus({ type: "error", text: String(e) });
    }
  };
  return <button type="button" className="btn" onClick={() => void run()} disabled={disabled} title="Excel uyumlu CSV dosyası kaydeder"><Download size={14} /> CSV'ye aktar</button>;
}

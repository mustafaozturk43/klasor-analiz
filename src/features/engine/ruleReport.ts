/** Kural motoru onizleme planini tablo satirlarina cevirir (CSV disa aktarma): hangi dosya nereye gidecek. */
import type { TableRows } from "../../lib/tableExport";
import type { RulePlan } from "./ruleEngineCommands";
import { ACTION_LABELS } from "./ruleModel";

export const RULE_HEADER = ["Kural", "Eylem", "Kaynak", "Hedef"];

/** Geri Donusum Kutusu eyleminde hedef bos kalir. */
export const ruleRows = (plan: RulePlan): TableRows => plan.entries.map(e => [e.rule, ACTION_LABELS[e.action], e.from, e.to]);

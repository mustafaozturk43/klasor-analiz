import { memo } from "react";
import type { FileGroup } from "../../types/file";
import { CATEGORY_LABELS, formatBytes } from "../../lib/format";

interface Props { group: FileGroup; active: boolean; selected: boolean; onPick: () => void; onToggle: () => void }

/** Sol listedeki grup kartı: ad, boyut, tür etiketi. Kutu toplu seçim, kartın kendisi karşılaştırmayı açar. */
function GroupListItem({ group, active, selected, onPick, onToggle }: Props) {
  const exact = group.groupType === "Exact";
  const first = group.files[0];
  return <li className="dup-item" data-active={active || undefined} data-selected={selected || undefined}>
    <input type="checkbox" aria-label={`${group.id} grubunu toplu taşıma için seç`} checked={selected} onChange={onToggle} />
    <button type="button" aria-pressed={active} onClick={onPick}>
      <span className="dup-item-top"><b>{first?.name ?? group.id}</b><span>{exact ? formatBytes(first?.size ?? 0) : "içerik farklı"}</span></span>
      <span className="dup-item-sub"><span className={`dup-tag ${exact ? "exact" : "similar"}`}>{exact ? "Birebir" : "Benzer ad"}</span>{group.files.length} dosya, {CATEGORY_LABELS[group.category]}</span>
    </button>
  </li>;
}

export default memo(GroupListItem);

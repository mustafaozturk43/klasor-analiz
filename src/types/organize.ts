/** Rust `organize_models.rs` ile birebir eslesen tipler. */
import type { KeepPolicy } from "./file";

export type OrganizeAction = "Move" | "InPlace" | "Quarantine" | "Skip";

export interface OrganizeEntry {
  from: string;
  to: string | null;
  action: OrganizeAction;
  category: string | null;
  kind: string | null;
  size: number;
  mtime: number;
  renamed: boolean;
  similarTo: string | null;
  note: string | null;
}

export interface KindCount {
  category: string;
  kind: string;
  count: number;
  bytes: number;
}

export interface OrganizePlan {
  id: string;
  rootPath: string;
  totalFiles: number;
  entries: OrganizeEntry[];
  kinds: KindCount[];
  protectedDirs: string[];
  warnings: string[];
  removeEmptyDirs: boolean;
}

export interface RuleDef {
  category: string;
  kind: string;
  extensions: string[];
}

export interface OrganizeOptions {
  rules: RuleDef[];
  byYear: boolean;
  removeEmptyDirs: boolean;
  includeSubdirs: boolean;
  quarantineDuplicates: boolean;
  protectProjects: boolean;
  includeOther: boolean;
  flat: boolean;
  keepPolicy: KeepPolicy;
  excludeFolders: string[];
  bannedExtensions: string[];
  similarityThreshold: number;
}

export interface OrganizeOutcome {
  moved: number;
  movedBytes: number;
  quarantined: number;
  quarantinedBytes: number;
  failed: { path: string; error: string }[];
  removedDirs: number;
}

/** Kullanicinin degistirebildigi secenekler (geri kalani uygulama durumundan gelir). */
export type OrganizeToggles = Pick<OrganizeOptions,
  "quarantineDuplicates" | "protectProjects" | "flat" | "byYear" | "removeEmptyDirs">;

/** Adlandirilmis, kaydedilmis duzen: kurallar + secenekler. */
export interface OrganizeProfile {
  name: string;
  rules: RuleDef[];
  toggles: OrganizeToggles;
}

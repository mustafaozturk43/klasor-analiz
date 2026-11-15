/** Rust `health.rs` ve `quarantine_manage.rs` ile eslesen tipler. */

export interface HealthFile {
  path: string;
  size: number;
  /** Son degisiklik (unix saniye). */
  mtime: number;
}

export interface HealthReport {
  rootPath: string;
  totalFiles: number;
  totalBytes: number;
  largest: HealthFile[];
  oldest: HealthFile[];
  emptyFiles: HealthFile[];
  emptyDirs: string[];
  warnings: string[];
}

export interface QuarantineItem {
  path: string;
  group: string;
  size: number;
  mtime: number;
  original: string | null;
  /** Karantinaya alinma zamani (ms); gecmiste kaydi yoksa null. */
  quarantinedMs: number | null;
}

export interface QuarantineListing {
  quarantinePath: string;
  items: QuarantineItem[];
  totalBytes: number;
}

export interface BatchOutcome {
  done: number;
  failed: { path: string; error: string }[];
}

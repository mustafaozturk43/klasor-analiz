import type { useAppState } from "../hooks/useAppState";
export type OrganizerState = ReturnType<typeof useAppState>;
/** Tek bir ekran (eski sekme kimlikleri). */
export type Module = "analysis" | "compare" | "files" | "groups" | "health" | "quarantine" | "naming" | "evening" | "automation" | "settings" | "history";
/** Kenar cubugundaki hedefler; her biri bir veya birkac ekrani toplar (lib/navigation.ts). */
export type Destination = "today" | "duplicates" | "map" | "rename" | "rules" | "history";

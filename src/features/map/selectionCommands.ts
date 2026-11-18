/** Harita > Klasordeki dosyalar: secim, onizleme, tasima ve adlandirma komutlari. */
import { invoke } from "@tauri-apps/api/core";
import type { MoveRecord } from "../../types/file";
import type { RenamePlan } from "../../lib/fileOperations";
import type { ListRenameOptions, MoveOutcome } from "../today/inboxCommands";

export interface SelectionMoveRequest { roots: string[]; paths: string[]; targetBase: string; group: string }
export interface SelectionMovePlan { id: string; targetDir: string; createdDirs: string[]; entries: MoveRecord[]; renamed: number }
export interface SubfolderList { paths: string[]; truncated: boolean }
/** `dates`: `paths` ile ayni sirada cekim tarihleri (unix sn); yoksa null (degisiklik tarihi kullanilir). */
export type SelectionRenameOptions = ListRenameOptions & { group: string; dates: (number | null)[]; tidy: true };

export const listSubfolders = (root: string, excludeFolders: string[]) =>
  invoke<SubfolderList>("list_subfolders", { root, excludeFolders });

/** Her yol icin Windows kucuk resmi (onbellekteki PNG yolu) ya da null. */
export const getShellThumbnails = (paths: string[], size: number) =>
  invoke<(string | null)[]>("get_shell_thumbnails", { paths, size });

/** Her yol icin cekim tarihi (unix sn, UTC) ya da null. */
export const readMediaDates = (paths: string[]) => invoke<(number | null)[]>("read_media_dates", { paths });

export const previewSelectionMove = (request: SelectionMoveRequest) =>
  invoke<SelectionMovePlan>("preview_selection_move", { request });

export const applySelectionMove = (planId: string) => invoke<MoveOutcome>("apply_selection_move", { planId });

/** Onizleme; uygulamak icin `applyRenamePlan(plan.id)` (automationCommands). */
export const previewSelectionRename = (roots: string[], paths: string[], options: SelectionRenameOptions) =>
  invoke<RenamePlan>("preview_selection_rename", { roots, paths, options });

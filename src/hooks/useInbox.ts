import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { applyRenamePlan } from "../lib/automationCommands";
import {
  createTargetFolder, deleteFilesToTrash, getTargetSubfolders, listInbox, moveSelectedFiles, previewInboxRename,
  type FailedItem, type ListRenameOptions, type TargetFolder,
} from "../lib/inboxCommands";
import { formatCount } from "../lib/format";
import { splitGroup } from "../lib/groupPath";
import type { RenamePlan } from "../lib/fileOperations";
import type { OrganizerState } from "../types/app";
import type { InboxListing } from "../types/inbox";

export interface InboxNotice { type: "ok" | "error"; text: string }
/** Her dosyanin karari: yerinde kalir, bir klasore tasinir ya da Geri Donusum Kutusu'na gider. */
export type Verdict = "keep" | "move" | "bin";

/**
 * Aksam Kontrolu durumu: secili gunun listesi, dosya basina karar (varsayilan "Tut"),
 * hedef klasorler ve kararlarin toplu uygulanmasi. Watcher yeni dosya tasidikca
 * (`inbox-updated`) liste yenilenir; kalmayan dosyalarin kararlari silinir.
 */
export function useInbox(app: OrganizerState, active: boolean) {
  const [listing, setListing] = useState<InboxListing | null>(null);
  const [date, setDate] = useState<string | null>(null); // null = bugun
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<InboxNotice | null>(null);
  const [verdicts, setVerdicts] = useState<Record<string, "move" | "bin">>({});
  const [targets, setTargets] = useState<TargetFolder[]>([]);
  const [busy, setBusy] = useState(false);
  const [group, setGroup] = useState(""); // hedefin icinde acilacak alt klasor ("" = dogrudan hedefe)
  const [defaultPath, setDefaultPath] = useState<string | null>(null);
  const seeded = useRef(new Set<string>()); // onerisi bir kez uygulanmis dosyalar: kullanicinin "Tut"u bozulmaz
  const { refreshHistory } = app;

  const load = useCallback(async (day: string | null = date) => {
    setLoading(true);
    try {
      const l = await listInbox(day ?? undefined);
      setListing(l);
      setError(null);
      const files = l.groups.flatMap(g => g.files);
      const present = new Set(files.map(f => f.path));
      const fresh = files.filter(f => !seeded.current.has(f.path));
      fresh.forEach(f => seeded.current.add(f.path));
      setVerdicts(prev => {
        const next: Record<string, "move" | "bin"> = Object.fromEntries(Object.entries(prev).filter(([p]) => present.has(p)));
        for (const f of fresh) if (f.suggestion && f.suggestion.verdict !== "keep" && !(f.path in next)) next[f.path] = f.suggestion.verdict;
        return next;
      });
    } catch (e) {
      setListing(null);
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [date]);

  const loadTargets = useCallback(async () => {
    try { setTargets(await getTargetSubfolders()); } catch { setTargets([]); }
  }, []);

  useEffect(() => { if (active) { void load(); void loadTargets(); } }, [active, load, loadTargets]);

  useEffect(() => {
    if (!active) return;
    const offs = [
      listen("inbox-updated", () => { void load(); }),
      listen("automation-status", () => { void load(); void loadTargets(); }),
    ];
    return () => { for (const off of offs) void off.then(f => f()).catch(() => undefined); };
  }, [active, load, loadTargets]);

  /** Ekrandaki sirayla (gruplar ve satirlar) yollar; adlandirmada sira numarasi bu siradir. */
  const ordered = useMemo(() => (listing?.groups ?? []).flatMap(g => g.files.map(f => f.path)), [listing]);
  const movePaths = useMemo(() => ordered.filter(p => verdicts[p] === "move"), [ordered, verdicts]);
  const binPaths = useMemo(() => ordered.filter(p => verdicts[p] === "bin"), [ordered, verdicts]);
  const counts = useMemo(() => ({ keep: ordered.length - movePaths.length - binPaths.length, move: movePaths.length, bin: binPaths.length }), [ordered, movePaths, binPaths]);

  const defaultTarget = useMemo(() => targets.find(t => t.path === defaultPath) ?? null, [targets, defaultPath]);
  const suggested = useMemo(() => {
    const m: Record<string, string> = {};
    for (const g of listing?.groups ?? []) for (const f of g.files) if (f.suggestion?.target) m[f.path] = f.suggestion.target;
    return m;
  }, [listing]);
  /** Dosyanin gidecegi klasor: cubukta secilen hedef hepsine uygulanir; secilmediyse dosyanin onerilen klasoru. */
  const targetOf = useCallback((p: string): TargetFolder | null => defaultTarget ?? targets.find(t => t.name === suggested[p]) ?? null, [targets, suggested, defaultTarget]);
  const missingTarget = useMemo(() => movePaths.filter(p => !targetOf(p)).length, [movePaths, targetOf]);
  /** Grup adi (alt klasor) parcalari ya da hata metni; kurallar arka uctaki `split_group` ile aynidir. */
  const groupParts = useMemo(() => splitGroup(group), [group]);

  const setVerdict = useCallback((paths: string[], v: Verdict) => {
    setVerdicts(prev => {
      const next = { ...prev };
      for (const p of paths) { if (v === "keep") delete next[p]; else next[p] = v; }
      return next;
    });
  }, []);

  const run = useCallback(async (job: () => Promise<void>) => {
    setBusy(true);
    try { await job(); } catch (e) { setNotice({ type: "error", text: String(e) }); } finally { setBusy(false); }
  }, []);

  /** Tum kararlari uygular: once "Tasi" dosyalari kendi hedeflerine, sonra "At" dosyalari Geri Donusum Kutusu'na. */
  const applyAll = useCallback(() => run(async () => {
    if (missingTarget) throw new Error("Hedefi olmayan dosyalar var. Bir hedef klasör seçin.");
    if (typeof groupParts === "string") throw new Error(groupParts);
    const sub = groupParts.join("\\");
    const byTarget = new Map<string, { target: TargetFolder; paths: string[] }>();
    for (const p of movePaths) {
      const target = targetOf(p) as TargetFolder;
      byTarget.set(target.path, { target, paths: [...(byTarget.get(target.path)?.paths ?? []), p] });
    }
    const failed: FailedItem[] = [];
    const parts: string[] = [];
    for (const { target, paths } of byTarget.values()) {
      const out = await moveSelectedFiles(paths, target.path, sub);
      failed.push(...out.failed);
      parts.push(`${formatCount(out.moved.length)} dosya “${sub ? `${target.name}\\${sub}` : target.name}” klasörüne taşındı.`);
    }
    if (byTarget.size) parts.push("Taşımalar İşlem Geçmişi’nden geri alınabilir.");
    if (binPaths.length) {
      const out = await deleteFilesToTrash(binPaths);
      failed.push(...out.failed);
      parts.push(`${formatCount(out.trashed.length)} dosya Geri Dönüşüm Kutusu’na gönderildi.`);
    }
    setVerdicts({});
    setGroup("");
    const text = parts.join(" ");
    setNotice(failed.length ? { type: "error", text: `${text} ${formatCount(failed.length)} dosya işlenemedi: ${failed[0].error}` } : { type: "ok", text });
    await Promise.all([load(), refreshHistory()]);
  }), [movePaths, binPaths, missingTarget, targetOf, groupParts, load, refreshHistory, run]);

  const previewRename = useCallback((options: ListRenameOptions) => previewInboxRename(movePaths, options), [movePaths]);

  /** Adlandirma sonrasi dosya yolu degisir; "Tasi" kararlari yeni yollara tasinir. */
  const applyRename = useCallback((plan: RenamePlan) => run(async () => {
    const n = await applyRenamePlan(plan.id);
    const renamed = new Map(plan.entries.filter(e => e.to).map(e => [e.from, e.to as string]));
    setVerdicts(prev => Object.fromEntries(Object.entries(prev).map(([p, v]) => [renamed.get(p) ?? p, v])));
    setNotice({ type: "ok", text: `${formatCount(n)} dosya yeniden adlandırıldı. İşlem Geçmişi’nden geri alınabilir.` });
    await Promise.all([load(), refreshHistory()]);
  }), [load, refreshHistory, run]);

  const createTarget = useCallback(async (name: string) => {
    const t = await createTargetFolder(name);
    await loadTargets();
    return t;
  }, [loadTargets]);

  const pickDate = useCallback((d: string | null) => { setDate(d); setVerdicts({}); }, []);

  return {
    listing, date, loading, error, notice, setNotice, verdicts, movePaths, binPaths, counts, targets, busy,
    defaultTarget, setDefaultTarget: (t: TargetFolder | null) => setDefaultPath(t?.path ?? null),
    targetOf, missingTarget, group, setGroup, groupParts,
    reload: () => load(), pickDate, setVerdict, applyAll, previewRename, applyRename, createTarget,
  };
}

export type InboxState = ReturnType<typeof useInbox>;

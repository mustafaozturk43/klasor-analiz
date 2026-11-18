import { useMemo, useState } from "react";
import type { OrganizerState } from "../../types/app";
import { useFileList } from "./useFileList";
import { useFolderStats } from "./useFolderStats";

export type FolderSource = "A" | "B";
type Scope = FolderSource | "all";
export function useFolderBrowser(app: OrganizerState) {
  const key = JSON.stringify([app.rootPath, app.rootB]);
  const [navigation, setNavigation] = useState<{ key: string; scope: Scope; trail: { path: string; source: FolderSource }[] }>({ key, scope: "A", trail: [] });
  const scope = navigation.key === key ? navigation.scope : "A";
  const trail = navigation.key === key ? navigation.trail : [];
  const current = trail[trail.length - 1];
  const enabled = app.fileListEnabled;
  const a = enabled && (current ? current.source === "A" : scope !== "B") ? current?.path ?? app.rootPath : "";
  const b = enabled && (current ? current.source === "B" : scope !== "A") ? current?.path ?? app.rootB : "";
  const excludes = useMemo(() => app.excludeText.split(",").map(s => s.trim()).filter(Boolean), [app.excludeText]);
  const listA = useFileList(a, false, excludes, app.bannedExtensions, !!a, app.fileListMode === "all", app.statsToken);
  const listB = useFileList(b, false, excludes, app.bannedExtensions, !!b, app.fileListMode === "all", app.statsToken);
  const statsA = useFolderStats(a, app.statsToken);
  const statsB = useFolderStats(b, app.statsToken);
  const files = useMemo(() => [
    ...listA.files.map(file => ({ file, source: "A" as const })),
    ...listB.files.map(file => ({ file, source: "B" as const })),
  ], [listA.files, listB.files]);
  const folders = useMemo(() => {
    const hidden = new Set(["_kopyalar_karantina", ...excludes.map(s => s.toLowerCase())]);
    return [
      ...(statsA.stats?.folders ?? []).map(folder => ({ folder, source: "A" as const })),
      ...(statsB.stats?.folders ?? []).map(folder => ({ folder, source: "B" as const })),
    ].filter(entry => !hidden.has(entry.folder.name.toLowerCase()));
  }, [statsA.stats, statsB.stats, excludes]);
  return { scope, trail, current, files, folders,
    loading: listA.isListing || listB.isListing,
    folderLoading: statsA.loading || statsB.loading,
    errors: [listA.error, listB.error, statsA.error, statsB.error].filter(Boolean),
    warnings: [...(statsA.stats?.warnings ?? []), ...(statsB.stats?.warnings ?? [])],
    setScope: (scope: Scope) => setNavigation({ key, scope, trail: [] }),
    enter: (path: string, source: FolderSource) => setNavigation({ key, scope, trail: [...trail, { path, source }] }),
    back: () => setNavigation({ key, scope, trail: trail.slice(0, -1) }),
    home: () => setNavigation({ key, scope, trail: [] }),
  };
}

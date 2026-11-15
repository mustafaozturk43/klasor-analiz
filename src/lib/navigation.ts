/** Ekranlar hedeflere toplanir; her hedefin icindeki ekranlar menu cubugunda alt sekme olur. */
import type { Destination, Module } from "../types/app";

export interface DestinationDef {
  id: Destination;
  label: string;
  /** "ops": kenar cubugunda "Klasör-Dosya İşlemleri" grubunun alt ogesi; klasor secimi yalnizca bu grupta gorunur. */
  group?: "ops";
  /** Hedefin icindeki ekranlar; ilki varsayilandir. Tek elemanliysa menu cubugu cikmaz. */
  views: { id: Module; label: string }[];
}

export const OPS_LABEL = "Klasör-Dosya İşlemleri";

export const DESTINATIONS: DestinationDef[] = [
  { id: "today", label: "Bugün", views: [{ id: "evening", label: "Akşam Kontrolü" }] },
  { id: "duplicates", label: "Kopyalar", group: "ops", views: [
    { id: "groups", label: "Dosya grupları" }, { id: "compare", label: "İki klasör karşılaştırma" }, { id: "quarantine", label: "Karantina" },
  ] },
  { id: "map", label: "Harita", group: "ops", views: [
    { id: "analysis", label: "Klasör analizi" }, { id: "files", label: "Klasördeki dosyalar" }, { id: "health", label: "Klasör sağlığı" },
  ] },
  { id: "rename", label: "Adlandırma", group: "ops", views: [{ id: "naming", label: "Adlandırma" }] },
  { id: "rules", label: "Kurallar", views: [
    { id: "settings", label: "Kurallar" }, { id: "automation", label: "Otomasyon durumu" },
  ] },
  { id: "history", label: "Geçmiş", views: [{ id: "history", label: "İşlem geçmişi" }] },
];

export const destinationOf = (view: Module): Destination =>
  DESTINATIONS.find(d => d.views.some(v => v.id === view))?.id ?? "today";

export const defaultViews = (): Record<Destination, Module> =>
  Object.fromEntries(DESTINATIONS.map(d => [d.id, d.views[0].id])) as Record<Destination, Module>;

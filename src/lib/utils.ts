import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/** shadcn/ui sinif birlestirici: kosullu siniflar + cakisan Tailwind siniflarini sadelestirir. */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

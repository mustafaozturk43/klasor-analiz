/**
 * Uygulama ici uyarilar (SweetAlert2): islem onaylari kalici pencere, durum bildirimleri sag ustte
 * kisa bildirim olarak gelir. Gorunum `styles/alerts.css` ile token'lara baglanir; dugmeler `.btn`.
 */
import Swal from "sweetalert2";
import type { StatusMessage } from "../types/ui";

const base = {
  buttonsStyling: false,
  customClass: { popup: "sfo-alert", confirmButton: "btn primary", cancelButton: "btn" },
} as const;

/** Acik onay penceresi; bildirimler onu kapatmasin diye bitmesini bekler. */
let modal: Promise<unknown> = Promise.resolve();

/** Islem onayi. Odak "Vazgeç"tedir; Esc, disari tiklama veya pencerenin kapanmasi vazgecme sayilir. */
export async function confirmAlert(message: string, title: string, okLabel = "Onayla"): Promise<boolean> {
  // Swal.fire bir Promise degil, then'li bir nesne dondurur; gercek Promise'e cevrilir.
  const fired = Promise.resolve(Swal.fire({
    ...base, icon: "warning", title, text: message, showCancelButton: true, confirmButtonText: okLabel, cancelButtonText: "Vazgeç",
    focusCancel: true, reverseButtons: true,
  }));
  modal = fired.catch(() => undefined);
  return (await fired).isConfirmed;
}

/** Durum bildirimi: basari ve bilgi 6 saniyede kapanir, hata kullanici kapatana kadar kalir. */
export async function statusAlert(status: StatusMessage, onClose: () => void): Promise<void> {
  await modal;
  void Swal.fire({
    ...base, toast: true, position: "top-end", width: "26rem", icon: status.type, title: status.text, showConfirmButton: false, showCloseButton: true,
    timer: status.type === "error" ? undefined : 6000, timerProgressBar: true, didClose: onClose,
  });
}

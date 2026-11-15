/** UI genelinde kullanilan durum bildirimi tipi. */
export type StatusMessage = {
  type: "info" | "success" | "error";
  text: string;
};

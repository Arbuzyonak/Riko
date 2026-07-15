export type ToastKind = "info" | "success" | "error";

export interface ToastItem {
  id: number;
  message: string;
  kind: ToastKind;
}

let nextId = 1;

export const toastState = $state({
  items: [] as ToastItem[],
});

export function toast(message: string, kind: ToastKind = "info") {
  const id = nextId++;
  toastState.items.push({ id, message, kind });
  setTimeout(() => {
    const index = toastState.items.findIndex((t) => t.id === id);
    if (index !== -1) toastState.items.splice(index, 1);
  }, 5000);
}

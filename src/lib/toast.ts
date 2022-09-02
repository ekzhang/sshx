/** @file Provides a simple, native toast library. */

import { writable } from "svelte/store";

export const toastStore = writable<(Toast & { expires: number })[]>([]);

export type Toast = {
  kind: "info" | "success" | "error";
  message: string;
  action?: string;
  onAction?: () => void;
};

export function makeToast(toast: Toast, duration = 3000) {
  const obj = Object.assign({ expires: Date.now() + duration }, toast);
  toastStore.update(($toasts) => [...$toasts, obj]);
}

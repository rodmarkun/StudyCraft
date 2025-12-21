import { writable } from 'svelte/store';

export type ToastType = 'error' | 'success' | 'info' | 'warning';

export interface ToastMessage {
  id: string;
  message: string;
  type: ToastType;
  duration: number;
}

interface ToastState {
  current: ToastMessage | null;
}

function createToastStore() {
  const { subscribe, set, update } = writable<ToastState>({
    current: null
  });

  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  function clearCurrentTimeout() {
    if (timeoutId) {
      clearTimeout(timeoutId);
      timeoutId = null;
    }
  }

  function show(message: string, type: ToastType = 'error', duration: number = 5000) {
    clearCurrentTimeout();

    const toast: ToastMessage = {
      id: crypto.randomUUID ? crypto.randomUUID() : Date.now().toString(),
      message,
      type,
      duration
    };

    set({ current: toast });

    if (duration > 0) {
      timeoutId = setTimeout(() => {
        clear();
      }, duration);
    }
  }

  function clear() {
    clearCurrentTimeout();
    set({ current: null });
  }

  return {
    subscribe,
    show,
    error: (message: string, duration: number = 5000) => show(message, 'error', duration),
    success: (message: string, duration: number = 3000) => show(message, 'success', duration),
    info: (message: string, duration: number = 4000) => show(message, 'info', duration),
    warning: (message: string, duration: number = 4000) => show(message, 'warning', duration),
    clear
  };
}

export const toastStore = createToastStore();

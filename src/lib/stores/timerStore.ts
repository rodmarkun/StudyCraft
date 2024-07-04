// src/lib/stores/timerStore.ts
import { writable } from 'svelte/store';

interface TimerState {
  isRunning: boolean;
  startTime: number | null;
  elapsedTime: number;
}

function createTimerStore() {
  const { subscribe, set, update } = writable<TimerState>({
    isRunning: false,
    startTime: null,
    elapsedTime: 0,
  });

  return {
    subscribe,
    start: () => update(state => ({ ...state, isRunning: true, startTime: Date.now() })),
    pause: () => update(state => {
      if (state.isRunning && state.startTime) {
        const now = Date.now();
        return { ...state, isRunning: false, elapsedTime: state.elapsedTime + (now - state.startTime) };
      }
      return state;
    }),
    resume: () => update(state => ({ ...state, isRunning: true, startTime: Date.now() })),
    stop: () => set({ isRunning: false, startTime: null, elapsedTime: 0 }),
    reset: () => set({ isRunning: false, startTime: null, elapsedTime: 0 }),
  };
}

export const timerStore = createTimerStore();
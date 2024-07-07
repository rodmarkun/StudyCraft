import { writable } from 'svelte/store';

interface TimerState {
  isRunning: boolean;
  startTime: number | null;
  elapsedTime: number;
}

export interface StudySession {
  date: string; // ISO date string (YYYY-MM-DD)
  startTime: number;
  endTime: number;
  duration: number;
}

function createTimerStore() {
  const { subscribe, set, update } = writable<TimerState>({
    isRunning: false,
    startTime: null,
    elapsedTime: 0,
  });

  let studySessions: StudySession[] = [];

  async function loadStudySessions() {
    studySessions = await window.electronAPI.loadStudySessions();
  }

  async function saveStudySession(session: StudySession) {
    console.log('Saving study session:', session);
    await window.electronAPI.saveStudySession(session);
    studySessions.push(session);
  }

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
    stop: () => update(state => {
      if (state.isRunning && state.startTime) {
        const now = Date.now();
        const duration = state.elapsedTime + (now - state.startTime);
        if (duration > 0) {
          const session: StudySession = {
            date: new Date(state.startTime).toISOString().split('T')[0], // YYYY-MM-DD
            startTime: state.startTime,
            endTime: now,
            duration: duration,
          };
          saveStudySession(session);
        }
      }
      loadStudySessions(); // Reload sessions after stopping
      return { isRunning: false, startTime: null, elapsedTime: 0 };
    }),
    reset: () => set({ isRunning: false, startTime: null, elapsedTime: 0 }),
    getStudySessions: () => studySessions,
    loadStudySessions, // Expose this method to allow manual reloading of sessions
  };
}

export const timerStore = createTimerStore();
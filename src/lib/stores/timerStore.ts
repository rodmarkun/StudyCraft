import { writable } from 'svelte/store';

interface TimerState {
  isRunning: boolean;
  startTime: number | null;
  elapsedTime: number;
  isPomodoroMode: boolean;
  pomodoroLength: number;
  restLength: number;
  isRestPhase: boolean;
}

export interface StudySession {
  date: string;
  startTime: number;
  endTime: number;
  duration: number;
  isPomodoro: boolean;
}

function createTimerStore() {
  const { subscribe, set, update } = writable<TimerState>({
    isRunning: false,
    startTime: null,
    elapsedTime: 0,
    isPomodoroMode: false,
    pomodoroLength: 25 * 60 * 1000,
    restLength: 5 * 60 * 1000,
    isRestPhase: false,
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
        if (duration > 0 && !state.isRestPhase) {
          const session: StudySession = {
            date: new Date(state.startTime).toISOString().split('T')[0],
            startTime: state.startTime,
            endTime: now,
            duration: duration,
            isPomodoro: state.isPomodoroMode,
          };
          saveStudySession(session);
        }
      }
      loadStudySessions();
      return { 
        ...state,
        isRunning: false, 
        startTime: null, 
        elapsedTime: 0, 
        isPomodoroMode: false, 
        isRestPhase: false 
      };
    }),
    reset: () => set({ 
      isRunning: false, 
      startTime: null, 
      elapsedTime: 0, 
      isPomodoroMode: false,
      pomodoroLength: 25 * 60 * 1000,
      restLength: 5 * 60 * 1000,
      isRestPhase: false,
    }),
    togglePomodoroMode: () => update(state => ({ ...state, isPomodoroMode: !state.isPomodoroMode })),
    setPomodoroLength: (length: number) => update(state => ({ ...state, pomodoroLength: length })),
    setRestLength: (length: number) => update(state => ({ ...state, restLength: length })),
    switchPhase: (callback?: () => void) => update(state => {
      if (state.isRunning && state.startTime) {
        const now = Date.now();
        const duration = state.elapsedTime + (now - state.startTime);
        if (!state.isRestPhase && duration > 0) {
          const session: StudySession = {
            date: new Date(state.startTime).toISOString().split('T')[0],
            startTime: state.startTime,
            endTime: now,
            duration: duration,
            isPomodoro: true,
          };
          saveStudySession(session);
        }
      }
      if (callback) {
        callback();
      }
      return { 
        ...state, 
        isRestPhase: !state.isRestPhase, 
        elapsedTime: 0, 
        startTime: null,
        isRunning: false 
      };
    }),
    getStudySessions: () => studySessions,
    loadStudySessions,
  };
}

export const timerStore = createTimerStore();
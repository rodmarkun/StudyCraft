<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { timerStore } from '../stores/timerStore';
  import Popover from './Popover.svelte';
  import TimeInput from './TimeInput.svelte';

  let displayTime = '00:00:00';
  let intervalId: number;
  let popoverOpen = false;
  let audio: HTMLAudioElement;

  function formatTime(ms: number): string {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    return [
      hours.toString().padStart(2, '0'),
      (minutes % 60).toString().padStart(2, '0'),
      (seconds % 60).toString().padStart(2, '0')
    ].join(':');
  }

  function updateDisplay() {
    const now = Date.now();
    const { isRunning, startTime, elapsedTime, isPomodoroMode, pomodoroLength, restLength, isRestPhase } = $timerStore;
    if (isPomodoroMode) {
      const totalElapsed = elapsedTime + (isRunning && startTime ? now - startTime : 0);
      const remainingTime = isRestPhase ? restLength - totalElapsed : pomodoroLength - totalElapsed;
      displayTime = formatTime(Math.max(0, remainingTime));
      if (remainingTime <= 0) {
        timerStore.switchPhase(playNotificationSound);
      }
    } else {
      const totalElapsed = elapsedTime + (isRunning && startTime ? now - startTime : 0);
      displayTime = formatTime(totalElapsed);
    }
  }

  function playNotificationSound() {
    if (audio) {
      audio.play().catch(error => {
        console.error('Error playing notification sound:', error);
      });
    }
  }

  onMount(() => {
    audio = new Audio('/src/assets/notification-sound.wav');
    audio.volume = 0.2;
    intervalId = setInterval(updateDisplay, 1000);
  });

  onDestroy(() => {
    clearInterval(intervalId);
  });

  function handleToggle() {
    if ($timerStore.isRunning) {
      timerStore.pause();
    } else if ($timerStore.elapsedTime > 0 || $timerStore.isPomodoroMode) {
      timerStore.resume();
    } else {
      timerStore.start();
    }
  }

  function handleStop() {
    timerStore.stop();
    console.log('Study session ended:', displayTime);
  }

  function handlePomodoroToggle() {
    if (!$timerStore.isRunning) {
      timerStore.togglePomodoroMode();
    }
  }

  function handlePomodoroLengthChange(newValue: number) {
    timerStore.setPomodoroLength(newValue);
  }

  function handleRestLengthChange(newValue: number) {
    timerStore.setRestLength(newValue);
  }
</script>

<div class="flex items-center">
  <span class="text-xl font-mono font-semibold mr-2">{displayTime}</span>
  <div class="flex">
    <button
      on:click={handleToggle}
      class="p-1.5 rounded-full bg-white dark:bg-gray-800 text-gray-800 dark:text-white hover:bg-gray-100 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500 dark:focus:ring-gray-400 transition-colors duration-200"
      title={$timerStore.isRunning ? "Pause" : "Start/Resume"}
    >
      {#if $timerStore.isRunning}
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      {/if}
    </button>
    <button
      on:click={handleStop}
      class="p-1.5 mx-1 rounded-full bg-white dark:bg-gray-800 text-gray-800 dark:text-white hover:bg-gray-100 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500 dark:focus:ring-gray-400 transition-colors duration-200"
      title="Stop"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
      </svg>
    </button>
    <Popover bind:open={popoverOpen} width="w-64">
      <button
        slot="trigger"
        on:click={() => popoverOpen = !popoverOpen}
        class="p-1.5 rounded-full bg-white dark:bg-gray-800 text-gray-800 dark:text-white hover:bg-gray-100 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500 dark:focus:ring-gray-400 transition-colors duration-200"
        class:text-gray-900={$timerStore.isPomodoroMode}
        title="Pomodoro Settings"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      </button>
      <div class="space-y-4">
        <label class="flex items-center space-x-2">
          <input
            type="checkbox"
            checked={$timerStore.isPomodoroMode}
            on:change={handlePomodoroToggle}
            disabled={$timerStore.isRunning}
            class="form-checkbox h-5 w-5 text-gray-600 dark:text-gray-400"
          />
          <span class="text-gray-900 dark:text-white">Pomodoro Mode</span>
        </label>
        <div class="space-y-2">
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
            Pomodoro Length
          </label>
          <TimeInput value={$timerStore.pomodoroLength} onChange={handlePomodoroLengthChange} />
        </div>
        <div class="space-y-2">
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
            Rest Length
          </label>
          <TimeInput value={$timerStore.restLength} onChange={handleRestLengthChange} />
        </div>
      </div>
    </Popover>
  </div>
</div>
<!-- src/lib/components/StudyTimer.svelte -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { timerStore } from '../stores/timerStore';

  let displayTime = '00:00:00';
  let intervalId: number;

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
    const { isRunning, startTime, elapsedTime } = $timerStore;
    const totalElapsed = elapsedTime + (isRunning && startTime ? now - startTime : 0);
    displayTime = formatTime(totalElapsed);
  }

  onMount(() => {
    intervalId = setInterval(updateDisplay, 1000);
  });

  onDestroy(() => {
    clearInterval(intervalId);
  });

  function handleToggle() {
    if ($timerStore.isRunning) {
      timerStore.pause();
    } else if ($timerStore.elapsedTime > 0) {
      timerStore.resume();
    } else {
      timerStore.start();
    }
  }

  function handleStop() {
    timerStore.stop();
    console.log('Study session ended:', displayTime);
  }
</script>

<div class="flex items-center space-x-2">
  <span class="text-xl font-mono">{displayTime}</span>
  <div class="flex space-x-1">
    <button
      on:click={handleToggle}
      class="p-1.5 rounded-full focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-text-light dark:focus:ring-text-dark"
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
      class="p-1.5 rounded-full focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-text-light dark:focus:ring-text-dark"
      title="Stop"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
      </svg>
    </button>
  </div>
</div>
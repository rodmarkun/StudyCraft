<!-- src/lib/components/Calendar.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { timerStore, type StudySession } from '../stores/timerStore';

  let currentDate = new Date();
  let daysInMonth: Date[] = [];
  let studiedDays: Set<string> = new Set();
  let studySessions: StudySession[] = [];
  let hoveredDay: string | null = null;
  let hoveredSessions: StudySession[] = [];
  let tooltipPosition = { x: 0, y: 0 };

  function getDaysInMonth(date: Date): Date[] {
    const year = date.getFullYear();
    const month = date.getMonth();
    const days = new Date(year, month + 1, 0).getDate();
    return Array.from({ length: days }, (_, i) => new Date(year, month, i + 1));
  }

  function prevMonth() {
    currentDate = new Date(currentDate.getFullYear(), currentDate.getMonth() - 1, 1);
    updateCalendar();
  }

  function nextMonth() {
    currentDate = new Date(currentDate.getFullYear(), currentDate.getMonth() + 1, 1);
    updateCalendar();
  }

  function updateCalendar() {
    daysInMonth = getDaysInMonth(currentDate);
    const currentMonth = currentDate.getMonth();
    const currentYear = currentDate.getFullYear();
   
    studiedDays = new Set(
      studySessions
        .filter(session => {
          const sessionDate = new Date(session.date);
          return sessionDate.getMonth() === currentMonth && sessionDate.getFullYear() === currentYear;
        })
        .map(session => session.date)
    );
  }

  function handleDayHover(event: MouseEvent, day: Date) {
    const dateString = day.toISOString().split('T')[0];
    hoveredDay = dateString;
    hoveredSessions = studySessions.filter(session => session.date === dateString);

    const rect = (event.target as HTMLElement).getBoundingClientRect();
    tooltipPosition = {
      x: rect.left + window.scrollX,
      y: rect.bottom + window.scrollY
    };
  }

  function handleDayLeave() {
    hoveredDay = null;
    hoveredSessions = [];
  }

  function formatDuration(duration: number): string {
    const hours = Math.floor(duration / 3600000);
    const minutes = Math.floor((duration % 3600000) / 60000);
    const seconds = Math.floor((duration % 60000) / 1000);
    return `${hours}h ${minutes}m ${seconds}s`;
  }

  onMount(async () => {
    await timerStore.loadStudySessions();
    studySessions = timerStore.getStudySessions();
    updateCalendar();
  });
</script>

<div class="calendar">
  <div class="flex justify-between items-center mb-4">
    <button on:click={prevMonth} class="text-text-light dark:text-text-dark">←</button>
    <h2 class="text-xl font-bold text-text-light dark:text-text-dark">
      {currentDate.toLocaleString('default', { month: 'long', year: 'numeric' })}
    </h2>
    <button on:click={nextMonth} class="text-text-light dark:text-text-dark">→</button>
  </div>
  <div class="grid grid-cols-7 gap-2">
    {#each ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'] as day}
      <div class="text-center font-medium text-gray-500 dark:text-gray-400">{day}</div>
    {/each}
    {#each daysInMonth as day, i}
      <div
        class="text-center p-2 rounded-full cursor-pointer
               {studiedDays.has(day.toISOString().split('T')[0]) ? 'bg-green-500 text-white' : 'text-text-light dark:text-text-dark'}
               {day.getMonth() === currentDate.getMonth() ? '' : 'opacity-50'}"
        on:mouseenter={(e) => handleDayHover(e, day)}
        on:mouseleave={handleDayLeave}
      >
        {day.getDate()}
      </div>
    {/each}
  </div>
</div>

{#if hoveredDay && hoveredSessions.length > 0}
  <div 
    class="fixed z-50 p-4 bg-white dark:bg-gray-800 rounded shadow-lg border border-gray-200 dark:border-gray-700"
    style="left: {tooltipPosition.x}px; top: {tooltipPosition.y}px;"
  >
    <h3 class="font-bold mb-2">Study sessions on {hoveredDay}:</h3>
    <ul>
      {#each hoveredSessions as session}
        <li>
          {new Date(session.startTime).toLocaleTimeString()} - {new Date(session.endTime).toLocaleTimeString()}
          (Duration: {formatDuration(session.duration)})
        </li>
      {/each}
    </ul>
  </div>
{/if}
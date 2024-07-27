<script lang="ts">
  import { onMount } from 'svelte';
  import { timerStore, type StudySession } from '../stores/timerStore';

  let currentDate = new Date();
  let calendarDays: (Date | null)[] = [];
  let studiedDays: Set<string> = new Set();
  let studySessions: StudySession[] = [];
  let hoveredDay: string | null = null;
  let hoveredSessions: StudySession[] = [];
  let tooltipPosition = { x: 0, y: 0 };

  function getCalendarDays(date: Date): (Date | null)[] {
    const year = date.getFullYear();
    const month = date.getMonth();
    const firstDay = new Date(year, month, 1);
    const lastDay = new Date(year, month + 1, 0);
    
    const daysInMonth = lastDay.getDate();
    const startingDay = firstDay.getDay();
    
    const calendarArray: (Date | null)[] = [];
    
    // Fill in the days before the first day of the month
    for (let i = 0; i < startingDay; i++) {
      calendarArray.push(null);
    }
    
    // Fill in the days of the month
    for (let i = 1; i <= daysInMonth; i++) {
      calendarArray.push(new Date(year, month, i));
    }
    
    // Fill in any remaining slots to complete the grid
    while (calendarArray.length % 7 !== 0) {
      calendarArray.push(null);
    }
    
    return calendarArray;
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
    calendarDays = getCalendarDays(currentDate);
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
    if (!day) return;
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

<div class="calendar bg-white dark:bg-gray-800 p-4 rounded-lg shadow">
  <div class="flex justify-between items-center mb-4">
    <button on:click={prevMonth} class="text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-white">
      <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
      </svg>
    </button>
    <h2 class="text-xl font-bold text-gray-800 dark:text-white">
      {currentDate.toLocaleString('default', { month: 'long', year: 'numeric' })}
    </h2>
    <button on:click={nextMonth} class="text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-white">
      <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
      </svg>
    </button>
  </div>
  <div class="grid grid-cols-7 gap-2">
    {#each ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'] as day}
      <div class="text-center font-medium text-gray-500 dark:text-gray-400">{day}</div>
    {/each}
    {#each calendarDays as day, i}
      <div
        class="text-center p-2 rounded-full cursor-pointer transition-colors duration-200
               {day && studiedDays.has(day.toISOString().split('T')[0]) ? 'bg-green-500 text-white' : 'hover:bg-gray-100 dark:hover:bg-gray-700'}
               {day ? 'text-gray-800 dark:text-gray-200' : 'text-gray-400 dark:text-gray-600'}"
        on:mouseenter={(e) => day && handleDayHover(e, day)}
        on:mouseleave={handleDayLeave}
      >
        {day ? day.getDate() : ''}
      </div>
    {/each}
  </div>
</div>

{#if hoveredDay && hoveredSessions.length > 0}
  <div 
    class="fixed z-50 p-4 bg-white dark:bg-gray-800 rounded shadow-lg border border-gray-200 dark:border-gray-700"
    style="left: {tooltipPosition.x}px; top: {tooltipPosition.y}px;"
  >
    <h3 class="font-bold mb-2 text-gray-800 dark:text-white">Study sessions on {hoveredDay}:</h3>
    <ul class="space-y-1">
      {#each hoveredSessions as session}
        <li class="text-gray-600 dark:text-gray-300">
          {new Date(session.startTime).toLocaleTimeString()} - {new Date(session.endTime).toLocaleTimeString()}
          <span class="text-gray-500 dark:text-gray-400">(Duration: {formatDuration(session.duration)})</span>
        </li>
      {/each}
    </ul>
  </div>
{/if}
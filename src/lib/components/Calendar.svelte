<!-- src/lib/components/Calendar.svelte -->
<script lang="ts">
    import { onMount } from 'svelte';
  
    let currentDate = new Date();
    let daysInMonth: Date[] = [];
    let studiedDays: Set<number> = new Set([5, 10, 15, 20]); // Dummy data for now
  
    function getDaysInMonth(date: Date): Date[] {
      const year = date.getFullYear();
      const month = date.getMonth();
      const days = new Date(year, month + 1, 0).getDate();
      return Array.from({ length: days }, (_, i) => new Date(year, month, i + 1));
    }
  
    function prevMonth() {
      currentDate = new Date(currentDate.getFullYear(), currentDate.getMonth() - 1, 1);
      daysInMonth = getDaysInMonth(currentDate);
    }
  
    function nextMonth() {
      currentDate = new Date(currentDate.getFullYear(), currentDate.getMonth() + 1, 1);
      daysInMonth = getDaysInMonth(currentDate);
    }
  
    onMount(() => {
      daysInMonth = getDaysInMonth(currentDate);
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
          class="text-center p-2 rounded-full
                 {studiedDays.has(day.getDate()) ? 'bg-green-500 text-white' : 'text-text-light dark:text-text-dark'}
                 {day.getMonth() === currentDate.getMonth() ? '' : 'opacity-50'}"
        >
          {day.getDate()}
        </div>
      {/each}
    </div>
  </div>
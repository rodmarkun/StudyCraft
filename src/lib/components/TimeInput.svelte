<script lang="ts">
    export let value: number; // in milliseconds
    export let onChange: (newValue: number) => void;
  
    let hours = Math.floor(value / 3600000);
    let minutes = Math.floor((value % 3600000) / 60000);
    let seconds = Math.floor((value % 60000) / 1000);
  
    function handleChange() {
      const newValue = (hours * 3600 + minutes * 60 + seconds) * 1000;
      onChange(newValue);
    }
  </script>
  
  <div class="flex space-x-2">
    <input
      type="number"
      bind:value={hours}
      min="0"
      max="23"
      on:change={handleChange}
      class="w-16 form-input block text-gray-900 dark:text-white bg-white dark:bg-gray-700"
    />
    <span class="text-gray-900 dark:text-white">:</span>
    <input
      type="number"
      bind:value={minutes}
      min="0"
      max="59"
      on:change={handleChange}
      class="w-16 form-input block text-gray-900 dark:text-white bg-white dark:bg-gray-700"
    />
    <span class="text-gray-900 dark:text-white">:</span>
    <input
      type="number"
      bind:value={seconds}
      min="0"
      max="59"
      on:change={handleChange}
      class="w-16 form-input block text-gray-900 dark:text-white bg-white dark:bg-gray-700"
    />
  </div>
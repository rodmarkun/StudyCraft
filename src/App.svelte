<!-- src/App.svelte -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { writable } from 'svelte/store';
	import StudyTimer from './lib/components/StudyTimer.svelte';
	import Collections from './lib/components/Collections.svelte';
	import Modal from './lib/components/Modal.svelte';
	import Calendar from './lib/components/Calendar.svelte';
	import { collections } from './lib/stores/collections';
	import logoLight from './assets/StudyCraftLogo2Transparente.png';
    import logoDark from './assets/StudyCraftLogo2TransparenteInv.png';
  
	let darkMode = false;
	let isCalendarModalOpen = false;
  
	onMount(() => {
	  darkMode = localStorage.getItem('theme') === 'dark' || 
				 (!localStorage.getItem('theme') && window.matchMedia('(prefers-color-scheme: dark)').matches);
	  updateTheme();
	});
  
	function toggleTheme() {
	  darkMode = !darkMode;
	  updateTheme();
	  collections.initialize();
	}
  
	function updateTheme() {
	  if (darkMode) {
		document.documentElement.classList.add('dark');
		localStorage.setItem('theme', 'dark');
	  } else {
		document.documentElement.classList.remove('dark');
		localStorage.setItem('theme', 'light');
	  }
	}
  
	function openCalendarModal() {
	  isCalendarModalOpen = true;
	}
  
	function closeCalendarModal() {
	  isCalendarModalOpen = false;
	}
  
	// Simple routing
	const route = writable('collections');
  </script>
  
  <div class="min-h-screen bg-background-light dark:bg-background-dark text-text-light dark:text-text-dark transition-colors duration-300 font-body">
    <nav class="bg-white dark:bg-gray-800 shadow-md">
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div class="flex justify-between items-center h-16">
                <div class="flex-shrink-0 flex items-center space-x-2">
                    <img src={darkMode ? logoDark : logoLight}  alt="StudyCraft Logo" class="h-8 w-auto" /> <!-- Add the logo here -->
                    <span class="text-2xl font-bold text-text-light dark:text-text-dark font-header">StudyCraft</span>
                </div>
		  <div class="flex-grow flex justify-center items-center">
			<StudyTimer />
		  </div>
		  <div class="flex items-center space-x-4">
			<button
			  on:click={openCalendarModal}
			  class="p-2 rounded-md text-text-light dark:text-text-dark hover:bg-gray-200 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-light dark:focus:ring-primary-dark"
			  aria-label="View Calendar"
			>
			  <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
			  </svg>
			</button>
			<button
			  on:click={toggleTheme}
			  class="p-2 rounded-md text-text-light dark:text-text-dark hover:bg-gray-200 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-light dark:focus:ring-primary-dark"
			>
			  {#if darkMode}
				🌙
			  {:else}
				☀️
			  {/if}
			</button>
		  </div>
		</div>
	  </div>
	</nav>
  
	<main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
	  <Collections />
	</main>
  
	<Modal isOpen={isCalendarModalOpen} title="Study Calendar" on:close={closeCalendarModal}>
	  <Calendar />
	</Modal>
  </div>
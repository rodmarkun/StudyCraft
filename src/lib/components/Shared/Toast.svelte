<script lang="ts">
    import { fade } from "svelte/transition";
    import { onMount, onDestroy } from "svelte";

    // Types
    export type ToastType = 'error' | 'success' | 'info' | 'warning';

    // Props
    export let message: string = '';
    export let type: ToastType = 'error';
    export let duration: number = 0; // 0 means no auto-close
    export let onClose: () => void = () => {};

    // State
    let timeoutId: ReturnType<typeof setTimeout> | null = null;

    // Auto-close logic
    $: if (message && duration > 0) {
        clearAutoClose();
        timeoutId = setTimeout(() => {
            onClose();
        }, duration);
    }

    function clearAutoClose() {
        if (timeoutId) {
            clearTimeout(timeoutId);
            timeoutId = null;
        }
    }

    function handleClose() {
        clearAutoClose();
        onClose();
    }

    onDestroy(() => {
        clearAutoClose();
    });
</script>

{#if message && message.trim()}
    <div
        class="toast-container"
        transition:fade={{duration: 200}}
        role="alert"
        aria-live="polite"
    >
        <div class="toast toast-{type}">
            <div class="toast-icon">
                {#if type === 'error'}
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <circle cx="12" cy="12" r="10"></circle>
                        <line x1="15" y1="9" x2="9" y2="15"></line>
                        <line x1="9" y1="9" x2="15" y2="15"></line>
                    </svg>
                {:else if type === 'success'}
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                        <polyline points="22 4 12 14.01 9 11.01"></polyline>
                    </svg>
                {:else if type === 'warning'}
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
                        <line x1="12" y1="9" x2="12" y2="13"></line>
                        <line x1="12" y1="17" x2="12.01" y2="17"></line>
                    </svg>
                {:else}
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <circle cx="12" cy="12" r="10"></circle>
                        <line x1="12" y1="16" x2="12" y2="12"></line>
                        <line x1="12" y1="8" x2="12.01" y2="8"></line>
                    </svg>
                {/if}
            </div>
            <span class="message">{message}</span>
            <button class="close" on:click={handleClose} aria-label="Close notification">×</button>
        </div>
    </div>
{/if}

<style>
 .toast-container {
   position: fixed;
   top: calc(var(--space-lg) + 60px);
   left: 50%;
   transform: translateX(-50%);
   z-index: 1000;
   pointer-events: none;
 }

 .toast {
   padding: var(--space-sm) var(--space-lg);
   border-radius: var(--border-radius);
   box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
   display: flex;
   align-items: center;
   gap: var(--space-md);
   pointer-events: all;
   max-width: 600px;
   border: 1px solid transparent;
 }

 .toast-icon {
   display: flex;
   align-items: center;
   justify-content: center;
   flex-shrink: 0;
 }

 .toast-error {
   background: color-mix(in srgb, var(--error) 15%, var(--surface));
   color: var(--error);
   border-color: color-mix(in srgb, var(--error) 30%, transparent);
 }

 .toast-success {
   background: color-mix(in srgb, var(--success) 15%, var(--surface));
   color: var(--success);
   border-color: color-mix(in srgb, var(--success) 30%, transparent);
 }

 .toast-warning {
   background: color-mix(in srgb, #f59e0b 15%, var(--surface));
   color: #f59e0b;
   border-color: color-mix(in srgb, #f59e0b 30%, transparent);
 }

 .toast-info {
   background: color-mix(in srgb, var(--accent) 15%, var(--surface));
   color: var(--accent);
   border-color: color-mix(in srgb, var(--accent) 30%, transparent);
 }

 .message {
   flex: 1;
   font-size: 0.9rem;
   line-height: 1.4;
 }

 .close {
   background: none;
   border: none;
   color: inherit;
   font-size: 1.5rem;
   cursor: pointer;
   padding: 0;
   line-height: 1;
   opacity: 0.7;
   transition: opacity 0.2s ease;
   flex-shrink: 0;
 }

 .close:hover {
   opacity: 1;
 }

 .close:focus {
   outline: 2px solid currentColor;
   outline-offset: 2px;
   border-radius: 4px;
 }
</style>
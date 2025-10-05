<script>
    import { fade } from "svelte/transition";

    // Props
    export let message = '';
    export let onClose = () => {};
</script>

{#if message}
    <div 
        class="toast-container" 
        transition:fade={{duration: 200}}
    >
        <div class="toast">
            <span class="message">{message}</span>
            <button class="close" on:click={onClose}>×</button>
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
   background: color-mix(in srgb, var(--error) 15%, var(--surface));
   color: var(--error);
   padding: var(--space-sm) var(--space-lg);
   border-radius: var(--border-radius);
   box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
   display: flex;
   align-items: center;
   gap: var(--space-md);
   pointer-events: all;
   max-width: 600px;
 }
 
 .message {
   flex: 1;
   font-size: 0.9rem;
 }

 .close {
   background: none;
   border: none;
   color: var(--error);
   font-size: 1.5rem;
   cursor: pointer;
   padding: 0;
   line-height: 1;
   opacity: 0.7;
   transition: opacity var(--transition-speed) ease;
 }
 
 .close:hover {
   opacity: 1;
 }
</style>
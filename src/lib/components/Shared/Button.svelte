<script lang="ts">
  // Props
  export let variant: 'save' | 'danger' | 'primary' | 'secondary' = 'primary';
  export let disabled = false;
  export let changed = false; 
  export let title = '';
  export let icon: any = null; 
  export let text = '';
  export let changedText = '';
  
  // Callbacks
  export let onClick = () => {};

  // Computed
  $: displayText = changed && changedText ? changedText : text;
  $: buttonTitle = title || displayText;
</script>

<button 
  class="action-button {variant}"
  class:changed
  on:click={onClick}
  {disabled}
  title={buttonTitle}
>
  {#if icon}
    <svelte:component this={icon} size={16} />
  {/if}
  <span>{displayText}</span>
</button>
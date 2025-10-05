<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { tweened } from "svelte/motion";
  import { cubicOut } from "svelte/easing";

  // Props
  export let compact = false;
  export let onZoomChange: ((zoom: number) => void) | undefined = undefined;

  // Consts
  const zoom = tweened(1, {
    duration: 300,
    easing: cubicOut,
  });
  const presets = [0.5, 0.6, 0.7, 0.8, 0.9, 1, 1.1, 1.2, 1.3, 1.4, 1.5];

  // Reactivity
  $: percentage = `${Math.round($zoom * 100)}%`;
  $: canZoomIn = $zoom < 2;
  $: canZoomOut = $zoom > 0.5;
  $: {
    if (onZoomChange) {
      onZoomChange($zoom);
    }
  }

  // Function
  function findClosestPreset(level: number): number {
    return presets.reduce((prev, curr) =>
      Math.abs(curr - level) < Math.abs(prev - level) ? curr : prev,
    );
  }

  function zoomIn() {
    const current = $zoom;
    const closestPreset = findClosestPreset(current);
    const currentIndex = presets.indexOf(closestPreset);

    if (currentIndex < presets.length - 1) {
      const newLevel = presets[currentIndex + 1];
      zoom.set(newLevel);
      localStorage.setItem("studycraft-zoom", newLevel.toString());
    }
  }

  function zoomOut() {
    const current = $zoom;
    const closestPreset = findClosestPreset(current);
    const currentIndex = presets.indexOf(closestPreset);

    if (currentIndex > 0) {
      const newLevel = presets[currentIndex - 1];
      zoom.set(newLevel);
      localStorage.setItem("studycraft-zoom", newLevel.toString());
    }
  }

  function reset() {
    zoom.set(1);
    localStorage.setItem("studycraft-zoom", "1");
  }

  function handleKeydown(event: KeyboardEvent) {
    // Check if user is typing in any input element
    const target = event.target as HTMLElement;
    const isTypingInInput =
      target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.tagName === "SELECT" ||
      target.hasAttribute("contenteditable") ||
      target.isContentEditable ||
      target.closest('[contenteditable="true"]') ||
      target.closest("input") ||
      target.closest("textarea");

    // Don't handle shortcuts if user is typing
    if (isTypingInInput) {
      return;
    }

    // Only handle Ctrl/Cmd shortcuts
    if (event.ctrlKey || event.metaKey) {
      switch (event.key) {
        case "=":
        case "+":
          event.preventDefault();
          zoomIn();
          break;
        case "-":
          event.preventDefault();
          zoomOut();
          break;
        case "0":
          event.preventDefault();
          reset();
          break;
      }
    }
  }

  function handleWheel(event: WheelEvent) {
    // Only handle Ctrl/Cmd + scroll
    if (event.ctrlKey || event.metaKey) {
      event.preventDefault();

      const threshold = 5;
      if (Math.abs(event.deltaY) > threshold) {
        if (event.deltaY > 0) {
          zoomOut();
        } else {
          zoomIn();
        }
      }
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
    window.addEventListener("wheel", handleWheel, { passive: false });
  });

  onDestroy(() => {
    window.removeEventListener("keydown", handleKeydown);
    window.removeEventListener("wheel", handleWheel);
  });
</script>

<div class="zoom-controls" class:compact>
  <div class="main-controls">
    <button
      class="zoom-button"
      on:click={zoomOut}
      disabled={!canZoomOut}
      title="Zoom out (Ctrl + -)"
    >
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <circle cx="11" cy="11" r="8" />
        <path d="21 21l-4.35-4.35" />
        <line x1="8" y1="11" x2="14" y2="11" />
      </svg>
    </button>

    <button
      class="zoom-level"
      title="Reset to 100% (Ctrl + 0)"
      on:click={reset}
    >
      {percentage}
    </button>

    <button
      class="zoom-button"
      on:click={zoomIn}
      disabled={!canZoomIn}
      title="Zoom in (Ctrl + +)"
    >
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <circle cx="11" cy="11" r="8" />
        <path d="21 21l-4.35-4.35" />
        <line x1="11" y1="8" x2="11" y2="14" />
        <line x1="8" y1="11" x2="14" y2="11" />
      </svg>
    </button>
  </div>
</div>

<style>
  .zoom-controls {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    background: var(--surface, #ffffff);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: 6px;
    padding: 0rem;
  }

  .zoom-controls.compact {
    padding: 0.25rem;
  }

  .main-controls {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .zoom-button {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.5rem;
    border: none;
    background: none;
    color: var(--text-secondary, #64748b);
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.2s ease;
  }

  .zoom-button:hover:not(:disabled) {
    color: var(--text, #1e293b);
  }

  .zoom-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .zoom-level {
    min-width: 4rem;
    padding: 0.5rem 0.75rem;
    border: none;
    background: none;
    color: var(--text-secondary, #64748b);
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.2s ease;
  }

  .zoom-level:hover {
    color: var(--text, #1e293b);
  }
</style>

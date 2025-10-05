<script>
    import { theme } from "../../stores/theme";

    // Icons
    import { BookOpen as BookOpenIcon } from "lucide-svelte";
    import { Brain as BrainIcon } from "lucide-svelte";
    import { ChartSpline as ChartSplineIcon } from "lucide-svelte";
    import { Settings as SettingsIcon } from "lucide-svelte";

    // State
    export let currViewMode = "study"; // 'study', 'review', or 'stats'

    // Callbacks
    export let onViewChange = (mode) => {};
    export let onOpenSettings = () => {};

    // Functions
    function toggleTheme() {
        theme.update((t) => (t === "light" ? "dark" : "light"));
    }

    function setMode(mode) {
        currViewMode = mode;
        onViewChange(mode);
    }

    function openSettings() {
        onOpenSettings();
    }
</script>

<header>
    <div class="left-section">
        <div class="welcome">StudyCraft</div>
        <div class="mode-buttons">
            <button
                class="mode-button"
                class:active={currViewMode === "study"}
                on:click={() => setMode("study")}
            >
                <BookOpenIcon size={18} />
                Study
            </button>

            <div class="mode-separator"></div>

            <button
                class="mode-button"
                class:active={currViewMode === "review"}
                on:click={() => setMode("review")}
            >
                <BrainIcon size={18} />
                Review
            </button>

            <div class="mode-separator"></div>

            <button
                class="mode-button"
                class:active={currViewMode === "stats"}
                on:click={() => setMode("stats")}
            >
                <ChartSplineIcon size={18} />
                Stats
            </button>
        </div>
    </div>
    <div class="right-section">
        <button class="theme-toggle" on:click={toggleTheme}>
            {$theme === "light" ? "☀️" : "🌙"}
        </button>
        <button
            aria-label="Settings"
            class="icon-button"
            on:click={openSettings}
            title="Settings"
        >
            <SettingsIcon size={18}></SettingsIcon>
        </button>
    </div>
</header>

<style>
    header {
        padding: var(--space-md) var(--space-lg);
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-bottom: 1px solid var(--border);
        background: var(--surface);
    }

    .left-section {
        display: flex;
        align-items: center;
        gap: var(--space-xl);
    }

    .right-section {
        display: flex;
        align-items: center;
        gap: var(--space-md);
    }

    .welcome {
        font-size: 1.2rem;
        font-weight: 400;
        letter-spacing: 0.05em;
        color: var(--text);
        font-family: var(--font-heading);
    }

    .mode-buttons {
        display: flex;
        align-items: center;
        gap: var(--space-md);
    }

    header .mode-button {
        background: none !important;
        border: none !important;
        font-family: var(--font-body) !important;
        font-size: 0.95rem !important;
        color: var(--text-secondary) !important;
        cursor: pointer !important;
        padding: var(--space-sm) var(--space-md) !important;
        transition: all var(--transition-speed) ease !important;
        position: relative !important;
        display: flex !important;
        align-items: center !important;
        gap: var(--space-sm) !important;
        border-radius: 0 !important;
        min-width: auto !important;
        height: auto !important;
        border-right: none !important;
        box-sizing: border-box !important;
    }

    header .mode-button:focus {
        outline: none !important;
        border: none !important;
    }

    header .mode-button:focus-visible {
        outline: none !important;
        border: none !important;
    }

    header .mode-button.active {
        color: var(--text) !important;
        background: none !important;
    }

    header .mode-button.active::after {
        content: "";
        position: absolute;
        bottom: -2px;
        left: 0;
        width: 100%;
        height: 2px;
        background: var(--accent);
        border-radius: 2px;
    }

    header .mode-button:hover {
        color: var(--text);
        transform: rotate(3deg) scale(1);
    }

    .mode-separator {
        width: 1px;
        height: 20px;
        background: var(--border);
    }

    .theme-toggle {
        background: none;
        border: none;
        font-size: 1.2rem;
        cursor: pointer;
        padding: var(--space-sm);
        border-radius: var(--border-radius);
        transition: transform var(--transition-speed) ease;
    }

    .theme-toggle:focus {
        outline: none;
    }

    .theme-toggle:hover {
        transform: scale(1.1);
    }

    .icon-button {
        background: none;
        border: none;
        color: var(--text-secondary);
        cursor: pointer;
        padding: var(--space-sm);
        border-radius: var(--border-radius);
        transition: all var(--transition-speed) ease;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .icon-button:focus {
        outline: none;
    }

    .icon-button:hover {
        color: var(--text);
        transform: rotate(15deg) scale(1.1);
    }
</style>

<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let handleError = (error) => {};
  export let clearError = () => {};

  let studySettings = null;
  let originalSettings = null;
  let settingsChanged = false;

  onMount(async () => {
    await loadStudySettings();
  });

  async function loadStudySettings() {
    try {
      studySettings = await invoke('get_study_settings');
      originalSettings = JSON.parse(JSON.stringify(studySettings)); // Deep clone
      settingsChanged = false;
      clearError();
    } catch (error) {
      console.error('Failed to load study settings:', error);
      handleError(`Failed to load study settings: ${error.message || error}`);
    }
  }

  async function saveStudySettings() {
    if (!studySettings || !settingsChanged) return;
    
    try {
      await invoke('save_study_settings', { newSettings: studySettings });
      originalSettings = JSON.parse(JSON.stringify(studySettings));
      settingsChanged = false;
      clearError();
    } catch (error) {
      console.error('Failed to save study settings:', error);
      handleError(`Failed to save study settings: ${error.message || error}`);
    }
  }

  async function resetStudySettings() {
    if (confirm('Are you sure you want to reset all study settings to their default values?')) {
      try {
        await invoke('reset_study_settings_to_defaults');
        await loadStudySettings();
        clearError();
      } catch (error) {
        console.error('Failed to reset study settings:', error);
        handleError(`Failed to reset study settings: ${error.message || error}`);
      }
    }
  }

  function updateStudySetting(settingName, value) {
    if (!studySettings) return;
    
    studySettings[settingName] = value;
    studySettings = { ...studySettings }; // Trigger reactivity
    
    // Check if settings have changed
    settingsChanged = JSON.stringify(studySettings) !== JSON.stringify(originalSettings);
  }

  function updateLearningStep(index, value) {
    if (!studySettings) return;
    
    const numValue = parseInt(value) || 1;
    const steps = [...studySettings.learning_steps];
    
    // Ensure we have at least 2 steps
    while (steps.length < 2) {
      steps.push(1);
    }
    
    steps[index] = numValue;
    updateStudySetting('learning_steps', steps);
  }

  function validateNumber(value, min, max, defaultValue) {
    const num = parseFloat(value);
    if (isNaN(num)) return defaultValue;
    return Math.min(Math.max(num, min), max);
  }
</script>

<div class="tab-content">
  {#if studySettings}
    <div class="settings-section">
      <div class="section-header">
        <h3>General Settings</h3>
        <div class="section-actions">
          <button 
            class="action-button save"
            class:changed={settingsChanged}
            on:click={saveStudySettings}
            disabled={!settingsChanged}
            title="Save changes"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path>
              <polyline points="17 21 17 13 7 13 7 21"></polyline>
              <polyline points="7 3 7 8 15 8"></polyline>
            </svg>
            <span>{settingsChanged ? 'Save Changes' : 'Saved'}</span>
          </button>
          <button 
            class="action-button danger"
            on:click={resetStudySettings}
            title="Reset to defaults"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="1 4 1 10 7 10"></polyline>
              <polyline points="23 20 23 14 17 14"></polyline>
              <path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4-4.64 4.36A9 9 0 0 1 3.51 15"></path>
            </svg>
            <span>Reset</span>
          </button>
        </div>
      </div>

      <div class="settings-grid">
        <!-- Flashcard Learning Settings -->
        <div class="setting-group">
          <h4>Flashcard Learning</h4>
          
          <div class="setting-item">
            <label class="setting-label">
              Learning Steps (minutes)
              <span class="setting-description">Time intervals for learning new cards</span>
            </label>
            <div class="setting-row">
              <div class="setting-item">
                <label class="setting-label">
                  First Step
                  <span class="setting-description">Minutes for first learning review</span>
                </label>
                <input 
                  type="number" 
                  class="setting-input"
                  min="1"
                  max="1440"
                  value={studySettings.learning_steps[0] || 1}
                  on:input={(e) => {
                    const target = e.target as HTMLInputElement;
                    updateLearningStep(0, target.value);
                  }}
                />
              </div>
              <div class="setting-item">
                <label class="setting-label">
                  Second Step
                  <span class="setting-description">Minutes for second learning review</span>
                </label>
                <input 
                  type="number" 
                  class="setting-input"
                  min="1"
                  max="1440"
                  value={studySettings.learning_steps[1] || 10}
                  on:input={(e) => {
                    const target = e.target as HTMLInputElement;
                    updateLearningStep(1, target.value);
                  }}
                />
              </div>
            </div>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              Starting Ease Factor
              <span class="setting-description">Initial ease factor for new cards (1.0 - 5.0)</span>
            </label>
            <input 
              type="number" 
              class="setting-input"
              min="1.0"
              max="5.0"
              step="0.1"
              value={studySettings.starting_ease_factor}
              on:input={(e) => {
                const target = e.target as HTMLInputElement;
                const value = validateNumber(target.value, 1.0, 5.0, 2.5);
                updateStudySetting('starting_ease_factor', value);
              }}
            />
          </div>

          <div class="setting-row">
            <div class="setting-item">
              <label class="setting-label">
                Min Ease Factor
                <span class="setting-description">Minimum allowed ease factor</span>
              </label>
              <input 
                type="number" 
                class="setting-input"
                min="1.0"
                max="5.0"
                step="0.1"
                value={studySettings.min_ease_factor}
                on:input={(e) => {
                  const target = e.target as HTMLInputElement;
                  const value = validateNumber(target.value, 1.0, studySettings.max_ease_factor, 1.3);
                  updateStudySetting('min_ease_factor', value);
                }}
              />
            </div>
            <div class="setting-item">
              <label class="setting-label">
                Max Ease Factor
                <span class="setting-description">Maximum allowed ease factor</span>
              </label>
              <input 
                type="number" 
                class="setting-input"
                min="1.0"
                max="5.0"
                step="0.1"
                value={studySettings.max_ease_factor}
                on:input={(e) => {
                  const target = e.target as HTMLInputElement;
                  const value = validateNumber(target.value, studySettings.min_ease_factor, 5.0, 2.5);
                  updateStudySetting('max_ease_factor', value);
                }}
              />
            </div>
          </div>
        </div>

        <!-- Interval Settings -->
        <div class="setting-group">
          <h4>Review Intervals</h4>
          
          <div class="setting-row">
            <div class="setting-item">
              <label class="setting-label">
                Easy Interval (New Cards)
                <span class="setting-description">Days when marking new card as "easy"</span>
              </label>
              <input 
                type="number" 
                class="setting-input"
                min="1"
                max="365"
                value={studySettings.easy_interval_new}
                on:input={(e) => {
                  const target = e.target as HTMLInputElement;
                  const value = validateNumber(target.value, 1, 365, 4);
                  updateStudySetting('easy_interval_new', Math.floor(value));
                }}
              />
            </div>
            <div class="setting-item">
              <label class="setting-label">
                Easy Interval (Learning Cards)
                <span class="setting-description">Days when marking learning card as "easy"</span>
              </label>
              <input 
                type="number" 
                class="setting-input"
                min="1"
                max="365"
                value={studySettings.easy_interval_learning}
                on:input={(e) => {
                  const target = e.target as HTMLInputElement;
                  const value = validateNumber(target.value, 1, 365, 4);
                  updateStudySetting('easy_interval_learning', Math.floor(value));
                }}
              />
            </div>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              Maximum Interval (days)
              <span class="setting-description">Maximum time between reviews</span>
            </label>
            <input 
              type="number" 
              class="setting-input"
              min="1"
              max="36500"
              value={studySettings.max_interval_days}
              on:input={(e) => {
                const target = e.target as HTMLInputElement;
                const value = validateNumber(target.value, 1, 36500, 365);
                updateStudySetting('max_interval_days', Math.floor(value));
              }}
            />
          </div>
        </div>

        <!-- Difficulty Multipliers -->
        <div class="setting-group">
          <h4>Difficulty Adjustments</h4>
          
          <div class="setting-row">
            <div class="setting-item">
              <label class="setting-label">
                Hard Multiplier
                <span class="setting-description">Interval multiplier for "hard" responses</span>
              </label>
              <input 
                type="number" 
                class="setting-input"
                min="1.0"
                max="2.0"
                step="0.1"
                value={studySettings.hard_interval_multiplier}
                on:input={(e) => {
                  const target = e.target as HTMLInputElement;
                  const value = validateNumber(target.value, 1.0, 2.0, 1.2);
                  updateStudySetting('hard_interval_multiplier', value);
                }}
              />
            </div>
            <div class="setting-item">
              <label class="setting-label">
                Easy Multiplier
                <span class="setting-description">Interval multiplier for "easy" responses</span>
              </label>
              <input 
                type="number" 
                class="setting-input"
                min="1.0"
                max="3.0"
                step="0.1"
                value={studySettings.easy_interval_multiplier}
                on:input={(e) => {
                  const target = e.target as HTMLInputElement;
                  const value = validateNumber(target.value, 1.0, 3.0, 1.3);
                  updateStudySetting('easy_interval_multiplier', value);
                }}
              />
            </div>
          </div>
        </div>

        <!-- Search Settings -->
        <div class="setting-group">
          <h4>Search & Content</h4>
          
          <div class="setting-item">
            <label class="setting-label">
              Max Search Results
              <span class="setting-description">Maximum chunks returned from vector search</span>
            </label>
            <input 
              type="number" 
              class="setting-input"
              min="1"
              max="50"
              value={studySettings.max_chunks_to_recover_from_search}
              on:input={(e) => {
                const target = e.target as HTMLInputElement;
                const value = validateNumber(target.value, 1, 50, 5);
                updateStudySetting('max_chunks_to_recover_from_search', Math.floor(value));
              }}
            />
          </div>
        </div>
      </div>
    </div>
  {:else}
    <div class="loading-state">
      <p>Loading study settings...</p>
    </div>
  {/if}
</div>

<style>
  .tab-content {
    height: 100%;
  }

  .settings-section {
    margin-bottom: var(--space-xl);
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-lg);
    padding-bottom: var(--space-md);
    border-bottom: 1px solid var(--border);
  }

  .section-header h3 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
  }

  .section-actions {
    display: flex;
    gap: var(--space-sm);
  }

  .action-button {
    background: var(--accent);
    color: white;
    border: none;
    padding: var(--space-md);
    border-radius: var(--border-radius);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    transition: all var(--transition-speed) ease;
    min-width: 40px;
    height: 40px;
  }

  .action-button:hover:not(:disabled) {
    filter: brightness(1.1);
    transform: translateY(-1px);
  }

  .action-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  .action-button.save.changed {
    background: var(--success);
    animation: pulse 2s infinite;
  }

  .action-button.danger {
    background: var(--error);
    color: white;
  }

  .settings-grid {
    display: grid;
    gap: var(--space-md);
  }

  .setting-group {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--border-radius);
    padding: var(--space-md);
  }

  .setting-group h4 {
    margin: 0 0 var(--space-md) 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text);
    border-bottom: 1px solid var(--border);
    padding-bottom: var(--space-sm);
  }

  .setting-item {
    margin-bottom: var(--space-md);
  }

  .setting-item:last-child {
    margin-bottom: 0;
  }

  .setting-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-md);
  }

  .setting-label {
    display: block;
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text);
    margin-bottom: var(--space-sm);
  }

  .setting-description {
    display: block;
    font-size: 0.8rem;
    font-weight: 400;
    color: var(--text-secondary);
    margin-top: var(--space-xs);
  }

  .setting-input {
    width: 100%;
    background: var(--background);
    border: 1px solid var(--border);
    padding: var(--space-md);
    border-radius: var(--border-radius);
    color: var(--text);
    font-family: var(--font-body);
    font-size: 0.9rem;
    transition: all var(--transition-speed) ease;
  }

  .setting-input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 20%, transparent);
  }

  .setting-input:invalid {
    border-color: var(--error);
  }

  .loading-state {
    display: flex;
    justify-content: center;
    align-items: center;
    padding: var(--space-xl);
    color: var(--text-secondary);
  }

  @keyframes pulse {
    0% {
      transform: scale(1);
      box-shadow: 0 0 0 0 color-mix(in srgb, var(--success) 50%, transparent);
    }
    70% {
      transform: scale(1.02);
      box-shadow: 0 0 0 8px color-mix(in srgb, var(--success) 0%, transparent);
    }
    100% {
      transform: scale(1);
      box-shadow: 0 0 0 0 color-mix(in srgb, var(--success) 0%, transparent);
    }
  }

  @media (max-width: 768px) {
    .setting-row {
      grid-template-columns: 1fr;
    }
    
    .section-header {
      flex-direction: column;
      align-items: stretch;
      gap: var(--space-md);
    }
    
    .section-actions {
      justify-content: stretch;
    }
    
    .section-actions .action-button {
      flex: 1;
    }
  }
</style>
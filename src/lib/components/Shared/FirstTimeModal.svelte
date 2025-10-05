<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { fade, scale, fly } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';

  import studyCraftLogo from '../../../assets/StudyCraftLogo.png';
  import characterImage from '../../../assets/chara/03.png';

  export let show = false;
  export let onComplete: (() => void) | undefined = undefined;
  
  let currentStep = 'welcome';
  let isDownloading = false;
  let statusMessage = '';
  let error = '';
  let setupProgress = { embedding: false, chromium: false };
  let currentSetupTask = '';
  const steps = [
    {
      id: 'welcome',
      title: 'Welcome to StudyCraft v2!',
      subtitle: 'Beta version is here!',
      description: 'StudyCraft helps you organize study materials, quickly create flashcards and track your study sessions.'
    },
    {
      id: 'downloading',
      title: 'Setting up Your Environment',
      subtitle: 'Preparing your study tool',
      description: 'We\'re downloading the embedding model for semantic search and Chromium for document previews. This only needs to be done once.'
    },
    {
      id: 'complete',
      title: 'Ready to Study!',
      subtitle: 'Everything is set up',
      description: 'Your study environment is ready! Start by uploading study materials and creating some flashcard decks or tests!'
    }
  ];

  $: currentStepData = steps.find(step => step.id === currentStep) || steps[0];

  async function startDownload() {
    currentStep = 'downloading';
    isDownloading = true;
    error = '';
    setupProgress = { embedding: false, chromium: false };
    
    try {
      currentSetupTask = 'embedding';
      statusMessage = 'Setting up embedding model...';
      await invoke('initialize_embedding_model');
      setupProgress.embedding = true;
      
      currentSetupTask = 'chromium';
      statusMessage = 'Downloading Chromium browser...';
      await invoke('initialize_chromium');
      setupProgress.chromium = true;
      
      statusMessage = 'Setup complete!';
      
      setTimeout(() => {
        currentStep = 'complete';
        isDownloading = false;
      }, 1000);
      
    } catch (err) {
      error = `Failed to setup ${currentSetupTask}: ${err}`;
      statusMessage = 'Setup failed';
      isDownloading = false;
      currentStep = 'welcome';
      setupProgress = { embedding: false, chromium: false };
    }
  }

  function handleGetStarted() {
    if (currentStep === 'welcome') {
      startDownload();
    } else if (currentStep === 'complete') {
      show = false;
      if (onComplete) {
        onComplete();
      }
    }
  }

  function handleRetry() {
    error = '';
    setupProgress = { embedding: false, chromium: false };
    currentSetupTask = '';
    startDownload();
  }

  function handleOverlayClick() {
    if (!isDownloading && currentStep !== 'welcome') {
      if (currentStep === 'complete') {
        show = false;
        if (onComplete) {
          onComplete();
        }
      }
    }
  }
</script>

{#if show}
  <div class="ftm-modal-overlay" transition:fade={{ duration: 300 }} on:click={handleOverlayClick}>
    <div class="ftm-modal" transition:scale={{ duration: 400, start: 0.8, easing: quintOut }} on:click|stopPropagation>
      
      <div class="ftm-modal-content" data-step={currentStep}>
        {#key currentStep}
          <div class="ftm-step-content" data-step={currentStep} in:fly={{ x: 50, duration: 300, delay: 100 }} out:fly={{ x: -50, duration: 200 }}>
            
            <div class="ftm-step-illustration">
              {#if currentStep === 'welcome'}
                <div class="ftm-logo-image">
                  <img src={studyCraftLogo} alt="StudyCraft Logo" />
                </div>
              {:else if currentStep === 'downloading'}
                <div class="ftm-downloading-icon">
                  <div class="ftm-spinner-container">
                    <div class="ftm-spinner"></div>
                    <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                      <polyline points="7,10 12,15 17,10"></polyline>
                      <line x1="12" y1="15" x2="12" y2="3"></line>
                    </svg>
                  </div>
                </div>
              {:else if currentStep === 'complete'}
                <div class="ftm-success-icon">
                  <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                    <polyline points="22,4 12,14.01 9,11.01"></polyline>
                  </svg>
                </div>
              {/if}
            </div>

            <div class="ftm-step-text">
              <h2>{currentStepData.title}</h2>
              <p class="ftm-subtitle">{currentStepData.subtitle}</p>
              <p class="ftm-description">{currentStepData.description}</p>
            </div>

            {#if currentStep === 'welcome'}
              <div class="ftm-character-image">
                <img src={characterImage} alt="StudyCraft Character" />
              </div>
            {/if}

            {#if currentStep === 'downloading'}
              <div class="ftm-progress-section">
                {#if error}
                  <div class="ftm-error-message">
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <circle cx="12" cy="12" r="10"></circle>
                      <line x1="15" y1="9" x2="9" y2="15"></line>
                      <line x1="9" y1="9" x2="15" y2="15"></line>
                    </svg>
                    <span>{error}</span>
                  </div>
                {:else}
                  <div class="ftm-progress-container">
                    <div class="ftm-setup-steps">
                      <div class="ftm-setup-step" class:completed={setupProgress.embedding} class:active={currentSetupTask === 'embedding'}>
                        <div class="ftm-step-icon">
                          {#if setupProgress.embedding}
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                              <polyline points="20,6 9,17 4,12"></polyline>
                            </svg>
                          {:else if currentSetupTask === 'embedding'}
                            <div class="ftm-mini-spinner"></div>
                          {:else}
                            <div class="ftm-step-number">1</div>
                          {/if}
                        </div>
                        <span>Embedding Model</span>
                      </div>
                      <div class="ftm-setup-step" class:completed={setupProgress.chromium} class:active={currentSetupTask === 'chromium'}>
                        <div class="ftm-step-icon">
                          {#if setupProgress.chromium}
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                              <polyline points="20,6 9,17 4,12"></polyline>
                            </svg>
                          {:else if currentSetupTask === 'chromium'}
                            <div class="ftm-mini-spinner"></div>
                          {:else}
                            <div class="ftm-step-number">2</div>
                          {/if}
                        </div>
                        <span>Chromium Browser</span>
                      </div>
                    </div>
                  </div>
                  
                  <div class="ftm-status-message">
                    {statusMessage || 'Setting up your AI-powered study environment...'}
                  </div>
                {/if}
              </div>
            {/if}

            {#if currentStep === 'complete'}
              <div class="ftm-getting-started">
                <h4>Quick Start Tips:</h4>
                <ul>
                  <li>Click "Add Study Material" to upload your first document</li>
                  <li>Switch to "Review" mode to create flashcard decks</li>
                  <li>Use the search feature to find content across materials</li>
                </ul>
              </div>
            {/if}
          </div>
        {/key}
      </div>
      
      <div class="ftm-modal-footer">
        {#if currentStep === 'welcome'}
          <button class="ftm-primary-button" on:click={handleGetStarted}>
            Get Started
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="9,18 15,12 9,6"></polyline>
            </svg>
          </button>
        {:else if currentStep === 'downloading'}
          {#if error}
            <button class="ftm-retry-button" on:click={handleRetry}>
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="23,4 23,10 17,10"></polyline>
                <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
              </svg>
              Try Again
            </button>
          {:else}
            <div class="ftm-downloading-status">
              <div class="ftm-status-dot"></div>
              <span>Setting up environment...</span>
            </div>
          {/if}
        {:else if currentStep === 'complete'}
          <button class="ftm-primary-button" on:click={handleGetStarted}>
            Start Using StudyCraft
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="9,18 15,12 9,6"></polyline>
            </svg>
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .ftm-modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
    padding: var(--space-md);
  }

  .ftm-modal {
    background: var(--surface);
    border-radius: var(--border-radius-lg);
    border: 1px solid var(--border);
    width: 100%;
    max-width: 580px;
    max-height: 85vh;
    min-height: 500px;
    overflow: hidden;
    box-shadow: var(--shadow-lg);
    display: flex;
    flex-direction: column;
  }

  .ftm-modal-content {
    flex: 1;
    padding: var(--space-lg);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    min-height: 0;
    width: 100%;
  }

  .ftm-modal-content[data-step="downloading"] {
    justify-content: center;
    align-items: center;
    min-height: 400px; 
  }

  .ftm-step-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-md);
    height: 100%;
    width: 100%;
    padding: 0;
  }

  .ftm-step-content[data-step="welcome"] {
    justify-content: center;
  }

  .ftm-step-content[data-step="downloading"] {
    justify-content: center;
    flex: 1;
    min-height: 300px;
  }

  .ftm-step-content[data-step="complete"] {
    justify-content: flex-start;
    padding-top: var(--space-lg);
  }

  .ftm-step-illustration {
    margin-bottom: var(--space-sm);
    flex-shrink: 0;
  }

  .ftm-logo-image {
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .ftm-logo-image img {
    max-width: 80px;
    max-height: 80px;
    width: auto;
    height: auto;
    object-fit: contain;
  }

  .ftm-downloading-icon {
    position: relative;
  }

  .ftm-spinner-container {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .ftm-spinner {
    position: absolute;
    width: 60px;
    height: 60px;
    border: 2px solid color-mix(in srgb, var(--accent) 20%, transparent);
    border-top: 2px solid var(--accent);
    border-radius: 50%;
    animation: ftm-spin 1.2s linear infinite;
  }

  .ftm-downloading-icon svg {
    color: var(--accent);
    z-index: 1;
  }

  .ftm-success-icon {
    color: var(--success, #22c55e);
    animation: ftm-successPulse 0.6s ease-out;
  }

  .ftm-step-text {
    flex-shrink: 0;
    width: 100%;
    text-align: center;
  }

  .ftm-step-text h2 {
    margin: 0;
    font-family: var(--font-heading);
    font-size: 1.6rem;
    font-weight: 600;
    color: var(--text);
    line-height: 1.2;
  }

  .ftm-subtitle {
    margin: var(--space-xs) 0;
    font-size: 1rem;
    color: var(--accent);
    font-weight: 500;
  }

  .ftm-description {
    margin: 0;
    font-size: 0.95rem;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .ftm-character-image {
    margin-top: var(--space-md);
    flex-shrink: 0;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .ftm-character-image img {
    max-width: 200px;
    max-height: 200px;
    width: auto;
    height: auto;
    object-fit: contain;
    border-radius: var(--border-radius);
    opacity: 0.9;
    transition: opacity var(--transition-speed) ease;
  }

  .ftm-character-image img:hover {
    opacity: 1;
  }

  .ftm-progress-section {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    align-items: center;
  }

  .ftm-progress-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    align-items: center;
    width: 100%;
  }

  .ftm-setup-steps {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    width: 100%;
    max-width: 300px;
    margin-top: var(--space-lg);
    margin-bottom: var(--space-sm);
  }

  .ftm-setup-step {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm);
    border-radius: var(--border-radius);
    background: color-mix(in srgb, var(--surface) 95%, var(--border));
    border: 1px solid var(--border);
    transition: all var(--transition-speed) ease;
  }

  .ftm-setup-step.active {
    background: color-mix(in srgb, var(--accent) 10%, var(--surface));
    border-color: var(--accent);
  }

  .ftm-setup-step.completed {
    background: color-mix(in srgb, var(--success, #22c55e) 10%, var(--surface));
    border-color: var(--success, #22c55e);
  }

  .ftm-step-icon {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .ftm-step-number {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--border);
    color: var(--text-secondary);
    font-size: 0.75rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .ftm-setup-step.active .ftm-step-number {
    background: var(--accent);
    color: var(--surface);
  }

  .ftm-setup-step.completed .ftm-step-icon svg {
    color: var(--success, #22c55e);
  }

  .ftm-mini-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid color-mix(in srgb, var(--accent) 30%, transparent);
    border-top: 2px solid var(--accent);
    border-radius: 50%;
    animation: ftm-spin 1s linear infinite;
  }

  .ftm-setup-step span {
    font-size: 0.9rem;
    color: var(--text);
    font-weight: 500;
  }

  .ftm-status-message {
    font-size: 0.9rem;
    color: var(--text-secondary);
    text-align: center;
    line-height: 1.4;
  }

  .ftm-error-message {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    color: var(--error, #ef4444);
    background: color-mix(in srgb, var(--error, #ef4444) 10%, var(--surface));
    padding: var(--space-md);
    border-radius: var(--border-radius);
    border: 1px solid color-mix(in srgb, var(--error, #ef4444) 25%, var(--border));
    width: 100%;
    justify-content: center;
  }

  .ftm-getting-started {
    text-align: left;
    width: 100%;
    margin-top: var(--space-sm);
  }

  .ftm-getting-started h4 {
    margin: 0 0 var(--space-sm) 0;
    font-family: var(--font-heading);
    font-size: 1rem;
    color: var(--text);
    font-weight: 500;
  }

  .ftm-getting-started ul {
    margin: 0;
    padding-left: var(--space-md);
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .ftm-getting-started li {
    margin-bottom: var(--space-xs);
    font-size: 0.9rem;
  }

  .ftm-modal-footer {
    padding: var(--space-md) var(--space-lg);
    border-top: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    justify-content: center;
    flex-shrink: 0;
  }

  .ftm-primary-button {
    background: var(--accent);
    color: var(--text);
    border: none;
    padding: var(--space-sm) var(--space-lg);
    border-radius: var(--border-radius);
    font-family: var(--font-body);
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition-speed) ease;
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .ftm-primary-button:hover {
    background: color-mix(in srgb, var(--accent) 90%, black);
    transform: translateY(-1px);
  }

  .ftm-retry-button {
    background: var(--error, #ef4444);
    color: white;
    border: none;
    padding: var(--space-sm) var(--space-lg);
    border-radius: var(--border-radius);
    font-family: var(--font-body);
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition-speed) ease;
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .ftm-retry-button:hover {
    background: color-mix(in srgb, var(--error, #ef4444) 90%, black);
    transform: translateY(-1px);
  }

  .ftm-downloading-status {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    color: var(--text-secondary);
  }

  .ftm-status-dot {
    width: 8px;
    height: 8px;
    background: var(--accent);
    border-radius: 50%;
    animation: ftm-pulse 2s infinite;
  }

  @keyframes ftm-spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  @keyframes ftm-successPulse {
    0% { transform: scale(0.8); opacity: 0; }
    50% { transform: scale(1.1); }
    100% { transform: scale(1); opacity: 1; }
  }

  @keyframes ftm-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  @media (max-width: 640px) {
    .ftm-modal-overlay {
      padding: var(--space-sm);
    }

    .ftm-modal {
      min-height: 450px;
    }

    .ftm-modal-content,
    .ftm-modal-footer {
      padding: var(--space-md);
    }

    .ftm-step-text h2 {
      font-size: 1.4rem;
    }

    .ftm-character-image img {
      max-width: 150px;
      max-height: 150px;
    }
  }
</style>
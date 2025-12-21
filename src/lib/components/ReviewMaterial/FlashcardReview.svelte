<script lang="ts">
import { onMount, onDestroy } from 'svelte';
import { fade, fly, scale, slide } from 'svelte/transition';
import { elasticOut, cubicOut } from 'svelte/easing';
import { invoke } from '@tauri-apps/api/core';
import "../../styles/variables.css";

// Components
import Button from '../Shared/Button.svelte';

// Logic
import { FlashcardReviewLogic, type Card } from '../../logic/ReviewMaterial/flashcardReview';

// Stores
import { llmStore } from '../../stores/llmStore';
import { toastStore } from '../../stores/toastStore';

// Props
export let deck = null;
export let isOpen = false;

// Callback props
export let onClose = () => {};
export let onReviewCompleted = (data) => {};

// State
let reviewLogic = null;
let currentCard = null;
let isFlipped = false;
let isLoading = true;
let studyComplete = false;
let error = '';
let confetti = false;
let showAnswer = false;
let isExplaining = false;
let explanation = '';
let reviewSettings = null;
let showSettings = false;
let tempSettings = {
  shuffle_deck: true,
  use_only_pending: true
};
let deckStats = {
  new_count: 0,
  learning_count: 0,
  review_count: 0,
  pending_count: 0,
  total_count: 0
};
let studyTimer;
let studyStartTime = null;
let studyTime = 0;
let sessionStartTime = null;
let cardStartTime = null;
let cardResponseTimes = [];
let progress = { current: 0, total: 0, percentage: 0 };
let session = null;
let isAnimating = false;
let cardDirection = 0; // -1 for left, 1 for right, 0 for none
let keyboardEnabled = true;
let isSubmittingRating = false; // Lock to prevent concurrent rating submissions
let confettiTimeoutId: ReturnType<typeof setTimeout> | null = null;

// Reactivity
$: llmStatus = $llmStore;
$: canUseAI = llmStatus.hasConfiguredProvider && !llmStatus.isLoading;
$: deckName = deck && deck.name ? deck.name : 'Flashcard Review';
$: if (deck && isOpen && !reviewSettings) {
  showSettings = true;
  loadDeckStats();
}
$: if (deck && isOpen && reviewSettings) {
  loadCards();
}

// Functions
async function loadDeckStats() {
  if (!deck?.id) return;
  
  try {
    const stats: typeof deckStats = await invoke('get_flashcard_counts', { deckId: deck.id });
    deckStats = stats;
  } catch (error) {
    console.error('Failed to load deck stats:', error);
    console.error('Error details:', error.message);
  }
}

async function loadCards() {
  isLoading = true;
  studyComplete = false;
  error = '';
  
  try {
    if (deck && deck.id && reviewSettings) {
      console.log('Loading cards for deck:', deck.id);
      
      const loadedCards: Card[] = await invoke('get_flashcards_for_review', { 
        deckId: deck.id,
        settings: reviewSettings
      });
      
      if (loadedCards && loadedCards.length > 0) {
        loadedCards.forEach((card, index) => {
          console.log(`Card ${index}:`, {
            id: card.id,
            idType: typeof card.id,
            front: card.front?.substring(0, 50) + '...',
            back: card.back?.substring(0, 50) + '...',
            status: card.status
          });
          
          if (card.id == null || card.id === undefined) {
            console.error(`ERROR: Card ${index} has null/undefined ID:`, card);
          }
          if (typeof card.id !== 'number') {
            console.warn(`WARNING: Card ${index} ID is not a number:`, card.id, typeof card.id);
            if (typeof card.id === 'string' && !isNaN(Number(card.id))) {
              card.id = Number(card.id);
            }
          }
        });
        
        reviewLogic = new FlashcardReviewLogic(loadedCards, reviewSettings);
        updateDisplayState();
        startStudySession();
      } else {
        error = 'No cards are available for review with the current settings.';
      }
    } else {
      throw new Error('Invalid deck information or missing settings');
    }
  } catch (err) {
    console.error('Failed to load flashcards:', err);
    console.error('Error stack:', err.stack);
    error = `Failed to load flashcards: ${err.message || 'Unknown error'}`;
  } finally {
    isLoading = false;
  }
}

function handleSettingsConfirm(settings) {
  reviewSettings = settings;
  showSettings = false;
}

function handleSettingsClose() {
  showSettings = false;
  if (!reviewSettings) {
    handleClose();
  }
}

function updateDisplayState() {
  if (reviewLogic) {
    const newCard = reviewLogic.getCurrentCard();
    const newProgress = reviewLogic.getProgress();
    const newSession = reviewLogic.getSession();
    
    currentCard = newCard;
    progress = newProgress;
    session = newSession;
    
    if (currentCard && (currentCard.id == null || typeof currentCard.id !== 'number')) {
      console.error('ERROR: Current card has invalid ID:', currentCard);
    }
  }
}

function startStudySession() {
  sessionStartTime = new Date();
  studyStartTime = Date.now();
  studyTime = 0;
  cardResponseTimes = [];
  startCardTimer();
  
  studyTimer = setInterval(() => {
    studyTime = Math.floor((Date.now() - studyStartTime) / 1000);
    if (reviewLogic) {
      reviewLogic.updateSessionTime(studyTime);
    }
  }, 1000);
}

function stopStudySession() {
  if (studyTimer) {
    clearInterval(studyTimer);
    studyTimer = null;
  }
}

function startCardTimer() {
  cardStartTime = Date.now();
}

function recordCardResponseTime() {
  if (cardStartTime) {
    const responseTime = (Date.now() - cardStartTime) / 1000;
    cardResponseTimes.push(responseTime);
    startCardTimer();
  }
}

function flipCard() {
  if (!isAnimating) {
    if (!showAnswer) {
      showAnswer = true;
      isFlipped = true;
    } else {
      showAnswer = false;
      isFlipped = false;
      explanation = '';
      isExplaining = false;
    }
  }
}

async function handleExplain() {
  if (!currentCard || isExplaining) return;
  
  if (!canUseAI) {
    explanation = 'No AI provider configured. Please configure an AI provider in Settings to use the explanation feature.';
    return;
  }
  
  isExplaining = true;
  explanation = '';
  
  try {
    const result: string = await invoke('explain_flashcard', {
      question: currentCard.front,
      answer: currentCard.back
    });
    explanation = result;
  } catch (err) {
    console.error('Failed to get explanation:', err);
    explanation = 'Sorry, I could not generate an explanation at this time. Please try again later.';
  } finally {
    isExplaining = false;
  }
}

async function handleRating(difficulty: string) {
  // Comprehensive guard against concurrent submissions
  if (!reviewLogic || !showAnswer || isAnimating || !currentCard || isSubmittingRating) return;

  if (currentCard.id == null || currentCard.id === undefined) {
    console.error('ERROR: Current card ID is null or undefined!', currentCard);
    toastStore.error('Invalid card data. Please try again.');
    return;
  }

  if (typeof currentCard.id !== 'number' || isNaN(currentCard.id) || currentCard.id <= 0) {
    console.error('ERROR: Invalid card ID:', currentCard.id, typeof currentCard.id);
    toastStore.error('Invalid card ID. Please try again.');
    return;
  }

  // Set submission lock
  isSubmittingRating = true;
  recordCardResponseTime();

  keyboardEnabled = false;
  isAnimating = true;
  cardDirection = 1;

  const isCurrentlyLastCard = !reviewLogic.canMoveNext();

  try {
    reviewLogic.reviewCard(difficulty);

    const payload = {
      cardId: currentCard.id,
      difficulty: difficulty
    };
    const updatedCard = await invoke('update_flashcard_after_review', payload);

    reviewLogic.updateCard(currentCard.id, updatedCard);

    setTimeout(() => {
      if (isCurrentlyLastCard) {
        completeStudy();
      } else if (reviewLogic && reviewLogic.canMoveNext()) {
        reviewLogic.moveNext();
        updateDisplayState();
        resetCardState();
      } else {
        completeStudy();
      }

      setTimeout(() => {
        keyboardEnabled = true;
        isAnimating = false;
        cardDirection = 0;
        isSubmittingRating = false;
      }, 100);
    }, 300);

  } catch (err: unknown) {
    const errorMessage = err instanceof Error ? err.message : 'Unknown error';
    console.error('Failed to update card:', err);
    toastStore.warning(`Card progress may not be saved: ${errorMessage}`);

    // Still proceed to next card even on error
    setTimeout(() => {
      if (isCurrentlyLastCard) {
        completeStudy();
      } else if (reviewLogic && reviewLogic.canMoveNext()) {
        reviewLogic.moveNext();
        updateDisplayState();
        resetCardState();
      } else {
        completeStudy();
      }

      setTimeout(() => {
        keyboardEnabled = true;
        isAnimating = false;
        cardDirection = 0;
        isSubmittingRating = false;
      }, 100);
    }, 300);
  }
}

function resetCardState() {
  showAnswer = false;
  isFlipped = false;
  explanation = '';
  isExplaining = false;
}

function handlePrevious() {
  if (reviewLogic && reviewLogic.canMovePrevious() && !isAnimating) {
    isAnimating = true;
    cardDirection = -1;
    
    setTimeout(() => {
      reviewLogic.movePrevious();
      updateDisplayState();
      resetCardState();
      
      setTimeout(() => {
        isAnimating = false;
        cardDirection = 0;
      }, 100);
    }, 150);
  }
}

function handleKeyboard(event) {
  const target = event.target;
  const isTypingInInput = target.tagName === 'INPUT' || 
                         target.tagName === 'TEXTAREA' || 
                         target.hasAttribute('contenteditable') ||
                         target.isContentEditable;
  
  if (isTypingInInput) {
    return;
  }
  
  if (!keyboardEnabled || isAnimating) return;
  
  // Prevent default for all our handled keys
  const handledKeys = [' ', 'Enter', '1', '2', '3', '4', 'e', 'E', 'ArrowLeft'];
  if (handledKeys.includes(event.key)) {
    event.preventDefault();
    event.stopPropagation();
  }
  
  switch (event.key) {
    case ' ':
    case 'Enter':
      flipCard();
      break;
    case '1':
      if (showAnswer) handleRating('again');
      break;
    case '2':
      if (showAnswer) handleRating('hard');
      break;
    case '3':
      if (showAnswer) handleRating('good');
      break;
    case '4':
      if (showAnswer) handleRating('easy');
      break;
    case 'e':
    case 'E':
      if (showAnswer && canUseAI) {
        console.log('Explaining...');
        handleExplain();
      }
      break;
    case 'ArrowLeft':
      handlePrevious();
      break;
  }
}

async function completeStudy() {
  stopStudySession();
  
  if (deck && deck.id && reviewLogic && sessionStartTime) {
    try {
      const sessionStats = reviewLogic.getSession();
      session = sessionStats; 
      console.log('Final session stats for completion:', sessionStats);
      
      const sessionEndTime = new Date();
      const averageResponseTime = cardResponseTimes.length > 0 
        ? cardResponseTimes.reduce((a, b) => a + b, 0) / cardResponseTimes.length 
        : 0;
      
      const reviewSessionRequest = {
        material_type: 'flashcard_deck',  
        material_id: deck.id,
        session_start: sessionStartTime.toISOString(),
        session_end: sessionEndTime.toISOString(),
        total_duration_seconds: studyTime,
        cards_studied: sessionStats.cardsStudied,
        again_count: sessionStats.againCount,
        hard_count: sessionStats.hardCount,
        good_count: sessionStats.goodCount,
        easy_count: sessionStats.easyCount,
        new_cards_count: sessionStats.newCards,
        learning_cards_count: sessionStats.learningCards,
        review_cards_count: sessionStats.reviewCards,
        retention_rate: reviewLogic.getRetentionRate(),
        average_response_time_seconds: averageResponseTime,
        completed: true
      };
      
      console.log('Saving review session:', reviewSessionRequest);
      
      const sessionId = await invoke('create_review_session', { 
        request: reviewSessionRequest 
      });
      
      await invoke('update_last_review', { 
        materialId: deck.id 
      });
      
      console.log('Review session saved successfully with ID:', sessionId);
      
      onReviewCompleted({
        deckId: deck.id,
        statistics: sessionStats,
        reviewSession: reviewSessionRequest,
        sessionId: sessionId
      });
      
      // Show confetti animation with cleanup tracking
      confetti = true;
      if (confettiTimeoutId) {
        clearTimeout(confettiTimeoutId);
      }
      confettiTimeoutId = setTimeout(() => {
        confetti = false;
        confettiTimeoutId = null;
      }, 5000);

      studyComplete = true;
      
    } catch (err) {
      console.error('Failed to save review session:', err);
      console.error('Error stack:', err.stack);
      
      const sessionStats = reviewLogic.getSession();
      session = sessionStats;
      studyComplete = true;
      
      onReviewCompleted({
        deckId: deck.id,
        statistics: sessionStats,
        error: err.message
      });
    }
  } else {
    console.log('Completing study without saving session - missing data:', {
      deckExists: !!deck,
      deckId: deck?.id,
      reviewLogicExists: !!reviewLogic,
      sessionStartExists: !!sessionStartTime
    });
    
    if (reviewLogic) {
      const sessionStats = reviewLogic.getSession();
      session = sessionStats;
    }
    studyComplete = true;
    
    onReviewCompleted({
      deckId: deck?.id,
      statistics: session
    });
  }
}

function handleClose() {
  stopStudySession();
  reviewSettings = null;
  showSettings = false;
  onClose();
}

function restartStudy() {
  if (reviewLogic) {
    reviewLogic.reset();
    updateDisplayState();
    resetCardState();
    studyComplete = false;
    reviewSettings = null;
    showSettings = true;
    loadDeckStats();
  }
}

function getCardTypeColor(card) {
  if (!card) return '';
  
  switch (card.status) {
    case 'new': return 'var(--accent)';
    case 'learning': return 'var(--warning)';
    case 'review': return 'var(--success)';
    default: return 'var(--text-secondary)';
  }
}

function getNextDueDisplay(card) {
  if (!reviewLogic || !card) return '';
  return reviewLogic.getNextDueTime(card);
}

onMount(() => {
  window.addEventListener('keydown', handleKeyboard, { capture: true });
  
  return () => {
    window.removeEventListener('keydown', handleKeyboard, { capture: true });
    stopStudySession();
  };
});

onDestroy(() => {
  // Clean up timer
  stopStudySession();

  // Clean up confetti timeout
  if (confettiTimeoutId) {
    clearTimeout(confettiTimeoutId);
    confettiTimeoutId = null;
  }

  // Reset state locks
  isSubmittingRating = false;
  isAnimating = false;
});
</script>

{#if isOpen}
<div class="fr-review-overlay" transition:fade={{ duration: 200 }}>
  <div class="fr-review-container" transition:slide={{ duration: 300, easing: cubicOut }}>
    {#if showSettings}
      <div class="fr-settings-overlay">
        <div class="fr-settings-modal">
          <div class="fr-settings-header">
            <h3>Review Settings</h3>
            <button class="fr-close-button" aria-label="Close" on:click={handleSettingsClose}>
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>
          
          <div class="fr-settings-content">
            <p class="fr-deck-name">Deck: {deckName}</p>
            
            <!-- Deck Statistics -->
            <div class="fr-deck-stats">
              <div class="fr-stat-box fr-new">
                <span class="fr-stat-number">{deckStats.new_count}</span>
                <span class="fr-stat-label">New</span>
              </div>
              <div class="fr-stat-box fr-learning">
                <span class="fr-stat-number">{deckStats.learning_count}</span>
                <span class="fr-stat-label">Learning</span>
              </div>
              <div class="fr-stat-box fr-review">
                <span class="fr-stat-number">{deckStats.review_count}</span>
                <span class="fr-stat-label">Review</span>
              </div>
              <div class="fr-stat-box fr-pending">
                <span class="fr-stat-number">{deckStats.pending_count}</span>
                <span class="fr-stat-label">Pending</span>
              </div>
            </div>
            
            <div class="fr-settings-options">
              <label class="fr-setting-item">
                <input 
                  type="checkbox" 
                  bind:checked={tempSettings.shuffle_deck}
                />
                <span class="fr-setting-label">Shuffle deck</span>
                <span class="fr-setting-description">Randomize card order during review</span>
              </label>
              
              <label class="fr-setting-item">
                <input 
                  type="checkbox" 
                  bind:checked={tempSettings.use_only_pending}
                />
                <span class="fr-setting-label">Use only pending flashcards</span>
                <span class="fr-setting-description">Review only cards that are due</span>
              </label>
            </div>
            
            <!-- Estimated Cards -->
            <div class="fr-estimated-cards">
              <span class="fr-estimated-label">Cards to review:</span>
              <span class="fr-estimated-number">
                {tempSettings.use_only_pending ? deckStats.pending_count : deckStats.total_count}
              </span>
            </div>
            
            <div class="fr-settings-actions">
              <button class="fr-settings-cancel" on:click={handleSettingsClose}>
                Cancel
              </button>
              <button 
                class="fr-settings-confirm" 
                on:click={() => handleSettingsConfirm(tempSettings)}
                disabled={tempSettings.use_only_pending && deckStats.pending_count === 0}
              >
                Start Review
              </button>
            </div>
          </div>
        </div>
      </div>
    {:else}
      <!-- Header -->
      <div class="fr-review-header">
        <div class="fr-deck-info">
          <h2>{deckName}</h2>
          <div class="fr-deck-meta">
            <span>{progress.total} cards</span>
            {#if !studyComplete && session}
              <span>Time: {FlashcardReviewLogic.formatStudyTime(studyTime)}</span>
              <span>Retention: {reviewLogic ? reviewLogic.getRetentionRate() : 0}%</span>
            {/if}
          </div>
        </div>
        
        <button class="fr-close-button" aria-label="Close" on:click={handleClose}>
          <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
      
      {#if !studyComplete}
        <!-- Progress Bar -->
        <div class="fr-progress-bar-container">
          <div class="fr-progress-bar" style="width: {progress.percentage}%"></div>
        </div>
        
        <!-- Progress Info -->
        <div class="fr-progress-info">
          <div class="fr-progress-text">
            <span>Card {progress.current} of {progress.total}</span>
            {#if currentCard && reviewLogic}
              <span class="fr-card-type" style="color: {reviewLogic.getCardTypeColor(currentCard)}">
                {reviewLogic.getCardTypeLabel(currentCard)}
              </span>
            {/if}
          </div>
          
          {#if session}
            <div class="fr-session-stats">
              <span class="fr-stat-item fr-again">{session.againCount}</span>
              <span class="fr-stat-item fr-hard">{session.hardCount}</span>
              <span class="fr-stat-item fr-good">{session.goodCount}</span>
              <span class="fr-stat-item fr-easy">{session.easyCount}</span>
            </div>
          {/if}
        </div>
      {/if}
      
      <!-- Main Content -->
      <div class="fr-review-content">
        {#if isLoading}
          <div class="fr-loading-state" in:fade>
            <div class="fr-loading-spinner"></div>
            <span>Loading flashcards...</span>
          </div>
        {:else if error}
          <div class="fr-error-state" in:fade>
            <div class="fr-error-icon">⚠️</div>
            <h3>Error</h3>
            <p>{error}</p>
            <Button variant="primary" text="Close" onClick={handleClose}/>
          </div>
        {:else if studyComplete}
          <div class="fr-complete-state" in:fade>
            <div class="fr-complete-icon">
              <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                <polyline points="22 4 12 14.01 9 11.01"></polyline>
              </svg>
            </div>
            <h3>Study Session Complete!</h3>
            
            {#if session}
              <div class="fr-completion-stats">
                <div class="fr-main-stats">
                  <div class="fr-stat-card">
                    <span class="fr-stat-value">{session.cardsStudied}</span>
                    <span class="fr-stat-label">Cards Studied</span>
                  </div>
                  <div class="fr-stat-card">
                    <span class="fr-stat-value">{FlashcardReviewLogic.formatStudyTime(studyTime)}</span>
                    <span class="fr-stat-label">Study Time</span>
                  </div>
                  <div class="fr-stat-card">
                    <span class="fr-stat-value">{reviewLogic ? reviewLogic.getRetentionRate() : 0}%</span>
                    <span class="fr-stat-label">Retention</span>
                  </div>
                </div>
                
                <div class="fr-rating-breakdown">
                  <div class="fr-rating-stat fr-again">
                    <span class="fr-rating-count">{session.againCount}</span>
                    <span class="fr-rating-label">Again</span>
                  </div>
                  <div class="fr-rating-stat fr-hard">
                    <span class="fr-rating-count">{session.hardCount}</span>
                    <span class="fr-rating-label">Hard</span>
                  </div>
                  <div class="fr-rating-stat fr-good">
                    <span class="fr-rating-count">{session.goodCount}</span>
                    <span class="fr-rating-label">Good</span>
                  </div>
                  <div class="fr-rating-stat fr-easy">
                    <span class="fr-rating-count">{session.easyCount}</span>
                    <span class="fr-rating-label">Easy</span>
                  </div>
                </div>
              </div>
            {/if}
            
            <div class="fr-complete-actions">
              <Button variant="secondary" text="Close" onClick={handleClose}/>
              <Button variant="primary" text="Study Again" onClick={restartStudy}/>
            </div>
            
            {#if confetti}
              <div class="fr-confetti-container">
                {#each Array(50) as _, i}
                  <div 
                    class="fr-confetti" 
                    style="
                      --fall-delay: {Math.random() * 3}s; 
                      --fall-distance: {50 + Math.random() * 40}vh; 
                      --fall-speed: {2 + Math.random() * 3}s; 
                      --left-pos: {Math.random() * 100}%; 
                      --size: {4 + Math.random() * 8}px; 
                      --hue: {Math.random() * 360}deg;
                    "
                    in:scale={{ duration: 300, delay: Math.random() * 500, easing: elasticOut }}
                  ></div>
                {/each}
              </div>
            {/if}
          </div>
        {:else if currentCard}
          <!-- Flashcard Study Interface -->
          <div class="fr-study-interface">
            <!-- Main Card Display -->
            <div class="fr-card-section">
              <div
                  class="fr-flashcard"
                  class:fr-flipped={isFlipped}
                  class:fr-animating={isAnimating}
                  on:click={flipCard}
                  on:keydown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      flipCard();
                    }
                  }}
                  role="button"
                  tabindex="0"
                  aria-label="Flashcard - click or press Enter/Space to flip"
                  in:fly={{
                    x: cardDirection * 300,
                    duration: isAnimating ? 300 : 0,
                    easing: cubicOut
                  }}
                >
                <!-- Front of Card (Question) -->
                <div class="fr-card-face fr-card-front">
                  <div class="fr-card-content">
                    <div class="fr-card-text" class:fr-long-text={currentCard.front.length > 100}>
                      {currentCard.front}
                    </div>
                  </div>
                  <div class="fr-card-footer">
                    <span class="fr-card-instructions">Click to reveal answer</span>
                    <span class="fr-keyboard-hint">Space/Enter</span>
                  </div>
                </div>
                
                <!-- Back of Card (Answer) -->
                <div class="fr-card-face fr-card-back">
                  <div class="fr-card-content">
                    <div class="fr-card-text fr-answer-text" class:fr-long-text={currentCard.back.length > 150}>
                      {currentCard.back}
                    </div>
                  </div>
                  <div class="fr-card-footer">
                    <span class="fr-card-instructions">Rate your knowledge</span>
                    <span class="fr-keyboard-hint">1-4</span>
                  </div>
                </div>
              </div>
            </div>
            
            <!-- Explanation Section (when available) -->
            {#if explanation && showAnswer}
              <div class="fr-explanation-panel" in:fade={{ duration: 300 }}>
                <div class="fr-explanation-header">
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="10"></circle>
                    <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
                    <path d="m12 17l.01 0"></path>
                  </svg>
                  <span>AI Explanation</span>
                </div>
                <div class="fr-explanation-content">
                  {explanation}
                </div>
              </div>
            {/if}
            
            <!-- Control Section -->
            <div class="fr-control-section">
              <!-- Navigation Controls -->
              <div class="fr-navigation-controls">
                <button 
                  class="fr-nav-button fr-previous" 
                  on:click|stopPropagation={handlePrevious}
                  disabled={!reviewLogic || !reviewLogic.canMovePrevious() || isAnimating}
                  title="Previous card (←)"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="19" y1="12" x2="5" y2="12"></line>
                    <polyline points="12 19 5 12 12 5"></polyline>
                  </svg>
                  <span>Previous</span>
                </button>
                
                <button 
                  class="fr-nav-button fr-flip" 
                  on:click|stopPropagation={flipCard}
                  disabled={isAnimating}
                  title="Flip card (Space)"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 2v6h-6"></path>
                    <path d="M3 12a9 9 0 0 1 15-6.7L21 8"></path>
                    <path d="M3 22v-6h6"></path>
                    <path d="M21 12a9 9 0 0 1-15 6.7L3 16"></path>
                  </svg>
                  <span>{showAnswer ? 'Question' : 'Answer'}</span>
                </button>
                
                {#if showAnswer}
                  <div class="fr-explain-button-wrapper" title={!canUseAI ? 'Configure an AI provider in Settings to use explanations' : ''}>
                    <button 
                      class="fr-nav-button fr-explain" 
                      on:click|stopPropagation={handleExplain}
                      disabled={isExplaining || !canUseAI}
                      title="Get AI explanation (E)"
                    >
                      {#if isExplaining}
                        <div class="fr-mini-spinner"></div>
                      {:else}
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <circle cx="12" cy="12" r="10"></circle>
                          <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
                          <line x1="12" y1="17" x2="12.01" y2="17"></line>
                        </svg>
                      {/if}
                      <span>{isExplaining ? 'Explaining...' : 'Explain'}</span>
                    </button>
                  </div>
                {/if}
              </div>
              
              <!-- Rating Buttons (when answer is shown) -->
              {#if showAnswer}
                <div class="fr-rating-section">
                  <p class="fr-rating-prompt">How well did you know this?</p>
                  <div class="fr-rating-buttons">
                    <button 
                      class="fr-rating-button fr-again" 
                      on:click|stopPropagation={() => handleRating('again')}
                      title="Show again soon (1)"
                    >
                      <span class="fr-rating-label">Again</span>
                      <span class="fr-rating-key">1</span>
                    </button>
                    <button 
                      class="fr-rating-button fr-hard" 
                      on:click|stopPropagation={() => handleRating('hard')}
                      title="Show again with reduced interval (2)"
                    >
                      <span class="fr-rating-label">Hard</span>
                      <span class="fr-rating-key">2</span>
                    </button>
                    <button 
                      class="fr-rating-button fr-good" 
                      on:click|stopPropagation={() => handleRating('good')}
                      title="Show answer with normal interval (3)"
                    >
                      <span class="fr-rating-label">Good</span>
                      <span class="fr-rating-key">3</span>
                    </button>
                    <button 
                      class="fr-rating-button fr-easy" 
                      on:click|stopPropagation={() => handleRating('easy')}
                      title="Show again with longer interval (4)"
                    >
                      <span class="fr-rating-label">Easy</span>
                      <span class="fr-rating-key">4</span>
                    </button>
                  </div>
                </div>
              {/if}
              
              <!-- Keyboard Shortcuts Hint -->
              <div class="fr-keyboard-shortcuts">
                <span class="fr-shortcut-hint">
                  Space (flip) • 1-4 (rate) • {canUseAI ? 'E (explain) • ' : ''}← (previous)
                  {#if !canUseAI}
                    <br><small style="color: var(--text-secondary); font-style: italic;">Configure AI provider in Settings to enable explanations</small>
                  {/if}
                </span>
              </div>
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
{/if}

<style>
  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @keyframes confetti-fall {
    0% {
      transform: translateY(-10px) rotate(0deg);
      opacity: 1;
    }
    100% {
      transform: translateY(var(--fall-distance)) rotate(720deg);
      opacity: 0;
    }
  }

  .fr-review-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(4px);
    padding: var(--space-md);
  }

  .fr-review-container {
    background: var(--surface);
    width: 100%;
    max-width: 900px;
    height: 100%;
    max-height: 95vh;
    border-radius: var(--border-radius);
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 1px solid var(--border);
  }

  .fr-settings-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
  }

  .fr-settings-modal {
    background: var(--surface);
    border-radius: var(--border-radius);
    padding: 0;
    max-width: 500px;
    width: 90%;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
    border: 1px solid var(--border);
  }

  .fr-settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-md) var(--space-md) 0 var(--space-md);
    border-bottom: 1px solid var(--border);
    margin-bottom: var(--space-lg);
  }

  .fr-settings-header h3 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
    font-family: var(--font-heading);
  }

  .fr-settings-content {
    padding: 0 var(--space-lg) var(--space-lg) var(--space-lg);
  }

  .fr-deck-name {
    font-size: 1rem;
    font-weight: 500;
    color: var(--text-secondary);
    margin-bottom: var(--space-lg);
    text-align: center;
  }

  .fr-deck-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-sm);
    margin-bottom: var(--space-lg);
  }

  .fr-stat-box {
    background: var(--background);
    border-radius: var(--border-radius);
    padding: var(--space-md) var(--space-sm);
    text-align: center;
    border: 2px solid var(--border);
    transition: all 0.2s ease;
  }

  .fr-stat-box.fr-new {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 8%, var(--background));
  }

  .fr-stat-box.fr-learning {
    border-color: var(--warning);
    background: color-mix(in srgb, var(--warning) 8%, var(--background));
  }

  .fr-stat-box.fr-review {
    border-color: var(--success);
    background: color-mix(in srgb, var(--success) 8%, var(--background));
  }

  .fr-stat-box.fr-pending {
    border-color: var(--error);
    background: color-mix(in srgb, var(--error) 8%, var(--background));
  }

  .fr-stat-number {
    display: block;
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text);
    margin-bottom: var(--space-xs);
  }

  .fr-stat-label {
    display: block;
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .fr-settings-options {
    margin-bottom: var(--space-lg);
  }

  .fr-setting-item {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
    margin-bottom: var(--space-md);
    cursor: pointer;
    padding: var(--space-md);
    border-radius: var(--border-radius);
    transition: background-color 0.2s ease;
  }

  .fr-setting-item:hover {
    background: color-mix(in srgb, var(--accent) 5%, var(--background));
  }

  .fr-setting-item input[type="checkbox"] {
    margin-right: var(--space-sm);
    width: 18px;
    height: 18px;
    accent-color: var(--accent);
  }

  .fr-setting-label {
    font-weight: 500;
    color: var(--text);
    display: flex;
    align-items: center;
    font-size: 0.95rem;
  }

  .fr-setting-description {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-left: 30px;
    line-height: 1.4;
  }

  .fr-estimated-cards {
    background: var(--background);
    border-radius: var(--border-radius);
    padding: var(--space-sm);
    text-align: center;
    margin-bottom: var(--space-lg);
    border: 1px solid var(--border);
  }

  .fr-estimated-label {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-right: var(--space-xs);
  }

  .fr-estimated-number {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--accent);
  }

  .fr-settings-actions {
    display: flex;
    gap: var(--space-sm);
    justify-content: flex-end;
  }

  .fr-settings-cancel {
    padding: var(--space-sm) var(--space-md);
    border: 1px solid var(--border);
    background: var(--background);
    color: var(--text);
    border-radius: var(--border-radius);
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s ease;
    font-family: var(--font-body);
  }

  .fr-settings-cancel:hover {
    background: color-mix(in srgb, var(--text) 5%, var(--background));
    border-color: var(--text-secondary);
  }

  .fr-settings-confirm {
    padding: var(--space-sm) var(--space-md);
    border: none;
    background: var(--accent);
    color: white;
    border-radius: var(--border-radius);
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s ease;
    font-family: var(--font-body);
  }

  .fr-settings-confirm:hover:not(:disabled) {
    background: color-mix(in srgb, var(--accent) 85%, black);
  }

  .fr-settings-confirm:disabled {
    background: var(--text-secondary);
    cursor: not-allowed;
    opacity: 0.6;
  }

  .fr-close-button {
    width: 58px;
    height: 58px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .fr-close-button:hover {
    color: var(--error);
    transform: scale(1.05);
  }

  .fr-review-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-lg) var(--space-xl);
    background: var(--background);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .fr-deck-info h2 {
    margin: 0;
    font-family: var(--font-heading);
    font-weight: 600;
    font-size: 1.5rem;
    color: var(--text);
  }

  .fr-deck-meta {
    color: var(--text-secondary);
    font-size: 0.9rem;
    display: flex;
    gap: var(--space-lg);
    margin-top: var(--space-xs);
    flex-wrap: wrap;
  }

  .fr-progress-bar-container {
    height: 4px;
    background: var(--background);
    width: 100%;
    overflow: hidden;
    flex-shrink: 0;
  }

  .fr-progress-bar {
    height: 100%;
    background: linear-gradient(90deg, var(--accent), color-mix(in srgb, var(--accent) 80%, var(--success)));
    transition: width 0.4s ease;
    border-radius: 0 3px 3px 0;
  }

  .fr-progress-info {
    padding: var(--space-md) var(--space-xl);
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--background);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .fr-progress-text {
    display: flex;
    gap: var(--space-lg);
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .fr-card-type {
    font-weight: 500;
    padding: 2px 8px;
    border-radius: 12px;
    background: color-mix(in srgb, currentColor 15%, transparent);
    font-size: 0.8rem;
  }

  .fr-session-stats {
    display: flex;
    gap: var(--space-md);
  }

  .fr-stat-item {
    display: flex;
    align-items: center;
    font-weight: 500;
    font-size: 0.9rem;
    padding: 4px 8px;
    border-radius: 8px;
  }

  .fr-stat-item.fr-again {
    color: var(--error);
    background: color-mix(in srgb, var(--error) 10%, transparent);
  }

  .fr-stat-item.fr-hard {
    color: var(--warning);
    background: color-mix(in srgb, var(--warning) 10%, transparent);
  }

  .fr-stat-item.fr-good {
    color: var(--success);
    background: color-mix(in srgb, var(--success) 10%, transparent);
  }

  .fr-stat-item.fr-easy {
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 10%, transparent);
  }

  .fr-review-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
    height: 0;
  }

  .fr-loading-state, .fr-error-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-lg);
    color: var(--text-secondary);
    padding: var(--space-xl);
  }

  .fr-loading-spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--border);
    border-top: 3px solid var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .fr-error-icon {
    font-size: 3rem;
  }

  .fr-study-interface {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: var(--space-lg);
    gap: var(--space-lg);
    min-height: 0;
    overflow-y: auto;
  }

  .fr-card-section {
    flex: 0 0 auto;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .fr-flashcard {
    position: relative;
    transform-style: preserve-3d;
    transition: transform 0.6s ease;
    cursor: pointer;
    border-radius: var(--border-radius);
    width: 100%;
    max-width: 810px;
    height: 290px;
  }

  .fr-flashcard.fr-flipped {
    transform: rotateY(180deg);
  }

  .fr-flashcard.fr-animating {
    pointer-events: none;
  }

  .fr-card-face {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    backface-visibility: hidden;
    -webkit-backface-visibility: hidden;    
    border-radius: var(--border-radius);
    background: var(--background);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
    border: 1px solid var(--border);
  }

  .fr-card-front {
    transform: rotateY(0deg);
  }

  .fr-card-back {
    transform: rotateY(180deg);
  }

  .fr-card-content {
    flex: 1;
    display: flex;
    padding: var(--space-md);
    align-items: center;
    justify-content: center;
    overflow-y: overflow;
  }

  .fr-card-text {
    font-size: 1.4rem;
    line-height: 1.6;
    color: var(--text);
    text-align: center;
    word-wrap: break-word;
    word-break: break-word;
    hyphens: none;
    overflow-wrap: break-word;
    max-width: 100%;
  }

  .fr-card-text.fr-long-text {
    font-size: 1.1rem;
    text-align: left;
    line-height: 1.7;
  }

  .fr-answer-text {
    display: flex;
    align-items: center;
    justify-content: center;
    text-align: center;
  }

  .fr-card-footer {
    padding: var(--space-md) var(--space-xl);
    background: color-mix(in srgb, var(--accent) 5%, var(--background));
    border-top: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
  }

  .fr-card-instructions {
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .fr-keyboard-hint {
    color: var(--text-secondary);
    font-size: 0.8rem;
    background: var(--surface);
    padding: 4px 8px;
    border-radius: 4px;
    border: 1px solid var(--border);
  }

  .fr-explanation-panel {
    flex: 0 0 auto;
    background: color-mix(in srgb, var(--accent) 5%, var(--background));
    border-radius: var(--border-radius);
    border: 1px solid color-mix(in srgb, var(--accent) 20%, var(--border));
    overflow: hidden;
  }

  .fr-explanation-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-md) var(--space-lg);
    background: color-mix(in srgb, var(--accent) 10%, var(--background));
    border-bottom: 1px solid color-mix(in srgb, var(--accent) 20%, var(--border));
    color: var(--accent);
    font-weight: 600;
    font-size: 0.95rem;
  }

  .fr-explanation-content {
    padding: var(--space-lg);
    color: var(--text);
    line-height: 1.6;
    font-size: 0.95rem;
    max-height: 200px;
    overflow-y: auto;
  }

  .fr-control-section {
    flex: 0 0 auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .fr-navigation-controls {
    display: flex;
    gap: var(--space-md);
    align-items: center;
    justify-content: center;
    flex-wrap: wrap;
  }

  .fr-nav-button {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-xs);
    padding: var(--space-md) var(--space-lg);
    border: none;
    border-radius: var(--border-radius);
    background: var(--background);
    color: var(--text);
    cursor: pointer;
    font-family: var(--font-body);
    font-weight: 500;
    transition: all 0.2s ease;
    border: 1px solid var(--border);
    min-width: 120px;
    font-size: 0.9rem;
  }

  .fr-nav-button:hover:not(:disabled) {
    background: color-mix(in srgb, var(--accent) 10%, var(--background));
    transform: translateY(-1px);
  }

  .fr-nav-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  .fr-nav-button.fr-flip {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }

  .fr-nav-button.fr-flip:hover:not(:disabled) {
    background: color-mix(in srgb, var(--accent) 85%, black);
  }

  .fr-nav-button.fr-explain {
    background: color-mix(in srgb, var(--accent) 12%, var(--background));
    border-color: color-mix(in srgb, var(--accent) 30%, var(--border));
    color: var(--accent);
    font-weight: 600;
  }

  .fr-nav-button.fr-explain:hover:not(:disabled) {
    background: color-mix(in srgb, var(--accent) 20%, var(--background));
  }

  .fr-nav-button.fr-explain:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .fr-nav-button.fr-explain:disabled:hover {
    transform: none;
    background: var(--background);
  }

  .fr-mini-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--border);
    border-top: 2px solid var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .fr-rating-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    background: var(--background);
    padding: var(--space-md);
    border-radius: var(--border-radius);
    border: 1px solid var(--border);
  }

  .fr-rating-prompt {
    text-align: center;
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.95rem;
    font-weight: 500;
  }

  .fr-rating-buttons {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-md);
  }

  .fr-rating-button {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-sm);
    border: none;
    border-radius: var(--border-radius);
    cursor: pointer;
    font-family: var(--font-body);
    transition: all 0.2s ease;
    position: relative;
    background: var(--surface);
    border: 2px solid var(--border);
    min-height: 60px;
    justify-content: center;
  }

  .fr-rating-label {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text);
  }

  .fr-rating-key {
    position: absolute;
    top: 6px;
    right: 8px;
    font-size: 0.7rem;
    color: var(--text-secondary);
    background: var(--background);
    padding: 2px 6px;
    border-radius: 3px;
    border: 1px solid var(--border);
  }

  .fr-rating-button.fr-again {
    border-color: color-mix(in srgb, var(--error) 40%, var(--border));
  }

  .fr-rating-button.fr-hard {
    border-color: color-mix(in srgb, var(--warning) 40%, var(--border));
  }

  .fr-rating-button.fr-good {
    border-color: color-mix(in srgb, var(--success) 40%, var(--border));
  }

  .fr-rating-button.fr-easy {
    border-color: color-mix(in srgb, var(--accent) 40%, var(--border));
  }

  .fr-rating-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .fr-rating-button.fr-again:hover {
    background: color-mix(in srgb, var(--error) 10%, var(--surface));
    border-color: var(--error);
  }

  .fr-rating-button.fr-hard:hover {
    background: color-mix(in srgb, var(--warning) 10%, var(--surface));
    border-color: var(--warning);
  }

  .fr-rating-button.fr-good:hover {
    background: color-mix(in srgb, var(--success) 10%, var(--surface));
    border-color: var(--success);
  }

  .fr-rating-button.fr-easy:hover {
    background: color-mix(in srgb, var(--accent) 10%, var(--surface));
    border-color: var(--accent);
  }

  .fr-keyboard-shortcuts {
    text-align: center;
    padding: var(--space-sm) var(--space-md) 0 var(--space-md);
  }

  .fr-shortcut-hint {
    color: var(--text-secondary);
    font-size: 0.8rem;
    font-style: italic;
  }

  .fr-complete-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    gap: var(--space-md);
    padding: var(--space-xl);
    text-align: center;
    overflow-y: auto;
    min-height: 0;
    height: 100%;
    box-sizing: border-box;
  }

  .fr-complete-icon {
    color: var(--success);
    margin-bottom: var(--space-md);
    flex-shrink: 0;
    margin-top: var(--space-lg);
  }

  .fr-complete-state h3 {
    font-family: var(--font-heading);
    font-weight: 600;
    font-size: 2rem;
    margin: 0;
    color: var(--text);
    flex-shrink: 0;
  }

  .fr-completion-stats {
    display: flex;
    flex-direction: column;
    gap: var(--space-md);
    width: 100%;
    max-width: 600px;
    flex-shrink: 0;
  }

  .fr-main-stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-lg);
    flex-shrink: 0;
  }

  .fr-stat-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--space-lg);
    background: var(--background);
    border-radius: var(--border-radius);
    border: 1px solid var(--border);
    min-height: 100px;
  }

  .fr-stat-value {
    font-size: 2.5rem;
    font-weight: 600;
    color: var(--accent);
    margin-bottom: var(--space-xs);
    line-height: 1.2;
  }

  .fr-stat-label {
    color: var(--text-secondary);
    font-size: 0.9rem;
    font-weight: 500;
  }

  .fr-rating-breakdown {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-md);
    flex-shrink: 0;
  }

  .fr-rating-stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--space-md);
    background: var(--background);
    border-radius: var(--border-radius);
    border: 1px solid var(--border);
    min-height: 80px;
  }

  .fr-rating-count {
    font-size: 1.8rem;
    font-weight: 600;
    margin-bottom: var(--space-xs);
    line-height: 1.2;
  }

  .fr-rating-stat.fr-again .fr-rating-count { color: var(--error); }
  .fr-rating-stat.fr-hard .fr-rating-count { color: var(--warning); }
  .fr-rating-stat.fr-good .fr-rating-count { color: var(--success); }
  .fr-rating-stat.fr-easy .fr-rating-count { color: var(--accent); }

  .fr-complete-actions {
    display: flex;
    gap: var(--space-md);
    margin-top: var(--space-lg);
    flex-shrink: 0;
    padding-bottom: var(--space-lg);
  }

  .fr-confetti-container {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    overflow: hidden;
  }

  .fr-confetti {
    position: absolute;
    top: -10px;
    left: var(--left-pos);
    width: var(--size);
    height: var(--size);
    background-color: hsl(var(--hue), 70%, 60%);
    opacity: 0.9;
    border-radius: 2px;
    animation: confetti-fall var(--fall-speed) var(--fall-delay) linear forwards;
    transition: opacity 2s ease-out;
  }

  .fr-close-button:focus,
  .fr-nav-button:focus,
  .fr-rating-button:focus,
  .fr-settings-cancel:focus,
  .fr-settings-confirm:focus {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .fr-flashcard:focus {
    outline: 2px solid var(--accent);
    outline-offset: 4px;
  }

  .fr-setting-item:focus-within {
    background: color-mix(in srgb, var(--accent) 8%, var(--background));
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-radius: var(--border-radius);
  }

  .fr-explain-button-wrapper {
    display: inline-block;
  }

  @media (max-width: 768px) {
    .fr-review-overlay {
      padding: var(--space-sm);
    }

    .fr-review-container {
      max-height: 100vh;
    }

    .fr-settings-modal {
      width: 95%;
      max-width: none;
    }

    .fr-deck-stats {
      grid-template-columns: repeat(2, 1fr);
      gap: var(--space-xs);
    }

    .fr-stat-box {
      padding: var(--space-sm);
    }

    .fr-stat-number {
      font-size: 1.2rem;
    }

    .fr-review-header {
      padding: var(--space-md) var(--space-lg);
    }

    .fr-deck-info h2 {
      font-size: 1.3rem;
    }

    .fr-progress-info {
      padding: var(--space-sm) var(--space-lg);
      flex-direction: column;
      gap: var(--space-sm);
      align-items: flex-start;
    }

    .fr-study-interface {
      padding: var(--space-md);
      gap: var(--space-md);
    }

    .fr-flashcard {
      height: 240px;
    }

    .fr-card-text {
      font-size: 1.2rem;
    }

    .fr-card-text.fr-long-text {
      font-size: 1rem;
    }

    .fr-navigation-controls {
      justify-content: center;
    }

    .fr-nav-button {
      min-width: 100px;
      font-size: 0.85rem;
    }

    .fr-rating-buttons {
      grid-template-columns: repeat(2, 1fr);
      gap: var(--space-sm);
    }

    .fr-rating-button {
      min-height: 70px;
    }

    .fr-explanation-content {
      max-height: 150px;
    }

    .fr-main-stats {
      grid-template-columns: 1fr;
      gap: var(--space-md);
    }

    .fr-rating-breakdown {
      grid-template-columns: repeat(2, 1fr);
    }

    .fr-complete-actions {
      flex-direction: column;
      width: 100%;
    }

    .fr-complete-state {
      padding: var(--space-lg) var(--space-md);
      justify-content: flex-start;
    }

    .fr-complete-icon {
      margin-top: var(--space-md);
    }

    .fr-complete-state h3 {
      font-size: 1.8rem;
    }

    .fr-complete-actions {
      flex-direction: column;
      width: 100%;
      margin-top: var(--space-md);
      padding-bottom: var(--space-xl);
    }

    .fr-stat-card {
      min-height: 80px;
      padding: var(--space-md);
    }

    .fr-stat-value {
      font-size: 2rem;
    }

    .fr-rating-stat {
      min-height: 70px;
    }

    .fr-rating-count {
      font-size: 1.5rem;
    }
  }

  @media (max-width: 480px) {
    .fr-settings-modal {
      width: 98%;
      margin: var(--space-sm);
    }

    .fr-deck-stats {
      grid-template-columns: 1fr;
      gap: var(--space-xs);
    }

    .fr-settings-actions {
      flex-direction: column;
      gap: var(--space-xs);
    }

    .fr-settings-cancel,
    .fr-settings-confirm {
      width: 100%;
    }

    .fr-flashcard {
      height: 200px;
    }

    .fr-rating-buttons {
      grid-template-columns: 1fr;
      gap: var(--space-xs);
    }
    
    .fr-rating-breakdown {
      grid-template-columns: 1fr;
    }
    
    .fr-session-stats {
      flex-wrap: wrap;
      justify-content: center;
      gap: var(--space-sm);
    }

    .fr-navigation-controls {
      flex-direction: column;
      gap: var(--space-sm);
    }

    .fr-nav-button {
      width: 100%;
      min-width: auto;
    }

    .fr-explanation-content {
      max-height: 120px;
    }

    .fr-complete-state {
      padding: var(--space-md) var(--space-sm);
      gap: var(--space-md);
    }

    .fr-complete-icon {
      margin-top: var(--space-sm);
      margin-bottom: var(--space-sm);
    }

    .fr-complete-state h3 {
      font-size: 1.5rem;
    }

    .fr-rating-breakdown {
      grid-template-columns: 1fr;
    }

    .fr-completion-stats {
      gap: var(--space-lg);
    }

    .fr-stat-value {
      font-size: 1.8rem;
    }

    .fr-rating-count {
      font-size: 1.3rem;
    }

    .fr-complete-actions {
      padding-bottom: var(--space-xl);
    }
  }

  @media (max-width: 360px) {
    .fr-settings-header {
      padding: var(--space-md);
      margin-bottom: var(--space-md);
    }

    .fr-settings-content {
      padding: 0 var(--space-md) var(--space-md) var(--space-md);
    }

    .fr-study-interface {
      padding: var(--space-sm);
    }

    .fr-flashcard {
      height: 180px;
    }

    .fr-card-text {
      font-size: 1.1rem;
    }

    .fr-nav-button {
      padding: var(--space-sm) var(--space-md);
      font-size: 0.8rem;
    }

    .fr-rating-button {
      padding: var(--space-sm);
      min-height: 60px;
    }

    .fr-rating-label {
      font-size: 0.8rem;
    }

    .fr-complete-state {
      padding: var(--space-sm);
    }

    .fr-complete-state h3 {
      font-size: 1.3rem;
    }

    .fr-stat-card {
      min-height: 70px;
      padding: var(--space-sm);
    }

    .fr-stat-value {
      font-size: 1.6rem;
    }

    .fr-rating-stat {
      min-height: 60px;
      padding: var(--space-sm);
    }

    .fr-rating-count {
      font-size: 1.2rem;
    }
  }

  @media (prefers-contrast: high) {
    .fr-card-face {
      border-width: 2px;
    }

    .fr-rating-button {
      border-width: 2px;
    }

    .fr-nav-button {
      border-width: 2px;
    }

    .fr-settings-modal {
      border-width: 2px;
    }

    .fr-stat-box {
      border-width: 3px;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .fr-flashcard {
      transition: none;
    }

    .fr-confetti {
      animation: none;
    }

    .fr-loading-spinner {
      animation: none;
      border-top-color: var(--accent);
    }

    .fr-mini-spinner {
      animation: none;
      border-top-color: var(--accent);
    }

    .fr-nav-button:hover:not(:disabled),
    .fr-rating-button:hover,
    .fr-settings-cancel:hover,
    .fr-settings-confirm:hover:not(:disabled) {
      transform: none;
    }
  }

  @media print {
    .fr-review-overlay {
      background: white;
      position: static;
    }

    .fr-review-container {
      box-shadow: none;
      border: 1px solid black;
      max-height: none;
      height: auto;
    }

    .fr-settings-overlay,
    .fr-control-section,
    .fr-close-button {
      display: none;
    }

    .fr-flashcard {
      transform: none !important;
      height: auto;
    }

    .fr-card-back {
      transform: none;
      position: static;
    }
  }
</style>
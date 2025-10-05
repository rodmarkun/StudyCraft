export type CardDifficulty = 'again' | 'hard' | 'good' | 'easy';
export type CardStatus = 'new' | 'learning' | 'review' | 'suspended';
export type ReviewResult = 'again' | 'hard' | 'good' | 'easy';

export interface Card {
  id: number;
  deck_id: string;
  front: string;
  back: string;
  position: number;
  due_date: string;
  interval_days: number;
  ease_factor: number;
  repetitions: number;
  status: CardStatus;
  last_reviewed?: string;
  learning_step: number;
  created_at: string;
  updated_at: string;
}

export interface ReviewSession {
  sessionStart: Date;
  cardsStudied: number;
  timeSpent: number;
  againCount: number;
  hardCount: number;
  goodCount: number;
  easyCount: number;
  newCards: number;
  reviewCards: number;
  learningCards: number;
}

export interface ReviewSettings {
  shuffle_deck: boolean;
  use_only_pending: boolean;
}

export interface FlashcardCounts {
  new_count: number;
  learning_count: number;
  review_count: number;
  pending_count: number;
  total_count: number;
}

export class FlashcardReviewLogic {
  private cards: Card[] = [];
  private originalCards: Card[] = [];
  public currentIndex: number = 0; 
  private session: ReviewSession;
  private settings: ReviewSettings;
  private reviewedCards: Set<number> = new Set(); 
  private againCards: Set<number> = new Set(); // Track cards marked as "again"

  constructor(cards: Card[], settings?: Partial<ReviewSettings>) {
    console.log('Initializing FlashcardReviewLogic with cards:', cards);
    console.log('Settings:', settings);
    
    const validatedCards = cards.map((card, index) => {
      console.log(`Validating card ${index}:`, {
        id: card.id,
        idType: typeof card.id,
        front: card.front?.substring(0, 30) + '...'
      });
      
      if (card.id == null || card.id === undefined) {
        console.error(`ERROR: Card ${index} has null/undefined ID:`, card);
        throw new Error(`Card ${index} has invalid ID: ${card.id}`);
      }
      
      if (typeof card.id !== 'number') {
        console.warn(`WARNING: Card ${index} ID is not a number:`, card.id, typeof card.id);
        // Try to convert string ID to number
        if (typeof card.id === 'string' && !isNaN(Number(card.id))) {
          console.log(`Converting string ID to number for card ${index}`);
          return { ...card, id: Number(card.id) };
        } else {
          throw new Error(`Card ${index} has invalid ID type: ${typeof card.id}`);
        }
      }
      
      return card;
    });
    
    this.originalCards = [...validatedCards];
    this.cards = [...validatedCards];
    this.session = this.initializeSession();
    this.settings = {
      shuffle_deck: true,
      use_only_pending: true,
      ...settings
    };
    
    console.log('FlashcardReviewLogic initialized successfully with', this.cards.length, 'cards');
  }

  private initializeSession(): ReviewSession {
    return {
      sessionStart: new Date(),
      cardsStudied: 0,
      timeSpent: 0,
      againCount: 0,
      hardCount: 0,
      goodCount: 0,
      easyCount: 0,
      newCards: 0,
      reviewCards: 0,
      learningCards: 0
    };
  }

  getCurrentCard(): Card | null {
    const card = this.cards[this.currentIndex] || null;
    
    if (card) {
      console.log('Getting current card:', {
        index: this.currentIndex,
        id: card.id,
        idType: typeof card.id,
        front: card.front?.substring(0, 30) + '...',
        status: card.status
      });
      
      // Validate the card ID
      if (card.id == null || card.id === undefined) {
        console.error('ERROR: Current card has null/undefined ID:', card);
        throw new Error('Current card has invalid ID');
      }
      
      if (typeof card.id !== 'number') {
        console.error('ERROR: Current card ID is not a number:', card.id, typeof card.id);
        throw new Error(`Current card has invalid ID type: ${typeof card.id}`);
      }
    } else {
      console.log('No current card at index:', this.currentIndex);
    }
    
    return card;
  }

  getProgress(): { current: number; total: number; percentage: number } {
    const progress = {
      current: this.currentIndex + 1,
      total: this.cards.length,
      percentage: this.cards.length > 0 ? ((this.currentIndex + 1) / this.cards.length) * 100 : 0
    };
    
    console.log('Current progress:', progress);
    return progress;
  }

  getSession(): ReviewSession {
    return { ...this.session };
  }

  canMoveNext(): boolean {
    const canMove = this.currentIndex < this.cards.length - 1;
    console.log(`Can move next: ${canMove} (current: ${this.currentIndex}, total: ${this.cards.length})`);
    return canMove;
  }

  canMovePrevious(): boolean {
    const canMove = this.currentIndex > 0;
    console.log(`Can move previous: ${canMove} (current: ${this.currentIndex})`);
    return canMove;
  }

  moveNext(): boolean {
    if (this.canMoveNext()) {
      this.currentIndex++;
      console.log(`Moved to next card. New index: ${this.currentIndex}`);
      return true;
    }
    console.log('Cannot move to next card');
    return false;
  }

  movePrevious(): boolean {
    if (this.canMovePrevious()) {
      this.currentIndex--;
      console.log(`Moved to previous card. New index: ${this.currentIndex}`);
      return true;
    }
    console.log('Cannot move to previous card');
    return false;
  }

  reviewCard(difficulty: CardDifficulty): void {
    const card = this.getCurrentCard();
    if (!card) throw new Error('No current card to review');
    
    console.log(`Reviewing card ${card.id} with difficulty: ${difficulty}`);
    
    const wasAlreadyReviewed = this.reviewedCards.has(card.id);
    if (!wasAlreadyReviewed) {
      this.session.cardsStudied++;
      this.updateCardTypeCounts(card);
      this.reviewedCards.add(card.id);
      console.log(`New card reviewed. Total cards studied: ${this.session.cardsStudied}`);
    } else {
      console.log(`Card ${card.id} was already reviewed, not incrementing cardsStudied`);
    }
    
    const countKey = `${difficulty}Count` as keyof ReviewSession;
    (this.session[countKey] as number)++;
    console.log(`Updated ${difficulty} count:`, this.session[countKey]);
    
    // Handle "again" cards - move them to the end of the session
    if (difficulty === 'again') {
      this.handleAgainCard(card);
    } else {
      // Remove from again cards if it was there (in case of re-review)
      this.againCards.delete(card.id);
    }
    
    console.log('Session after review:', {
      cardsStudied: this.session.cardsStudied,
      againCount: this.session.againCount,
      hardCount: this.session.hardCount,
      goodCount: this.session.goodCount,
      easyCount: this.session.easyCount,
      reviewedCardIds: Array.from(this.reviewedCards),
      againCardIds: Array.from(this.againCards)
    });
  }

  private handleAgainCard(card: Card): void {
    console.log(`Handling "again" card ${card.id}`);
    
    // Don't add the same card multiple times to avoid infinite loops
    if (!this.againCards.has(card.id)) {
      // Create a copy of the card to add to the end
      const cardCopy = { ...card };
      
      // Add to the end of the cards array
      this.cards.push(cardCopy);
      this.againCards.add(card.id);
      
      console.log(`Added card ${card.id} to end of session. New total cards: ${this.cards.length}`);
    } else {
      console.log(`Card ${card.id} already scheduled for repetition`);
    }
  }

  private updateCardTypeCounts(card: Card): void {
    console.log(`Updating card type count for status: ${card.status}`);
    switch (card.status) {
      case 'new':
        this.session.newCards++;
        break;
      case 'learning':
        this.session.learningCards++;
        break;
      case 'review':
        this.session.reviewCards++;
        break;
    }
  }

  updateCard(cardId: number, updatedCard: Card): void {
    console.log('Updating card:', cardId, 'with:', updatedCard);
    
    // Update all instances of this card (original and any copies)
    let updateCount = 0;
    for (let i = 0; i < this.cards.length; i++) {
      if (this.cards[i].id === cardId) {
        this.cards[i] = { ...updatedCard };
        updateCount++;
        console.log(`Updated card at index ${i}`);
      }
    }
    
    if (updateCount > 0) {
      console.log(`Updated ${updateCount} instance(s) of card ${cardId}`);
    } else {
      console.error('ERROR: Could not find card with ID:', cardId);
      console.log('Available card IDs:', this.cards.map(c => c.id));
    }
  }

  getNextDueTime(card: Card): string {
    if (!card.due_date) return 'Now';
    
    const now = new Date();
    const due = new Date(card.due_date);
    const diffMs = due.getTime() - now.getTime();
    
    if (diffMs <= 0) return 'Now';
    
    const diffMinutes = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    
    if (diffMinutes < 60) return `${diffMinutes}m`;
    if (diffHours < 24) return `${diffHours}h`;
    if (diffDays < 30) return `${diffDays}d`;
    
    const diffMonths = Math.floor(diffDays / 30);
    if (diffMonths < 12) return `${diffMonths}mo`;
    
    const diffYears = Math.floor(diffDays / 365);
    return `${diffYears}y`;
  }

  getCardTypeLabel(card: Card): string {
    switch (card.status || 'new') {
      case 'new': return 'New';
      case 'learning': return 'Learning';
      case 'review': return 'Review';
      case 'suspended': return 'Suspended';
      default: return 'Unknown';
    }
  }

  getCardTypeColor(card: Card): string {
    switch (card.status) {
      case 'new': return 'var(--accent, #3b82f6)';
      case 'learning': return 'var(--warning, #f59e0b)';
      case 'review': return 'var(--success, #10b981)';
      default: return 'var(--text-secondary, #6b7280)';
    }
  }

  updateSessionTime(seconds: number): void {
    this.session.timeSpent = seconds;
  }

  reset(): void {
    console.log('Resetting FlashcardReviewLogic');
    this.currentIndex = 0;
    this.session = this.initializeSession();
    this.cards = [...this.originalCards];
    this.reviewedCards.clear();
    this.againCards.clear(); // Clear again cards tracking
    console.log('Reset complete. Cards:', this.cards.length);
  }

  static formatStudyTime(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    
    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${minutes}:${secs.toString().padStart(2, '0')}`;
  }

  getRetentionRate(): number {
    const total = this.session.againCount + this.session.hardCount + this.session.goodCount + this.session.easyCount;
    if (total === 0) return 0;
    const successful = this.session.goodCount + this.session.easyCount;
    return Math.round((successful / total) * 100);
  }

  getEstimatedReviewTime(card: Card): string {
    if (card.status === 'new') {
      return '1m';
    }
    
    if (card.status === 'learning') {
      const learningSteps = [1, 10]; 
      const currentStep = card.learning_step;
      
      if (currentStep < learningSteps.length - 1) {
        return `${learningSteps[currentStep + 1]}m`;
      } else {
        return '1 day';
      }
    }
    
    if (card.status === 'review') {
      let nextInterval;
      
      if (card.repetitions === 0) {
        nextInterval = 1;
      } else if (card.repetitions === 1) {
        nextInterval = 6;
      } else {
        nextInterval = Math.round(card.interval_days * card.ease_factor);
      }
      
      if (nextInterval === 1) return '1 day';
      if (nextInterval < 30) return `${nextInterval} days`;
      if (nextInterval < 365) {
        const months = Math.round(nextInterval / 30);
        return `${months} month${months > 1 ? 's' : ''}`;
      }
      const years = Math.round(nextInterval / 365);
      return `${years} year${years > 1 ? 's' : ''}`;
    }
    
    return 'Soon';
  }

  getEstimatedTimeForDifficulty(card: Card, difficulty: CardDifficulty): string {
    const learningSteps = [1, 10];
    
    switch (difficulty) {
      case 'again':
        return '<1m';
      
      case 'hard':
        if (card.status === 'new') {
          return '1m';
        } else if (card.status === 'learning') {
          const currentStep = card.learning_step;
          if (currentStep < learningSteps.length - 1) {
            return `${learningSteps[currentStep + 1]}m`;
          } else {
            return '1 day';
          }
        } else if (card.status === 'review') {
          const hardInterval = Math.max(1, Math.round(card.interval_days * 1.2));
          if (hardInterval === 1) return '1 day';
          if (hardInterval < 30) return `${hardInterval} days`;
          return `${Math.round(hardInterval / 30)} months`;
        }
        break;
      
      case 'good':
        return '10m';
        return this.getEstimatedReviewTime(card);
      
      case 'easy':
        if (card.status === 'new' || card.status === 'learning') {
          return '4 days';
        } else if (card.status === 'review') {
          let easyInterval;
          if (card.repetitions <= 1) {
            easyInterval = 4;
          } else {
            easyInterval = Math.round(card.interval_days * card.ease_factor * 1.3);
          }
          if (easyInterval < 30) return `${easyInterval} days`;
          if (easyInterval < 365) return `${Math.round(easyInterval / 30)} months`;
          return `${Math.round(easyInterval / 365)} years`;
        }
        break;
    }
    
    return 'Soon';
  }

  // Helper method to check if a card is scheduled for repetition
  isCardScheduledForRepetition(cardId: number): boolean {
    return this.againCards.has(cardId);
  }

  // Get the number of cards remaining in the session
  getRemainingCardsCount(): number {
    return this.cards.length - this.currentIndex - 1;
  }
}
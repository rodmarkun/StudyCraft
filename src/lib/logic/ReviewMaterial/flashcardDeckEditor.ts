import { invoke } from '@tauri-apps/api/core';
import type { StudyMaterial, GeneratedCard } from './flashcardDeckCreator';

export interface ReviewMaterial {
  id: string;
  name: string;
  type: 'flashcard_deck' | 'test';
  cards_count?: number;
  questions_count?: number;
  last_review?: string;
  tags: string[];
}

export interface EditableCard {
  id?: number | null;
  front: string;
  back: string;
  position?: number;
  isNew?: boolean; 
  isModified?: boolean; 
  isDeleted?: boolean;
  tempId?: string;
}

export interface DeckEditFormData {
  deckId: string;
  name: string;
  language: string;
  selectedMaterials: StudyMaterial[];
  concept: string;
}

export interface ExistingFlashcard {
  id: number;
  deck_id: string;
  front: string;
  back: string;
  position: number;
}

export interface FlashcardUpdateRequest {
  deck_id: string;
  name?: string;
  cards_to_add?: Array<{
    front: string;
    back: string;
    position: number;
  }>;
  cards_to_update?: Array<{
    id: number;
    front: string;
    back: string;
    position: number;
  }>;
  cards_to_delete?: number[];
}

export class FlashcardDeckEditorLogic {
  private cards: EditableCard[] = [];
  private originalCards: EditableCard[] = [];
  private isLoading: boolean = false;
  private isSaving: boolean = false;
  private isGenerating: boolean = false;
  private error: string = '';
  private generationStatus: string = '';
  private hasUnsavedChanges: boolean = false;
  private originalDeckName: string = '';
  private nextTempId: number = 1;
  public currentStep: number = 2;

  get getCards(): EditableCard[] {
    return [...this.cards];
  }

  get getIsLoading(): boolean {
    return this.isLoading;
  }

  get getIsSaving(): boolean {
    return this.isSaving;
  }

  get getIsGenerating(): boolean {
    return this.isGenerating;
  }

  get getError(): string {
    return this.error;
  }

  get getGenerationStatus(): string {
    return this.generationStatus;
  }

  get getHasUnsavedChanges(): boolean {
    return this.hasUnsavedChanges;
  }

  moveToNextStep(): void {
    this.currentStep = Math.min(this.currentStep + 1, 3);
  }

  moveToPreviousStep(): void {
    this.currentStep = Math.max(this.currentStep - 1, 2); 
  }

  getCurrentStep(): number {
    return this.currentStep;
  }

  async loadExistingCards(deckId: string): Promise<void> {
    this.isLoading = true;
    this.error = '';

    try {
      console.log('Loading existing cards for deck:', deckId);
      
      const existingCards = await invoke('get_flashcards', {
        deckId: deckId
      }) as ExistingFlashcard[];

      console.log('Loaded existing cards:', existingCards);

      this.cards = existingCards
        .sort((a, b) => a.position - b.position) 
        .map((card, index) => ({
          id: card.id,
          front: card.front,
          back: card.back,
          position: card.position,
          isNew: false,
          isModified: false,
          isDeleted: false,
          tempId: `existing_${card.id}_${index}`
        }));

      this.originalCards = JSON.parse(JSON.stringify(this.cards));
      this.hasUnsavedChanges = false;
      this.nextTempId = 1000; // Start temp IDs high to avoid conflicts

      console.log('Processed cards:', this.cards);
    } catch (err) {
      console.error('Failed to load existing cards:', err);
      this.error = `Failed to load existing cards: ${err}`;
    } finally {
      this.isLoading = false;
    }
  }

  isFormValid(formData: DeckEditFormData): boolean {
    return formData.name.trim() !== '' && this.cards.some(card => !card.isDeleted) && 
           this.cards.filter(card => !card.isDeleted).every(card => 
             card.front.trim() !== '' && card.back.trim() !== ''
           );
  }

  addCard(): void {
    const newCard: EditableCard = {
      id: null,
      front: '',
      back: '',
      position: this.cards.filter(card => !card.isDeleted).length,
      isNew: true,
      isModified: false,
      isDeleted: false,
      tempId: `temp_${this.nextTempId++}`
    };
    
    this.cards = [...this.cards, newCard];
    this.updateCardPositions();
    this.checkForChanges();
  }

  addCardWithContent(front: string, back: string): void {
    const newCard: EditableCard = {
      id: null,
      front,
      back,
      position: this.cards.filter(card => !card.isDeleted).length,
      isNew: true,
      isModified: false,
      isDeleted: false,
      tempId: `temp_${this.nextTempId++}`
    };
    
    this.cards = [...this.cards, newCard];
    this.updateCardPositions();
    this.checkForChanges();
  }

  removeCard(index: number): void {
    const visibleCards = this.cards.filter(card => !card.isDeleted);
    const cardToRemove = visibleCards[index];
    
    if (!cardToRemove) return;
    
    const actualIndex = this.cards.findIndex(card => card === cardToRemove);
    
    if (cardToRemove.isNew) {
      this.cards = this.cards.filter((_, i) => i !== actualIndex);
    } else {
      this.cards = this.cards.map((card, i) => 
        i === actualIndex ? { ...cardToRemove, isDeleted: true } : card
      );
    }
    
    this.updateCardPositions();
    this.checkForChanges();
  }

  updateCard(index: number, side: 'front' | 'back', value: string): void {
    const visibleCards = this.cards.filter(card => !card.isDeleted);
    const cardToUpdate = visibleCards[index];
    
    if (!cardToUpdate) return;
    
    const actualIndex = this.cards.findIndex(card => card === cardToUpdate);
    const originalCard = this.originalCards.find(c => c.id === cardToUpdate.id);
    
    this.cards = this.cards.map((card, i) => 
      i === actualIndex ? {
        ...cardToUpdate,
        [side]: value,
        isModified: !cardToUpdate.isNew && originalCard && (
          originalCard.front !== (side === 'front' ? value : cardToUpdate.front) ||
          originalCard.back !== (side === 'back' ? value : cardToUpdate.back)
        )
      } : card
    );
    
    this.checkForChanges();
  }

  updateDeckName(name: string): void {
    if (name !== this.originalDeckName) {
      this.hasUnsavedChanges = true;
    } else {
      this.checkForChanges();
    }
  }

  private updateCardPositions(): void {
    let position = 0;
    this.cards = this.cards.map(card => {
      if (!card.isDeleted) {
        return { ...card, position: position++ };
      }
      return card;
    });
  }

  private checkForChanges(): void {
    const hasNewCards = this.cards.some(card => card.isNew);
    const hasModifiedCards = this.cards.some(card => card.isModified);
    const hasDeletedCards = this.cards.some(card => card.isDeleted);
    
    this.hasUnsavedChanges = hasNewCards || hasModifiedCards || hasDeletedCards;
  }

  removeMaterial(materialToRemove: StudyMaterial, currentMaterials: StudyMaterial[]): StudyMaterial[] {
    return currentMaterials.filter(material => material.id !== materialToRemove.id);
  }

  updateMaterialCardCount(materialId: string, count: number, materials: StudyMaterial[]): StudyMaterial[] {
    return materials.map(material => 
      material.id === materialId ? { ...material, cardsToGenerate: count } : material
    );
  }

  getTotalCardsToGenerate(materials: StudyMaterial[]): number {
    return materials.reduce((sum, material) => sum + (material.cardsToGenerate || 5), 0);
  }

  async generateAdditionalCards(formData: DeckEditFormData): Promise<void> {
    if (formData.selectedMaterials.length === 0) {
      throw new Error('No materials selected for generation');
    }

    if (!formData.concept.trim()) {
      throw new Error('No concept specified for generation');
    }

    this.isGenerating = true;
    this.error = '';
    this.generationStatus = 'Preparing study materials...';

    try {
      console.log('Generating additional cards for:', formData.selectedMaterials);
      
      const materialsRequest = formData.selectedMaterials.map(material => {
        if (!material.id) {
          console.warn('Missing material ID for material:', material);
          return null;
        }
        
        return {
          material_id: material.id,
          cards_to_generate: material.cardsToGenerate || 5
        };
      }).filter(req => req !== null);

      if (materialsRequest.length === 0) {
        throw new Error('No valid materials with IDs found');
      }

      this.generationStatus = `Generating flashcards focused on "${formData.concept}"...`;
      
      const generatedCards = await invoke('generate_flashcards', {
        request: {
          materials: materialsRequest,
          language: formData.language || 'English',
          concept: formData.concept
        }
      }) as GeneratedCard[];

      console.log('Generated additional cards:', generatedCards);
      
      if (generatedCards && Array.isArray(generatedCards) && generatedCards.length > 0) {
        this.generationStatus = `Successfully generated ${generatedCards.length} additional flashcards`;
        
        const currentVisibleCards = this.cards.filter(card => !card.isDeleted);
        const newCards: EditableCard[] = generatedCards.map((card: GeneratedCard, index: number) => ({
          id: null,
          front: card.question,
          back: card.answer,
          position: currentVisibleCards.length + index,
          isNew: true,
          isModified: false,
          isDeleted: false
        }));
        
        this.cards = [...this.cards, ...newCards];
        this.checkForChanges();
        
        setTimeout(() => {
          this.generationStatus = '';
          this.isGenerating = false;
        }, 2000);
      } else {
        throw new Error('No additional flashcards were generated. Please try again or add cards manually.');
      }
    } catch (err) {
      console.error('Failed to generate additional flashcards:', err);
      this.error = `Failed to generate additional flashcards: ${err}`;
      this.isGenerating = false;
      throw err;
    }
  }

  async saveChanges(formData: DeckEditFormData): Promise<void> {
    if (!this.isFormValid(formData)) {
      throw new Error('Form is not valid - check that all cards have content and deck has a name');
    }

    this.isSaving = true;
    this.error = '';

    try {
      const updateRequest: FlashcardUpdateRequest = {
        deck_id: formData.deckId
      };

      if (formData.name !== this.originalDeckName) {
        updateRequest.name = formData.name;
      }

      const cardsToAdd = this.cards
        .filter(card => card.isNew && !card.isDeleted)
        .map((card) => ({
          front: card.front,
          back: card.back,
          position: card.position || 0
        }));

      if (cardsToAdd.length > 0) {
        updateRequest.cards_to_add = cardsToAdd;
      }

      const cardsToUpdate = this.cards
        .filter(card => !card.isNew && card.isModified && !card.isDeleted && card.id !== null)
        .map(card => ({
          id: card.id as number,
          front: card.front,
          back: card.back,
          position: card.position || 0
        }));

      if (cardsToUpdate.length > 0) {
        updateRequest.cards_to_update = cardsToUpdate;
      }

      const cardsToDelete = this.cards
        .filter(card => card.isDeleted && card.id !== null)
        .map(card => card.id as number);

      if (cardsToDelete.length > 0) {
        updateRequest.cards_to_delete = cardsToDelete;
      }

      console.log('Sending update request:', updateRequest);
      await invoke('update_flashcard_deck', {
        request: updateRequest
      });
      console.log('Successfully updated flashcard deck');
      this.hasUnsavedChanges = false;
      
      this.originalCards = this.cards
        .filter(card => !card.isDeleted)
        .map(card => ({
          ...card,
          isNew: false,
          isModified: false,
          id: card.id || Date.now() + Math.random()
        }));
        
      this.cards = [...this.originalCards];
      this.originalDeckName = formData.name;

    } catch (err) {
      console.error('Failed to save changes:', err);
      this.error = `Failed to save changes: ${err}`;
      throw err;
    } finally {
      this.isSaving = false;
    }
  }

  setOriginalDeckName(name: string): void {
    this.originalDeckName = name;
  }

  reset(): void {
    this.cards = [];
    this.originalCards = [];
    this.isLoading = false;
    this.isSaving = false;
    this.error = '';
    this.hasUnsavedChanges = false;
    this.originalDeckName = '';
    this.currentStep = 2;
  }
}
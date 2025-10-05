import { invoke } from '@tauri-apps/api/core';

export interface StudyMaterial {
  id: string;
  name?: string;
  display_name?: string;
  file_name?: string;
  cardsToGenerate?: number;
  study_material_original?: string;
  markdown_path?: string;
}

export interface FlashcardDeck {
  id: string;
  name: string;
  type: 'flashcard_deck';
  cards_count: number;
  last_review: string;
  tags: string[];
  progress: number;
}

export interface Card {
  front: string;
  back: string;
}

export interface DeckFormData {
  name: string;
  language: string;
  tags: string[];
  selectedMaterials: StudyMaterial[];
  concept: string;
}

export interface GeneratedCard {
  question: string;
  answer: string;
}

export interface FlashcardDeckCreationResponse {
  review_material_id: string;
  review_material_name: string;
  cards_count: number;
}

export class FlashcardDeckCreatorLogic {
  private cards: Card[] = [];
  private currentStep: number = 1;
  private isCreating: boolean = false;
  private isGenerating: boolean = false;
  private error: string = '';
  private generationStatus: string = '';

  get getCurrentStep(): number {
    return this.currentStep;
  }

  get getCards(): Card[] {
    return this.cards;
  }

  get getIsCreating(): boolean {
    return this.isCreating;
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

  isFormValid(formData: DeckFormData): boolean {
    return formData.name.trim() !== '' && this.cards.every(card => 
      card.front.trim() !== '' && card.back.trim() !== ''
    );
  }

  isStepComplete(step: number, formData: DeckFormData): boolean {
    switch (step) {
      case 1:
        return formData.name.trim() !== '';
      case 2:
        return this.cards.length > 0 && this.cards.every(card => 
          card.front.trim() !== '' && card.back.trim() !== ''
        );
      default:
        return false;
    }
  }

  canEnableProvider(provider: string): boolean {
    return true;
  }

  addCard(): void {
    this.cards = [...this.cards, { front: '', back: '' }];
  }

  addCardWithContent(front: string, back: string): void {
    this.cards = [...this.cards, { front, back }];
  }

  removeCard(index: number): void {
    this.cards = this.cards.filter((_, i) => i !== index);
  }

  updateCard(index: number, side: 'front' | 'back', value: string): void {
    this.cards[index][side] = value;
  }

  async moveToNextStep(formData: DeckFormData): Promise<void> {
    if (this.currentStep < 3) {
      if (this.currentStep === 1 && formData.selectedMaterials.length > 0) {
        await this.generateFlashcards(formData);
      } else {
        this.currentStep++;
      }
    }
  }

  moveToPreviousStep(): void {
    if (this.currentStep > 1) {
      this.currentStep--;
    }
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

  async generateFlashcards(formData: DeckFormData): Promise<void> {
    if (formData.selectedMaterials.length === 0) {
      this.currentStep++;
      return;
    }

    this.isGenerating = true;
    this.error = '';
    this.generationStatus = 'Preparing study materials...';

    try {
      console.log('Preparing to generate flashcards for:', formData.selectedMaterials);
      
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

      console.log('Sending generation request:', materialsRequest);
      this.generationStatus = formData.concept 
        ? `Generating flashcards focused on "${formData.concept}"...`
        : 'Sending request to AI model...';
      
      const generatedCards = await invoke('generate_flashcards', {
        request: {
          materials: materialsRequest,
          language: formData.language || 'English',
          concept: formData.concept || null 
        }
      }) as GeneratedCard[];

      console.log('Generated cards:', generatedCards);
      this.generationStatus = `Successfully generated ${generatedCards.length} flashcards`;
      
      if (generatedCards && Array.isArray(generatedCards) && generatedCards.length > 0) {
        this.cards = generatedCards.map((card: GeneratedCard) => ({
          front: card.question,
          back: card.answer
        }));
        
        setTimeout(() => {
          this.currentStep = 2;
          this.isGenerating = false;
          console.log('Advanced to step 2, cards:', this.cards);
        }, 1000);
      } else {
        this.error = 'No flashcards were generated. Please try again or create cards manually.';
        this.isGenerating = false;
        this.currentStep = 2;
      }
    } catch (err) {
      console.error('Failed to generate flashcards:', err);
      this.error = `Failed to generate flashcards: ${err}. Please check the material format and try again.`;
      this.isGenerating = false;
      this.currentStep = 2;
    }
  }

  async createFlashcardDeck(formData: DeckFormData, deckId: String): Promise<FlashcardDeck> {
    if (!this.isFormValid(formData)) {
      throw new Error('Form is not valid');
    }

    this.isCreating = true;
    this.error = '';

    try {
      const cardData = this.cards.map((card, index) => ({
        front: card.front,
        back: card.back,
        deck_id: "",
        position: index,
        id: null
      }));
      
      const result = await invoke('create_flashcard_deck', {
        id: deckId,
        name: formData.name,
        tags: formData.tags,
        cards: cardData,
        language: formData.language
      }) as FlashcardDeckCreationResponse;

      console.log('Created flashcard deck:', result);
      
      const newDeck: FlashcardDeck = {
        id: result.review_material_id,
        name: result.review_material_name,
        type: 'flashcard_deck',
        cards_count: this.cards.length,
        last_review: new Date().toISOString().split('T')[0],
        tags: formData.tags,
        progress: 0
      };

      return newDeck;
    } catch (err) {
      console.error('Failed to create flashcard deck:', err);
      this.error = `Failed to create flashcard deck: ${err}`;
      throw err;
    } finally {
      this.isCreating = false;
    }
  }

  reset(): void {
    this.cards = [];
    this.currentStep = 1;
    this.isCreating = false;
    this.isGenerating = false;
    this.error = '';
    this.generationStatus = '';
  }
}
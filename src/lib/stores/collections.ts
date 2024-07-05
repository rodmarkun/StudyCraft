import { writable } from 'svelte/store';
import type { Writable } from 'svelte/store';

export interface StudyMaterial {
  type: 'pdf' | 'markdown' | 'webpage';
  filePath?: string;
  fileName?: string;
  url?: string;
  name: string;
  content?: string;
}

export interface Flashcard {
  id: string;
  question: string;
  answer: string;
}

export interface FlashcardDeck {
  id: string;
  name: string; 
  flashcards: Flashcard[];
}

export interface TestData {
  id: string;
  question: string;
  options: string[];
  correctOptionIndex: number;
}

export interface Test {
  id: string;
  name: string;
  questions: TestData[];
}

export type ReviewMaterial = FlashcardDeck | Test;

export interface Collection {
  id: string;
  name: string;
  studyMaterials: StudyMaterial[];
  reviewMaterials: ReviewMaterial[];
}

const initialCollections: Collection[] = [];

export const collections: Writable<Collection[]> = writable(initialCollections);

export function addCollection(name: string) {
  collections.update(cols => [...cols, { id: Date.now().toString(), name, studyMaterials: [], reviewMaterials: [] }]);
}

export function deleteCollection(id: string) {
  collections.update(cols => cols.filter(col => col.id !== id));
}

export function addStudyMaterials(collectionId: string, newMaterials: StudyMaterial[]) {
  collections.update(cols => {
    const collectionIndex = cols.findIndex(col => col.id === collectionId);
    if (collectionIndex !== -1) {
      cols[collectionIndex].studyMaterials = [...cols[collectionIndex].studyMaterials, ...newMaterials];
    }
    return cols;
  });
}

export function removeStudyMaterial(collectionId: string, materialName: string) {
  collections.update(cols => {
    const collectionIndex = cols.findIndex(col => col.id === collectionId);
    if (collectionIndex !== -1) {
      cols[collectionIndex].studyMaterials = cols[collectionIndex].studyMaterials.filter(
        material => material.name !== materialName
      );
    }
    return cols;
  });
}

export function addFlashcardDeck(collectionId: string, deck: FlashcardDeck) {
  collections.update(cols => {
    const collection = cols.find(c => c.id === collectionId);
    if (collection) {
      collection.reviewMaterials = [...collection.reviewMaterials, deck];
    }
    return cols;
  });
}

export function removeFlashcardDeck(collectionId: string, deckId: string) {
  collections.update(cols => {
    const collectionIndex = cols.findIndex(col => col.id === collectionId);
    if (collectionIndex !== -1) {
      cols[collectionIndex].reviewMaterials = cols[collectionIndex].reviewMaterials.filter(
        material => !('flashcards' in material) || material.id !== deckId
      );
    }
    return cols;
  });
}

export function addFlashcardToDeck(collectionId: string, deckId: string, flashcard: Flashcard) {
  collections.update(cols => {
    const collectionIndex = cols.findIndex(col => col.id === collectionId);
    if (collectionIndex !== -1) {
      const deckIndex = cols[collectionIndex].reviewMaterials.findIndex(
        material => 'flashcards' in material && material.id === deckId
      );
      if (deckIndex !== -1) {
        const deck = cols[collectionIndex].reviewMaterials[deckIndex] as FlashcardDeck;
        deck.flashcards = [...deck.flashcards, flashcard];
      }
    }
    return cols;
  });
}

export function removeFlashcardFromDeck(collectionId: string, deckId: string, flashcardId: string) {
  collections.update(cols => {
    const collectionIndex = cols.findIndex(col => col.id === collectionId);
    if (collectionIndex !== -1) {
      const deckIndex = cols[collectionIndex].reviewMaterials.findIndex(
        material => 'flashcards' in material && material.id === deckId
      );
      if (deckIndex !== -1) {
        const deck = cols[collectionIndex].reviewMaterials[deckIndex] as FlashcardDeck;
        deck.flashcards = deck.flashcards.filter(card => card.id !== flashcardId);
      }
    }
    return cols;
  });
}

export function addTest(collectionId: string, test: Test) {
  collections.update(cols => {
    const collectionIndex = cols.findIndex(col => col.id === collectionId);
    if (collectionIndex !== -1) {
      cols[collectionIndex].reviewMaterials = [...cols[collectionIndex].reviewMaterials, test];
    }
    return cols;
  });
}

export function removeTest(collectionId: string, testId: string) {
  collections.update(cols => {
    const collectionIndex = cols.findIndex(col => col.id === collectionId);
    if (collectionIndex !== -1) {
      cols[collectionIndex].reviewMaterials = cols[collectionIndex].reviewMaterials.filter(
        material => !('questions' in material) || material.id !== testId
      );
    }
    return cols;
  });
}

export function updateTest(collectionId: string, updatedTest: Test) {
  collections.update(cols => {
    const collectionIndex = cols.findIndex(col => col.id === collectionId);
    if (collectionIndex !== -1) {
      const testIndex = cols[collectionIndex].reviewMaterials.findIndex(
        material => 'questions' in material && material.id === updatedTest.id
      );
      if (testIndex !== -1) {
        cols[collectionIndex].reviewMaterials[testIndex] = updatedTest;
      }
    }
    return cols;
  });
}
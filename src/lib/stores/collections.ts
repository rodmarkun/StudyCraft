import { writable } from 'svelte/store';
import type { Writable } from 'svelte/store';

export interface StudyMaterial {
  type: 'pdf' | 'markdown' | 'webpage';
  filePath?: string;
  fileName?: string;
  url?: string;
  name: string;
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

async function loadCollections(): Promise<Collection[]> {
  try {
    const content = await window.electronAPI.readFile('collections.json');
    return JSON.parse(content);
  } catch (error) {
    console.error('Error loading collections:', error);
    return [];
  }
}

async function saveCollections(collections: Collection[]): Promise<void> {
  try {
    await window.electronAPI.saveFile(JSON.stringify(collections), 'collections.json', '');
  } catch (error) {
    console.error('Error saving collections:', error);
  }
}

function createCollectionsStore() {
  const { subscribe, set, update } = writable<Collection[]>([]);

  return {
    subscribe,
    initialize: async () => {
      const loadedCollections = await loadCollections();
      set(loadedCollections);
    },
    addCollection: async (name: string) => {
      update(cols => {
        const newCols = [...cols, { id: Date.now().toString(), name, studyMaterials: [], reviewMaterials: [] }];
        saveCollections(newCols);
        return newCols;
      });
    },
    deleteCollection: async (id: string) => {
      update(cols => {
        const newCols = cols.filter(col => col.id !== id);
        saveCollections(newCols);
        return newCols;
      });
    },
    addStudyMaterials: async (collectionId: string, newMaterials: StudyMaterial[]) => {
      update(cols => {
        const newCols = cols.map(col => {
          if (col.id === collectionId) {
            return { ...col, studyMaterials: [...col.studyMaterials, ...newMaterials] };
          }
          return col;
        });
        saveCollections(newCols);
        return newCols;
      });
    },
    removeStudyMaterial: async (collectionId: string, materialName: string) => {
      update(cols => {
        const newCols = cols.map(col => {
          if (col.id === collectionId) {
            return {
              ...col,
              studyMaterials: col.studyMaterials.filter(material => material.name !== materialName)
            };
          }
          return col;
        });
        saveCollections(newCols);
        return newCols;
      });
    },
    addFlashcardDeck: async (collectionId: string, deck: FlashcardDeck) => {
      update(cols => {
        const newCols = cols.map(col => {
          if (col.id === collectionId) {
            return { ...col, reviewMaterials: [...col.reviewMaterials, deck] };
          }
          return col;
        });
        saveCollections(newCols);
        return newCols;
      });
    },
    removeFlashcardDeck: async (collectionId: string, deckId: string) => {
      update(cols => {
        const newCols = cols.map(col => {
          if (col.id === collectionId) {
            return {
              ...col,
              reviewMaterials: col.reviewMaterials.filter(
                material => !('flashcards' in material) || material.id !== deckId
              )
            };
          }
          return col;
        });
        saveCollections(newCols);
        return newCols;
      });
    },
    addFlashcardToDeck: async (collectionId: string, deckId: string, flashcard: Flashcard) => {
      update(cols => {
        const newCols = cols.map(col => {
          if (col.id === collectionId) {
            return {
              ...col,
              reviewMaterials: col.reviewMaterials.map(material => {
                if ('flashcards' in material && material.id === deckId) {
                  return {
                    ...material,
                    flashcards: [...material.flashcards, flashcard]
                  };
                }
                return material;
              })
            };
          }
          return col;
        });
        saveCollections(newCols);
        return newCols;
      });
    },
    removeFlashcardFromDeck: async (collectionId: string, deckId: string, flashcardId: string) => {
      update(cols => {
        const newCols = cols.map(col => {
          if (col.id === collectionId) {
            return {
              ...col,
              reviewMaterials: col.reviewMaterials.map(material => {
                if ('flashcards' in material && material.id === deckId) {
                  return {
                    ...material,
                    flashcards: material.flashcards.filter(card => card.id !== flashcardId)
                  };
                }
                return material;
              })
            };
          }
          return col;
        });
        saveCollections(newCols);
        return newCols;
      });
    },
    addTest: async (collectionId: string, test: Test) => {
      update(cols => {
        const newCols = cols.map(col => {
          if (col.id === collectionId) {
            return { ...col, reviewMaterials: [...col.reviewMaterials, test] };
          }
          return col;
        });
        saveCollections(newCols);
        return newCols;
      });
    },
    removeTest: async (collectionId: string, testId: string) => {
      update(cols => {
        const newCols = cols.map(col => {
          if (col.id === collectionId) {
            return {
              ...col,
              reviewMaterials: col.reviewMaterials.filter(
                material => !('questions' in material) || material.id !== testId
              )
            };
          }
          return col;
        });
        saveCollections(newCols);
        return newCols;
      });
    },
    updateTest: async (collectionId: string, updatedTest: Test) => {
      update(cols => {
        const newCols = cols.map(col => {
          if (col.id === collectionId) {
            return {
              ...col,
              reviewMaterials: col.reviewMaterials.map(material => 
                'questions' in material && material.id === updatedTest.id ? updatedTest : material
              )
            };
          }
          return col;
        });
        saveCollections(newCols);
        return newCols;
      });
    }
  };
}

export const collections = createCollectionsStore();

export async function loadStudyMaterialContent(filePath: string): Promise<string> {
  try {
    return await window.electronAPI.readFile(filePath);
  } catch (error) {
    console.error('Error loading study material content:', error);
    throw error;
  }
}

// Initialize the store when the app starts
collections.initialize();

// You can keep your existing function declarations, but they should now call the corresponding methods on the collections object
export const addCollection = collections.addCollection;
export const deleteCollection = collections.deleteCollection;
export const addStudyMaterials = collections.addStudyMaterials;
export const removeStudyMaterial = collections.removeStudyMaterial;
export const addFlashcardDeck = collections.addFlashcardDeck;
export const removeFlashcardDeck = collections.removeFlashcardDeck;
export const addFlashcardToDeck = collections.addFlashcardToDeck;
export const removeFlashcardFromDeck = collections.removeFlashcardFromDeck;
export const addTest = collections.addTest;
export const removeTest = collections.removeTest;
export const updateTest = collections.updateTest;
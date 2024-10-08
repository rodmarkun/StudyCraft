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
    console.log('No collections found, returning empty array');
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

  function getUniqueName(name: string, existingNames: Set<string>): string {
    let newName = name;
    let counter = 1;
    while (existingNames.has(newName)) {
      newName = `${name} copy${counter > 1 ? ' ' + counter : ''}`;
      counter++;
    }
    return newName;
  }

  return {
    subscribe,
    initialize: async () => {
      const loadedCollections = await loadCollections();
      set(loadedCollections);
    },
    addCollection: async (name: string) => {
      let uniqueName = '';
      await update(cols => {
        const existingNames = new Set(cols.map(c => c.name));
        uniqueName = getUniqueName(name, existingNames);
        const newCollection = { 
          id: Date.now().toString(), 
          name: uniqueName, 
          studyMaterials: [], 
          reviewMaterials: [] 
        };
        const newCols = [...cols, newCollection];
        saveCollections(newCols); // Save immediately after adding a new collection
        return newCols;
      });
      try {
        await window.electronAPI.createCollectionFolder(uniqueName);
      } catch (error) {
        console.error('Failed to create collection folder:', error);
        throw new Error('Failed to create collection');
      }
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
        const newCols = cols.map(col => 
          col.id === collectionId 
            ? { ...col, studyMaterials: [...col.studyMaterials, ...newMaterials.map(m => ({...m}))] }
            : col
        );
        saveCollections(newCols);
        return newCols;
      });
    },
    removeStudyMaterial: async (collectionId: string, materialName: string) => {
      update(cols => {
        const newCols = cols.map(col => 
          col.id === collectionId
            ? { 
                ...col, 
                studyMaterials: col.studyMaterials.filter(material => material.name !== materialName)
              }
            : col
        );
        saveCollections(newCols);
        return newCols;
      });
    },
    addFlashcardDeck: async (collectionId: string, deck: FlashcardDeck) => {
      update(cols => {
        const newCols = cols.map(col => {
          if (col.id === collectionId) {
            const existingDeckIndex = col.reviewMaterials.findIndex(
              (material): material is FlashcardDeck => 
                'flashcards' in material && material.id === deck.id
            );

            if (existingDeckIndex !== -1) {
              // Update existing deck
              return {
                ...col,
                reviewMaterials: col.reviewMaterials.map((material, index) =>
                  index === existingDeckIndex 
                    ? {...deck, flashcards: deck.flashcards.map(card => ({...card}))}
                    : material
                )
              };
            } else {
              // Add new deck
              return { 
                ...col, 
                reviewMaterials: [...col.reviewMaterials, {...deck, flashcards: deck.flashcards.map(card => ({...card}))}]
              };
            }
          }
          return col;
        });
        console.log('Saving collections:', JSON.stringify(newCols, null, 2));
        saveCollections(newCols);
        return newCols;
      });
    },
    updateFlashcardDeck: async (collectionId: string, updatedDeck: FlashcardDeck) => {
      update(cols => {
        try {
          const newCols = cols.map(col => {
            if (col.id === collectionId) {
              const newReviewMaterials = col.reviewMaterials.map(material => {
                if ('flashcards' in material && material.id === updatedDeck.id) {
                  return {
                    ...updatedDeck,
                    flashcards: updatedDeck.flashcards.map(card => ({...card}))
                  };
                }
                return material;
              });    
              return {
                ...col,
                reviewMaterials: newReviewMaterials
              };
            }
            return col;
          });
          saveCollections(newCols);
          return newCols;
        } catch (error) {
          return cols; // Return original state if there's an error
        }
      });
    },
    removeFlashcardDeck: async (collectionId: string, deckId: string) => {
      update(cols => {
        const newCols = cols.map(col => 
          col.id === collectionId
            ? {
                ...col,
                reviewMaterials: col.reviewMaterials.filter(material => 
                  !('flashcards' in material) || material.id !== deckId
                )
              }
            : col
        );
        saveCollections(newCols);
        return newCols;
      });
    },
    addTest: async (collectionId: string, test: Test) => {
      update(cols => {
        const newCols = cols.map(col => 
          col.id === collectionId
            ? { 
                ...col, 
                reviewMaterials: [
                  ...col.reviewMaterials, 
                  {...test, questions: test.questions.map(q => ({...q}))}
                ] 
              }
            : col
        );
        saveCollections(newCols);
        return newCols;
      });
    },
    removeTest: async (collectionId: string, testId: string) => {
      update(cols => {
        const newCols = cols.map(col => 
          col.id === collectionId
            ? {
                ...col,
                reviewMaterials: col.reviewMaterials.filter(material => 
                  !('questions' in material) || material.id !== testId
                )
              }
            : col
        );
        saveCollections(newCols);
        return newCols;
      });
    },
    updateTest: async (collectionId: string, updatedTest: Test) => {
      update(cols => {
        const newCols = cols.map(col => 
          col.id === collectionId
            ? {
                ...col,
                reviewMaterials: col.reviewMaterials.map(material => 
                  'questions' in material && material.id === updatedTest.id
                    ? {...updatedTest, questions: updatedTest.questions.map(q => ({...q}))}
                    : material
                )
              }
            : col
        );
        saveCollections(newCols);
        return newCols;
      });
    },
    renameCollection: async (id: string, newName: string) => {
      let oldName = '';
      let collection: Collection | undefined;
      
      update(collections => {
        collection = collections.find(c => c.id === id);
        oldName = collection ? collection.name : '';
        return collections;
      });
    
      if (oldName && collection) {
        try {
          await window.electronAPI.renameCollectionFolder(oldName, newName);
          
          // Update file paths for study materials
          const updatedStudyMaterials = collection.studyMaterials.map(material => {
            if (material.filePath) {
              const newFilePath = material.filePath.replace(oldName, newName);
              return { ...material, filePath: newFilePath };
            }
            return material;
          });
    
          update(collections => {
            const newCollections = collections.map(col => 
              col.id === id ? { ...col, name: newName, studyMaterials: updatedStudyMaterials } : col
            );
            saveCollections(newCollections); // Save after updating
            return newCollections;
          });
        } catch (error) {
          console.error('Failed to rename collection folder:', error);
          throw new Error('Failed to rename collection');
        }
      }
    },
    importCollection: async (importedCollection: Collection) => {
      let uniqueName = '';
      await update(cols => {
        const existingNames = new Set(cols.map(c => c.name));
        uniqueName = getUniqueName(importedCollection.name, existingNames);
        const newCollection = { ...importedCollection, name: uniqueName };
        return [...cols, newCollection];
      });
      try {
        await window.electronAPI.createCollectionFolder(uniqueName);
        // You might want to add logic here to copy imported study materials
      } catch (error) {
        console.error('Failed to create folder for imported collection:', error);
        throw new Error('Failed to import collection');
      }
    },
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
export const addTest = collections.addTest;
export const removeTest = collections.removeTest;
export const updateTest = collections.updateTest;
export const updateFlashcardDeck = collections.updateFlashcardDeck;
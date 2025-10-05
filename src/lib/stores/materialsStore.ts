import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { v4 as uuidv4 } from 'uuid';

export interface StudyMaterial {
  id: string;
  display_name?: string;
  name?: string;
  preview_available: boolean;
  is_processing: boolean;
  tags: string[];
}

export interface ReviewMaterial {
  id: string;
  name: string;
  type: 'flashcard_deck' | 'test';
  cards_count?: number;
  questions_count?: number;
  last_review?: string;
  tags: string[];
  review_material_id?: string | any;
  review_material_name?: string;
  review_material_type?: string | any;
}

export type MaterialType = 'study' | 'review';
export type SortOption = 'date_asc' | 'date_desc' | 'name_asc' | 'name_desc';

interface MaterialsState {
  materials: (StudyMaterial | ReviewMaterial)[];
  isLoading: boolean;
  materialType: MaterialType;
  sortOption: SortOption;
  searchTerm: string;
  selectedMaterials: (StudyMaterial | ReviewMaterial)[];
  selectionMode: boolean;
}

const initialState: MaterialsState = {
  materials: [],
  isLoading: true,
  materialType: 'study',
  sortOption: 'date_asc',
  searchTerm: '',
  selectedMaterials: [],
  selectionMode: false,
};

function createMaterialsStore() {
  const { subscribe, set, update } = writable<MaterialsState>(initialState);

  let unlisten: (() => void)[] = [];
  let pollingInterval: number | null = null;
  let isPolling = false;
  let savedScrollPosition = 0;

  const getMaterialIdentifier = (material: StudyMaterial | ReviewMaterial): string => {
    return material.id;
  };

  async function get(): Promise<MaterialsState> {
    let state: MaterialsState = initialState;
    subscribe(s => { state = s; })();
    return {...state};
  }

  function generateUniqueDisplayName(originalName: string, existingMaterials: (StudyMaterial | ReviewMaterial)[]): string {
    const existingDisplayNames = new Set(
      existingMaterials
        .filter(m => 'display_name' in m)
        .map(m => (m as StudyMaterial).display_name || (m as StudyMaterial).name || '')
    );

    if (!existingDisplayNames.has(originalName)) {
      return originalName;
    }

    const lastDotIndex = originalName.lastIndexOf('.');
    const nameWithoutExt = lastDotIndex > 0 ? originalName.slice(0, lastDotIndex) : originalName;
    const extension = lastDotIndex > 0 ? originalName.slice(lastDotIndex) : '';

    let counter = 1;
    let newName = `${nameWithoutExt} (${counter})${extension}`;

    while (existingDisplayNames.has(newName)) {
      counter++;
      newName = `${nameWithoutExt} (${counter})${extension}`;
    }

    return newName;
  }

async function updateSpecificMaterial(identifier: string) {
  try {
    const state = await get();
    if (state.materialType !== 'study') return;

    const allMaterials = await invoke<any[]>("get_study_materials", { sortBy: state.sortOption });
    const updatedMaterial = allMaterials.find((m: any) => m.id === identifier);
    
    if (!updatedMaterial) return;

    const formattedMaterial: StudyMaterial = {
      ...updatedMaterial,
      name: updatedMaterial.display_name
    };

    update(state => {
      const index = state.materials.findIndex(m => m.id === identifier);
      if (index === -1) return state;
      
      const newMaterials = [...state.materials];
      newMaterials[index] = formattedMaterial;
      
      return {
        ...state,
        materials: newMaterials
      };
    });

    if (!formattedMaterial.is_processing && isPolling) {
      stopPolling();
      setTimeout(() => loadMaterials(), 100);
    }
  } catch (error) {
    console.error('Failed to update specific material:', error);
  }
}

  function startPolling(identifier: string) {
    if (pollingInterval) {
      clearInterval(pollingInterval);
    }
    
    isPolling = true;
    let pollCount = 0;
    const maxPolls = 60; 
    
    pollingInterval = window.setInterval(async () => {
      pollCount++;
      
      if (pollCount >= maxPolls) {
        console.warn('Polling stopped after maximum attempts');
        stopPolling();
        return;
      }

      try {
        await updateSpecificMaterial(identifier);
      } catch (error) {
        console.error('Polling error:', error);
        stopPolling();
      }
    }, 2000); 
  }

  function stopPolling() {
    if (pollingInterval) {
      clearInterval(pollingInterval);
      pollingInterval = null;
    }
    isPolling = false;
  }

  async function initialize() {
    try {
      unlisten.push(await listen('cover-ready', (event) => {
        const payload = event.payload as string;
        const materialId = payload.split('/').pop() as string;
        updateSpecificMaterial(materialId);
      }));

      unlisten.push(await listen('material-updated', (event) => {
        if (!isPolling) {
          loadMaterials();
        }
      }));
      
      unlisten.push(await listen('flashcard-deck-created', async () => {
        const state = await get();
        if (state.materialType === 'review') {
          loadMaterials();
        }
      }));
      
      unlisten.push(await listen('flashcard-deck-updated', async () => {
        const state = await get();
        if (state.materialType === 'review') {
          loadMaterials();
        }
      }));

      loadMaterials();
    } catch (error) {
      console.error("Error initializing materials store:", error);
    }
  }

  function destroy() {
    stopPolling();
    unlisten.forEach(unl => unl());
    unlisten = [];
  }

  async function loadMaterials(identifier?: string) {
    if (identifier && isPolling) {
      await updateSpecificMaterial(identifier);
      return;
    }

    update(state => ({ ...state, isLoading: true }));

    try {
      const state = await get();
      let newMaterials: (StudyMaterial | ReviewMaterial)[] = [];
      
      if (state.materialType === 'study') {
        const studyMaterials = await invoke<any[]>("get_study_materials", { sortBy: state.sortOption });
        
        newMaterials = studyMaterials.map((m: any) => ({
          ...m,
          name: m.display_name
        })) as StudyMaterial[];
        
      } else {
        const reviewMaterialsRaw = await invoke<any[]>("get_review_materials", { sortBy: state.sortOption });

        newMaterials = await Promise.all(reviewMaterialsRaw.map(async (material: any) => {
          console.log("Processing review material from backend:", material);
          
          const materialId = String(material.id);
          
          const reviewMaterialType = String(material.rm_type || '').toLowerCase();
          const isFlashcardDeck = reviewMaterialType === 'flashcard_deck' || reviewMaterialType === 'flashcarddeck';
          
          const formattedMaterial: ReviewMaterial = {
            id: materialId,
            name: material.display_name,
            type: isFlashcardDeck ? "flashcard_deck" : "test",
            last_review: material.last_review,
            tags: [],
            review_material_id: materialId,
            review_material_name: material.display_name,
            review_material_type: material.rm_type
          };
          
          try {
            const tags = await invoke<string[]>("get_material_tags", { materialId: materialId });
            formattedMaterial.tags = tags;
            
            if (formattedMaterial.type === "flashcard_deck") {
              try {
                const cards = await invoke<any[]>("get_flashcards", { deckId: materialId });
                formattedMaterial.cards_count = cards?.length || 0;
                console.log(`Loaded ${formattedMaterial.cards_count} cards for deck ${materialId}`);
              } catch (cardError) {
                console.error(`Failed to get cards for deck ${materialId}:`, cardError);
                console.error("Full error details:", cardError);
                formattedMaterial.cards_count = 0;
              }
            } else if (formattedMaterial.type === "test") {
              try {
                const questions = await invoke<any[]>("get_test_questions", { testId: materialId });
                formattedMaterial.questions_count = questions?.length || 0;
                console.log(`Loaded ${formattedMaterial.questions_count} questions for test ${materialId}`);
              } catch (err) {
                console.error(`Failed to get questions for test ${materialId}:`, err);
                console.error("Full error details:", err);
                formattedMaterial.questions_count = 0;
              }
            }
          } catch (err) {
            console.error(`Failed to get additional data for material ${materialId}:`, err);
          }
          
          return formattedMaterial;
        }));
      }
      
      update(state => ({
        ...state,
        materials: newMaterials,
        isLoading: false
      }));
    } catch (error) {
      console.error(`Failed to load materials:`, error);
      update(state => ({ ...state, isLoading: false }));
    }
  }

  async function refreshReviewMaterials() {
    const state = await get();
    if (state.materialType === 'review') {
      console.log('Refreshing review materials...');
      await loadMaterials();
    }
  }

  function isFileSupported(fileName: string) {
    const fileNameLow = fileName.toLowerCase();
    const supportedExtensions = ['.pdf', '.md', '.txt'];
    return supportedExtensions.some(ext => fileNameLow.endsWith(ext));
  }

  async function addStudyMaterialFromPath(path: string) {
    const isSupported = isFileSupported(path);
    
    if (!path || !isSupported) {
      throw new Error('Please select a PDF, Markdown (.md), or Text (.txt) file');
    }
    
    try {
      const state = await get();
      const fileName = path.split(/[/\\]/).pop() || path;
      const uniqueDisplayName = generateUniqueDisplayName(fileName, state.materials);
      const materialId = uuidv4();

      if (uniqueDisplayName !== fileName) {
        console.log(`File renamed from "${fileName}" to "${uniqueDisplayName}" to avoid conflict`);
      }
      
      const tempMaterial: StudyMaterial = {
        id: materialId,
        name: uniqueDisplayName,
        display_name: uniqueDisplayName,
        preview_available: false,
        is_processing: true,
        tags: []
      };
      
      update(state => ({
        ...state,
        materials: state.materials.filter(m => m.id !== materialId)
      }));

      update(state => ({
        ...state,
        materials: [...state.materials, tempMaterial]
      }));
      
      await invoke('add_study_material_from_path', {
        id: materialId,
        filePath: path,
        fileName: uniqueDisplayName 
      });
      
      startPolling(materialId);
    } catch (error) {
      const state = await get();
      const fileName = path.split(/[/\\]/).pop() || path;
      const uniqueDisplayName = generateUniqueDisplayName(fileName, state.materials);
      const materialId = uuidv4();
      
      update(state => ({
        ...state,
        materials: state.materials.filter(m => m.id !== materialId)
      }));
      console.error('Failed to add study material from path:', error);
      throw error;
    }
  }

  async function addStudyMaterialFromLink(link: string) {
    if (!link || !link.trim()) {
      throw new Error('Please provide a valid URL');
    }

    try {
      const state = await get();
      const url = new URL(link.trim());
      const materialId = uuidv4();
      const materialName = url.pathname.split('/').filter(Boolean).pop() || url.hostname;
      const uniqueMaterialName = generateUniqueDisplayName(materialName + '.md', state.materials);
      const displayName = uniqueMaterialName.replace('.md', '');
      
      const tempMaterial: StudyMaterial = {
        id: materialId,
        name: displayName,
        display_name: displayName,
        preview_available: false,
        is_processing: true,
        tags: []
      };
      
      update(state => ({
        ...state,
        materials: state.materials.filter(m => m.id !== materialId)
      }));

      update(state => ({
        ...state,
        materials: [...state.materials, tempMaterial]
      }));
      
      await invoke('add_study_material_from_link', {
        id: materialId,
        link: link.trim()
      });
      
      startPolling(materialId);
    } catch (error) {
      const url = new URL(link.trim());
      const materialName = url.hostname + url.pathname.replace(/\/$/, '');
      const state = await get();
      const uniqueMaterialName = generateUniqueDisplayName(materialName + '.md', state.materials);
      const materialId = uuidv4();
      
      update(state => ({
        ...state,
        materials: state.materials.filter(m => m.id !== materialId)
      }));
      console.error('Failed to add study material from link:', error);
      throw error;
    }
  }

  async function addStudyMaterial(file: File) {
    const fileName = file.name;
    const isSupported = isFileSupported(fileName);
    
    if (!file || !isSupported) {
      throw new Error('Please select a PDF, Markdown (.md), or Text (.txt) file');
    }

    try {
      const state = await get();
      const materialId = uuidv4();
      const uniqueDisplayName = generateUniqueDisplayName(file.name, state.materials);
      
      if (uniqueDisplayName !== file.name) {
        console.log(`File renamed from "${file.name}" to "${uniqueDisplayName}" to avoid conflict`);
      }

      const tempMaterial: StudyMaterial = {
        id: materialId,
        name: uniqueDisplayName,
        display_name: uniqueDisplayName,
        preview_available: false,
        is_processing: true,
        tags: []
      };
      
      update(state => ({
        ...state,
        materials: [...state.materials.filter(m => m.id !== materialId), tempMaterial]
      }));

      const arrayBuffer = await file.arrayBuffer();
      const bytes = new Uint8Array(arrayBuffer);

      await invoke('add_study_material', {
        id: materialId,
        fileContent: Array.from(bytes),
        fileName: uniqueDisplayName 
      });

      startPolling(materialId);
    } catch (error) {
      const state = await get();
      const uniqueDisplayName = generateUniqueDisplayName(file.name, state.materials);
      const materialId = uuidv4();
      
      update(state => ({
        ...state,
        materials: state.materials.filter(m => m.id !== materialId)
      }));
      console.error('Failed to add study material:', error);
      throw error;
    }
  }

  async function addReviewMaterial(materialData: {
    name: string;
    type: 'flashcard_deck' | 'test';
    tags?: string[];
  }): Promise<string> {
    const state = await get();
    const materialId = uuidv4();
    const uniqueName = generateUniqueReviewMaterialName(materialData.name, state.materials);
    
    const tempMaterial: ReviewMaterial = {
      id: materialId,
      name: uniqueName,
      type: materialData.type,
      tags: materialData.tags || [],
      review_material_id: materialId,
      review_material_name: uniqueName,
      review_material_type: materialData.type === 'flashcard_deck' ? 'FlashcardDeck' : 'Test',
      cards_count: materialData.type === 'flashcard_deck' ? 0 : undefined,
      questions_count: materialData.type === 'test' ? 0 : undefined,
    };
    
    update(state => ({
      ...state,
      materials: [...state.materials, tempMaterial]
    }));
    
    return materialId;
  }

  function generateUniqueReviewMaterialName(originalName: string, existingMaterials: (StudyMaterial | ReviewMaterial)[]): string {
    const existingNames = new Set(
      existingMaterials.map(m => m.name || '')
    );

    if (!existingNames.has(originalName)) {
      return originalName;
    }

    let counter = 1;
    let newName = `${originalName} (${counter})`;

    while (existingNames.has(newName)) {
      counter++;
      newName = `${originalName} (${counter})`;
    }

    return newName;
  }

  async function deleteMaterial(material: StudyMaterial | ReviewMaterial) {
    try {
      const identifier = getMaterialIdentifier(material);
      
      if (isStudyMaterial(material)) {
        await invoke('delete_study_material', { id: identifier });
      } else {
        await invoke('delete_flashcard_deck', { deckId: identifier });
      }
      
      update(state => ({
        ...state,
        materials: state.materials.filter(m => 
          getMaterialIdentifier(m) !== identifier
        )
      }));
    } catch (error) {
      console.error(`Failed to delete material:`, error);
      throw error;
    }
  }

  function setSelectionMode(enabled: boolean, initialSelected: (StudyMaterial | ReviewMaterial)[] = []) {
    update(state => ({
      ...state,
      selectionMode: enabled,
      selectedMaterials: enabled ? [...initialSelected] : []
    }));
  }

  function toggleMaterialSelection(material: StudyMaterial | ReviewMaterial) {
    const identifier = getMaterialIdentifier(material);
    
    update(state => {
      if (state.selectedMaterials.some(m => getMaterialIdentifier(m) === identifier)) {
        return {
          ...state,
          selectedMaterials: state.selectedMaterials.filter(m => 
            getMaterialIdentifier(m) !== identifier
          )
        };
      } else {
        return {
          ...state,
          selectedMaterials: [...state.selectedMaterials, material]
        };
      }
    });
  }

  async function isMaterialSelected(material: StudyMaterial | ReviewMaterial): Promise<boolean> {
    const state = await get();
    if (!state.selectionMode) return false;
    
    const identifier = getMaterialIdentifier(material);
    return state.selectedMaterials.some(m => 
      getMaterialIdentifier(m) === identifier
    );
  }
  
  function setMaterialType(type: MaterialType) {
    update(state => {
      if (state.materialType === type) return state;
      
      if (isPolling) {
        stopPolling();
      }
      
      return {
        ...state,
        materialType: type,
        materials: [],
        isLoading: true,
        selectedMaterials: []
      };
    });
    
    setTimeout(() => {
      loadMaterials();
    }, 0);
  }
  
  function setSortOption(option: SortOption) {
    update(state => ({
      ...state,
      sortOption: option
    }));
    
    setTimeout(() => {
      loadMaterials();
    }, 0);
  }
  
  function setSearchTerm(term: string) {
    update(state => ({
      ...state,
      searchTerm: term
    }));
  }

  return {
    subscribe,
    initialize,
    destroy,
    loadMaterials,
    refreshReviewMaterials,
    addStudyMaterial,
    addStudyMaterialFromPath,
    addStudyMaterialFromLink,
    addReviewMaterial,
    deleteMaterial,
    setSelectionMode,
    toggleMaterialSelection,
    isMaterialSelected,
    setMaterialType,
    setSortOption,
    setSearchTerm,
    get,
  };
}

export const materialsStore = createMaterialsStore();

export const filteredMaterials = derived(
  materialsStore,
  ($materialsStore) => {
    return $materialsStore.materials.filter(material => {
      if (!$materialsStore.searchTerm) return true;
      
      const name = 'display_name' in material 
        ? (material.display_name || material.name || "")
        : (material.name || "");
        
      return name.toLowerCase().includes($materialsStore.searchTerm.toLowerCase());
    });
  }
);

export function isStudyMaterial(material: StudyMaterial | ReviewMaterial): material is StudyMaterial {
  return 'display_name' in material || ('name' in material && !('type' in material));
}

export function isReviewMaterial(material: StudyMaterial | ReviewMaterial): material is ReviewMaterial {
  return 'type' in material;
}
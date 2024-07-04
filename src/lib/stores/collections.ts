// src/lib/stores/collections.ts
import { writable } from 'svelte/store';
import type { Writable } from 'svelte/store';

export interface StudyMaterial {
  type: 'pdf' | 'markdown';
  filePath: string;
  fileName: string;
}

export interface ReviewMaterial {
  question: string;
  answer: string;
}

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

export function addMarkdownToCollection(collectionId: string, filePath: string, fileName: string) {
  collections.update(cols => {
    const index = cols.findIndex(col => col.id === collectionId);
    if (index !== -1) {
      cols[index].studyMaterials.push({ type: 'markdown', filePath, fileName });
    }
    return cols;
  });
}

export function removeStudyMaterial(collectionId: string, fileName: string) {
  collections.update(cols => {
    const index = cols.findIndex(col => col.id === collectionId);
    if (index !== -1) {
      cols[index].studyMaterials = cols[index].studyMaterials.filter(material => material.fileName !== fileName);
    }
    return cols;
  });
}

export function addStudyMaterials(collectionId: string, newMaterials: Array<{ type: 'pdf' | 'markdown' | 'webpage', filePath?: string, fileName?: string, url?: string }>) {
  collections.update(cols => {
    const collectionIndex = cols.findIndex(col => col.id === collectionId);
    if (collectionIndex !== -1) {
      cols[collectionIndex].studyMaterials = [...cols[collectionIndex].studyMaterials, ...newMaterials];
    }
    return cols;
  });
}
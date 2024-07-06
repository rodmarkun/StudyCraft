// src/lib/services/database.ts

import Database from 'better-sqlite3';
import { app } from 'electron';
import path from 'path';
import fs from 'fs';
import { Collection, StudyMaterial, FlashcardDeck, Test, Flashcard, TestData } from '../stores/collections';

let db: Database.Database;

export function initializeDatabase(): void {
  const dbPath = path.join(app.getPath('userData'), 'studycraft.db');
  const dbExists = fs.existsSync(dbPath);

  db = new Database(dbPath);

  if (!dbExists) {
    db.exec(`
      CREATE TABLE collections (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL
      );

      CREATE TABLE study_materials (
        id TEXT PRIMARY KEY,
        collection_id TEXT,
        type TEXT NOT NULL,
        name TEXT NOT NULL,
        file_path TEXT,
        url TEXT,
        FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
      );

      CREATE TABLE review_materials (
        id TEXT PRIMARY KEY,
        collection_id TEXT,
        type TEXT NOT NULL,
        name TEXT NOT NULL,
        FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
      );

      CREATE TABLE flashcards (
        id TEXT PRIMARY KEY,
        deck_id TEXT,
        question TEXT NOT NULL,
        answer TEXT NOT NULL,
        FOREIGN KEY (deck_id) REFERENCES review_materials(id) ON DELETE CASCADE
      );

      CREATE TABLE test_questions (
        id TEXT PRIMARY KEY,
        test_id TEXT,
        question TEXT NOT NULL,
        options TEXT NOT NULL,
        correct_option_index INTEGER NOT NULL,
        FOREIGN KEY (test_id) REFERENCES review_materials(id) ON DELETE CASCADE
      );
    `);
  }
}

export function getAllCollections(): Collection[] {
  try {
    const collections = db.prepare('SELECT * FROM collections').all();
    return collections.map(col => ({
      ...col,
      studyMaterials: getStudyMaterials(col.id),
      reviewMaterials: getReviewMaterials(col.id)
    }));
  } catch (error) {
    console.error('Error getting all collections:', error);
    return [];
  }
}

export function addCollection(name: string): string {
  try {
    const id = Date.now().toString();
    db.prepare('INSERT INTO collections (id, name) VALUES (?, ?)').run(id, name);
    return id;
  } catch (error) {
    console.error('Error adding collection:', error);
    throw error;
  }
}

export function updateCollection(id: string, name: string): void {
  try {
    db.prepare('UPDATE collections SET name = ? WHERE id = ?').run(name, id);
  } catch (error) {
    console.error('Error updating collection:', error);
    throw error;
  }
}

export function deleteCollection(id: string): void {
  try {
    db.prepare('DELETE FROM collections WHERE id = ?').run(id);
  } catch (error) {
    console.error('Error deleting collection:', error);
    throw error;
  }
}

export function getStudyMaterials(collectionId: string): StudyMaterial[] {
  try {
    return db.prepare('SELECT * FROM study_materials WHERE collection_id = ?').all(collectionId);
  } catch (error) {
    console.error('Error getting study materials:', error);
    return [];
  }
}

export function addStudyMaterial(collectionId: string, material: StudyMaterial): string {
  try {
    const id = Date.now().toString();
    db.prepare('INSERT INTO study_materials (id, collection_id, type, name, file_path, url) VALUES (?, ?, ?, ?, ?, ?)')
      .run(id, collectionId, material.type, material.name, material.filePath, material.url);
    return id;
  } catch (error) {
    console.error('Error adding study material:', error);
    throw error;
  }
}

export function updateStudyMaterial(material: StudyMaterial): void {
  try {
    db.prepare('UPDATE study_materials SET type = ?, name = ?, file_path = ?, url = ? WHERE id = ?')
      .run(material.type, material.name, material.filePath, material.url, material.id);
  } catch (error) {
    console.error('Error updating study material:', error);
    throw error;
  }
}

export function deleteStudyMaterial(id: string): void {
  try {
    db.prepare('DELETE FROM study_materials WHERE id = ?').run(id);
  } catch (error) {
    console.error('Error deleting study material:', error);
    throw error;
  }
}

export function getReviewMaterials(collectionId: string): (FlashcardDeck | Test)[] {
  try {
    const reviewMaterials = db.prepare('SELECT * FROM review_materials WHERE collection_id = ?').all(collectionId);
    return reviewMaterials.map(material => {
      if (material.type === 'flashcard_deck') {
        return {
          ...material,
          flashcards: getFlashcards(material.id)
        } as FlashcardDeck;
      } else if (material.type === 'test') {
        return {
          ...material,
          questions: getTestQuestions(material.id)
        } as Test;
      }
      return material;
    });
  } catch (error) {
    console.error('Error getting review materials:', error);
    return [];
  }
}

export function addFlashcardDeck(collectionId: string, deck: FlashcardDeck): string {
  try {
    const id = Date.now().toString();
    db.prepare('INSERT INTO review_materials (id, collection_id, type, name) VALUES (?, ?, ?, ?)')
      .run(id, collectionId, 'flashcard_deck', deck.name);
    deck.flashcards.forEach(card => addFlashcard(id, card));
    return id;
  } catch (error) {
    console.error('Error adding flashcard deck:', error);
    throw error;
  }
}

export function updateFlashcardDeck(deck: FlashcardDeck): void {
  try {
    db.prepare('UPDATE review_materials SET name = ? WHERE id = ?').run(deck.name, deck.id);
    db.prepare('DELETE FROM flashcards WHERE deck_id = ?').run(deck.id);
    deck.flashcards.forEach(card => addFlashcard(deck.id, card));
  } catch (error) {
    console.error('Error updating flashcard deck:', error);
    throw error;
  }
}

export function deleteFlashcardDeck(id: string): void {
  try {
    db.prepare('DELETE FROM review_materials WHERE id = ?').run(id);
  } catch (error) {
    console.error('Error deleting flashcard deck:', error);
    throw error;
  }
}

export function getFlashcards(deckId: string): Flashcard[] {
  try {
    return db.prepare('SELECT * FROM flashcards WHERE deck_id = ?').all(deckId);
  } catch (error) {
    console.error('Error getting flashcards:', error);
    return [];
  }
}

export function addFlashcard(deckId: string, card: Flashcard): string {
  try {
    const id = Date.now().toString();
    db.prepare('INSERT INTO flashcards (id, deck_id, question, answer) VALUES (?, ?, ?, ?)')
      .run(id, deckId, card.question, card.answer);
    return id;
  } catch (error) {
    console.error('Error adding flashcard:', error);
    throw error;
  }
}

export function updateFlashcard(card: Flashcard): void {
  try {
    db.prepare('UPDATE flashcards SET question = ?, answer = ? WHERE id = ?')
      .run(card.question, card.answer, card.id);
  } catch (error) {
    console.error('Error updating flashcard:', error);
    throw error;
  }
}

export function deleteFlashcard(id: string): void {
  try {
    db.prepare('DELETE FROM flashcards WHERE id = ?').run(id);
  } catch (error) {
    console.error('Error deleting flashcard:', error);
    throw error;
  }
}

export function addTest(collectionId: string, test: Test): string {
  try {
    const id = Date.now().toString();
    db.prepare('INSERT INTO review_materials (id, collection_id, type, name) VALUES (?, ?, ?, ?)')
      .run(id, collectionId, 'test', test.name);
    test.questions.forEach(question => addTestQuestion(id, question));
    return id;
  } catch (error) {
    console.error('Error adding test:', error);
    throw error;
  }
}

export function updateTest(test: Test): void {
  try {
    db.prepare('UPDATE review_materials SET name = ? WHERE id = ?').run(test.name, test.id);
    db.prepare('DELETE FROM test_questions WHERE test_id = ?').run(test.id);
    test.questions.forEach(question => addTestQuestion(test.id, question));
  } catch (error) {
    console.error('Error updating test:', error);
    throw error;
  }
}

export function deleteTest(id: string): void {
  try {
    db.prepare('DELETE FROM review_materials WHERE id = ?').run(id);
  } catch (error) {
    console.error('Error deleting test:', error);
    throw error;
  }
}

export function getTestQuestions(testId: string): TestData[] {
  try {
    return db.prepare('SELECT * FROM test_questions WHERE test_id = ?').all(testId);
  } catch (error) {
    console.error('Error getting test questions:', error);
    return [];
  }
}

export function addTestQuestion(testId: string, question: TestData): string {
  try {
    const id = Date.now().toString();
    db.prepare('INSERT INTO test_questions (id, test_id, question, options, correct_option_index) VALUES (?, ?, ?, ?, ?)')
      .run(id, testId, question.question, JSON.stringify(question.options), question.correctOptionIndex);
    return id;
  } catch (error) {
    console.error('Error adding test question:', error);
    throw error;
  }
}

export function updateTestQuestion(question: TestData): void {
  try {
    db.prepare('UPDATE test_questions SET question = ?, options = ?, correct_option_index = ? WHERE id = ?')
      .run(question.question, JSON.stringify(question.options), question.correctOptionIndex, question.id);
  } catch (error) {
    console.error('Error updating test question:', error);
    throw error;
  }
}

export function deleteTestQuestion(id: string): void {
  try {
    db.prepare('DELETE FROM test_questions WHERE id = ?').run(id);
  } catch (error) {
    console.error('Error deleting test question:', error);
    throw error;
  }
}
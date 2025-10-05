use super::connection::DbService;
use super::queries;
use super::review_materials::ReviewMaterialOperations;
use crate::commands::structure::requests;
use crate::errors::{AppError, AppResult};
use crate::materials::flashcard::{CardStatus, Flashcard, FlashcardCounts, NewFlashcard};
use crate::materials::review_material::{
    FlashcardDeckReviewSettings, ReviewMaterial, ReviewMaterialType,
};
use chrono::Utc;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rusqlite::{params, Row};

#[derive(Debug)]
struct FlashcardRow {
    id: i64,
    deck_id: String,
    front: String,
    back: String,
    position: i32,
    due_date: String,
    interval_days: i32,
    ease_factor: f64,
    repetitions: i32,
    status: String,
    last_reviewed: Option<String>,
    learning_step: i32,
    created_at: String,
    updated_at: String,
}

impl FlashcardRow {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(FlashcardRow {
            id: row.get("id")?,
            deck_id: row.get("deck_id")?,
            front: row.get("front")?,
            back: row.get("back")?,
            position: row.get("position")?,
            due_date: row.get("due_date")?,
            interval_days: row.get("interval_days")?,
            ease_factor: row.get("ease_factor")?,
            repetitions: row.get("repetitions")?,
            status: row.get("status")?,
            last_reviewed: row.get("last_reviewed")?,
            learning_step: row.get("learning_step")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }

    fn into_flashcard(self) -> AppResult<Flashcard> {
        let due_date = chrono::NaiveDateTime::parse_from_str(&self.due_date, "%Y-%m-%d %H:%M:%S")
            .map_err(|e| AppError::DbError(format!("Failed to parse due_date: {}", e)))?
            .and_utc();

        let last_reviewed = if let Some(lr_str) = self.last_reviewed {
            Some(
                chrono::NaiveDateTime::parse_from_str(&lr_str, "%Y-%m-%d %H:%M:%S")
                    .map_err(|e| {
                        AppError::DbError(format!("Failed to parse last_reviewed: {}", e))
                    })?
                    .and_utc(),
            )
        } else {
            None
        };

        let created_at =
            chrono::NaiveDateTime::parse_from_str(&self.created_at, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| AppError::DbError(format!("Failed to parse created_at: {}", e)))?
                .and_utc();

        let updated_at =
            chrono::NaiveDateTime::parse_from_str(&self.updated_at, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| AppError::DbError(format!("Failed to parse updated_at: {}", e)))?
                .and_utc();

        Ok(Flashcard {
            id: Some(self.id),
            deck_id: self.deck_id,
            front: self.front,
            back: self.back,
            position: self.position,
            due_date,
            interval_days: self.interval_days,
            ease_factor: self.ease_factor,
            repetitions: self.repetitions,
            status: CardStatus::from(self.status),
            last_reviewed,
            learning_step: self.learning_step,
            created_at,
            updated_at,
        })
    }
}

pub trait FlashcardDeckOperations {
    fn create_flashcard_deck(
        &self,
        deck_id_str: &str,
        name: &str,
        tags: &[String],
        cards: &[NewFlashcard],
    ) -> AppResult<ReviewMaterial>;
    fn get_flashcard_decks(&self) -> AppResult<Vec<ReviewMaterial>>;
    fn get_flashcard_deck(&self, id: &str) -> AppResult<Option<ReviewMaterial>>;
    fn get_flashcards(&self, deck_id: &str) -> AppResult<Vec<Flashcard>>;
    fn get_flashcards_for_review(
        &self,
        deck_id: &str,
        settings: &FlashcardDeckReviewSettings,
    ) -> AppResult<Vec<Flashcard>>;
    fn get_flashcard_counts(&self, deck_id: &str) -> AppResult<FlashcardCounts>;
    fn get_content_from_random_flashcard_deck(&self, nitems: u32) -> AppResult<Vec<Flashcard>>;
    fn update_flashcard_after_review(&self, card_id: i64, difficulty: &str)
        -> AppResult<Flashcard>;
    fn delete_flashcard_deck(&self, id: &str) -> AppResult<()>;
    fn update_flashcard_deck_name(&self, id: &str, name: &str) -> AppResult<()>;
    fn update_flashcard_deck(&self, request: requests::FlashcardDeckUpdateRequest)
        -> AppResult<()>;
}

impl FlashcardDeckOperations for DbService {
    fn create_flashcard_deck(
        &self,
        deck_id_str: &str,
        name: &str,
        tags: &[String],
        cards: &[NewFlashcard],
    ) -> AppResult<ReviewMaterial> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn
            .transaction()
            .map_err(|e| AppError::DbError(format!("Failed to start transaction: {}", e)))?;

        let now = Utc::now();

        let review_material = ReviewMaterial {
            id: String::from(deck_id_str),
            display_name: name.to_string(),
            rm_type: ReviewMaterialType::FlashcardDeck,
            last_review: None,
            created_at: now,
            updated_at: now,
        };

        self.add_review_material_tx(&tx, &review_material)?;

        for tag in tags {
            if tag.trim().is_empty() {
                continue;
            }

            tx.execute(queries::INSERT_TAG, params![tag.trim()])
                .map_err(|e| AppError::DbError(format!("Failed to insert tag: {}", e)))?;

            let tag_id: i64 = tx
                .query_row(queries::GET_TAG_ID, params![tag.trim()], |row| row.get(0))
                .map_err(|e| AppError::DbError(format!("Failed to get tag ID: {}", e)))?;

            tx.execute(queries::INSERT_MATERIAL_TAG, params![deck_id_str, tag_id])
                .map_err(|e| AppError::DbError(format!("Failed to link deck to tag: {}", e)))?;
        }

        for (i, card) in cards.iter().enumerate() {
            if card.front.trim().is_empty() || card.back.trim().is_empty() {
                continue; // skip empty cards
            }

            tx.execute(
                queries::INSERT_FLASHCARD,
                params![
                    deck_id_str,
                    card.front.trim(),
                    card.back.trim(),
                    card.position.max(i as i32)
                ],
            )
            .map_err(|e| AppError::DbError(format!("Failed to insert flashcard: {}", e)))?;
        }

        tx.commit()
            .map_err(|e| AppError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(review_material)
    }

    fn get_flashcard_decks(&self) -> AppResult<Vec<ReviewMaterial>> {
        Ok(self
            .get_review_materials()?
            .into_iter()
            .filter(|rm| matches!(rm.rm_type, ReviewMaterialType::FlashcardDeck))
            .collect())
    }

    fn get_flashcard_deck(&self, id: &str) -> AppResult<Option<ReviewMaterial>> {
        self.get_review_material(id)
    }

    fn get_flashcards(&self, deck_id: &str) -> AppResult<Vec<Flashcard>> {
        let conn = self.conn.borrow();

        let mut stmt = conn
            .prepare(queries::GET_FLASHCARDS_WITH_SCHEDULING)
            .map_err(|e| AppError::DbError(format!("Failed to prepare statement: {}", e)))?;

        let card_iter = stmt
            .query_map(params![deck_id], |row| FlashcardRow::from_row(row))
            .map_err(|e| AppError::DbError(format!("Failed to query flashcards: {}", e)))?;

        let mut cards = Vec::new();
        for card_result in card_iter {
            let row = card_result.map_err(|e| {
                AppError::DbError(format!("Failed to process flashcard row: {}", e))
            })?;
            cards.push(row.into_flashcard()?);
        }

        Ok(cards)
    }

    fn get_flashcards_for_review(
        &self,
        deck_id: &str,
        settings: &FlashcardDeckReviewSettings,
    ) -> AppResult<Vec<Flashcard>> {
        let conn = self.conn.borrow();

        let mut cards = if settings.use_only_pending {
            let mut stmt = conn
                .prepare(queries::GET_PENDING_FLASHCARDS)
                .map_err(|e| AppError::DbError(format!("Failed to prepare statement: {}", e)))?;

            let card_iter = stmt
                .query_map(params![deck_id], |row| FlashcardRow::from_row(row))
                .map_err(|e| {
                    AppError::DbError(format!("Failed to query pending flashcards: {}", e))
                })?;

            let mut cards = Vec::new();
            for card_result in card_iter {
                let row = card_result.map_err(|e| {
                    AppError::DbError(format!("Failed to process flashcard row: {}", e))
                })?;
                cards.push(row.into_flashcard()?);
            }
            cards
        } else {
            self.get_flashcards(deck_id)?
        };

        // Shuffle if requested
        if settings.shuffle_deck {
            let mut rng = thread_rng();
            cards.shuffle(&mut rng);
        }

        Ok(cards)
    }

    fn get_flashcard_counts(&self, deck_id: &str) -> AppResult<FlashcardCounts> {
        let conn = self.conn.borrow();

        let result = conn
            .query_row(queries::GET_FLASHCARD_COUNTS, params![deck_id], |row| {
                Ok(FlashcardCounts {
                    new_count: row.get(0)?,
                    learning_count: row.get(1)?,
                    review_count: row.get(2)?,
                    pending_count: row.get(3)?,
                    total_count: row.get(4)?,
                })
            })
            .map_err(|e| AppError::DbError(format!("Failed to get flashcard counts: {}", e)))?;

        Ok(result)
    }

    fn get_content_from_random_flashcard_deck(&self, nitems: u32) -> AppResult<Vec<Flashcard>> {
        let conn = self.conn.borrow();

        let random_deck_id: String =
            match conn.query_row(queries::GET_RANDOM_FLASHCARD_DECK_ID, [], |row| row.get(0)) {
                Ok(id) => id,
                Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(Vec::new()),
                Err(e) => {
                    return Err(AppError::DbError(format!(
                        "Failed to get random deck: {}",
                        e
                    )))
                }
            };

        let mut stmt = conn
            .prepare(queries::GET_RANDOM_FLASHCARDS_FROM_DECK)
            .map_err(|e| AppError::DbError(format!("Failed to prepare statement: {}", e)))?;

        let card_iter = stmt
            .query_map(params![random_deck_id, nitems], |row| {
                FlashcardRow::from_row(row)
            })
            .map_err(|e| AppError::DbError(format!("Failed to query random flashcards: {}", e)))?;

        let mut cards = Vec::new();
        for card_result in card_iter {
            let row = card_result.map_err(|e| {
                AppError::DbError(format!("Failed to process flashcard row: {}", e))
            })?;
            cards.push(row.into_flashcard()?);
        }

        Ok(cards)
    }

    fn update_flashcard_after_review(
        &self,
        card_id: i64,
        difficulty: &str,
    ) -> AppResult<Flashcard> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn
            .transaction()
            .map_err(|e| AppError::DbError(format!("Failed to start transaction: {}", e)))?;

        let card = tx
            .query_row(queries::GET_FLASHCARD_BY_ID, params![card_id], |row| {
                FlashcardRow::from_row(row)
            })
            .map_err(|e| AppError::DbError(format!("Failed to get flashcard: {}", e)))?
            .into_flashcard()?;

        let updated_card = card.calculate_next_review(difficulty);

        let due_date_str = updated_card
            .due_date
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let last_reviewed_str = updated_card
            .last_reviewed
            .as_ref()
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string());
        let status_str = String::from(updated_card.status.clone());
        let updated_at_str = updated_card
            .updated_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        tx.execute(
            queries::UPDATE_FLASHCARD_SCHEDULING,
            params![
                due_date_str,               // ?1 - due_date
                updated_card.interval_days, // ?2 - interval_days
                updated_card.ease_factor,   // ?3 - ease_factor
                updated_card.repetitions,   // ?4 - repetitions
                status_str,                 // ?5 - status
                last_reviewed_str,          // ?6 - last_reviewed
                updated_card.learning_step, // ?7 - learning_step
                updated_at_str,             // ?8 - updated_at
                card_id                     // ?9 - id
            ],
        )
        .map_err(|e| AppError::DbError(format!("Failed to update flashcard scheduling: {}", e)))?;

        tx.commit()
            .map_err(|e| AppError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(updated_card)
    }

    fn delete_flashcard_deck(&self, id: &str) -> AppResult<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn
            .transaction()
            .map_err(|e| AppError::DbError(format!("Failed to start transaction: {}", e)))?;

        self.delete_review_material_tx(&tx, id)?;

        tx.commit()
            .map_err(|e| AppError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    fn update_flashcard_deck_name(&self, id: &str, name: &str) -> AppResult<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn
            .transaction()
            .map_err(|e| AppError::DbError(format!("Failed to start transaction: {}", e)))?;

        self.update_review_material_name_tx(&tx, id, name)?;

        tx.commit()
            .map_err(|e| AppError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    fn update_flashcard_deck(
        &self,
        request: requests::FlashcardDeckUpdateRequest,
    ) -> AppResult<()> {
        let mut conn = self.conn.borrow_mut();
        let tx = conn
            .transaction()
            .map_err(|e| AppError::DbError(format!("Failed to start transaction: {}", e)))?;

        if let Some(new_name) = &request.name {
            self.update_review_material_name_tx(&tx, &request.deck_id, new_name)?;
        }

        if let Some(cards_to_delete) = &request.cards_to_delete {
            for card_id in cards_to_delete {
                tx.execute(queries::DELETE_FLASHCARD, params![card_id, request.deck_id])
                    .map_err(|e| {
                        AppError::DbError(format!("Failed to delete flashcard {}: {}", card_id, e))
                    })?;
            }
        }

        if let Some(cards_to_update) = &request.cards_to_update {
            for card in cards_to_update {
                tx.execute(
                    queries::UPDATE_EXISTING_FLASHCARD,
                    params![
                        card.front,
                        card.back,
                        card.position,
                        card.id,
                        request.deck_id
                    ],
                )
                .map_err(|e| {
                    AppError::DbError(format!("Failed to update flashcard {}: {}", card.id, e))
                })?;
            }
        }

        if let Some(cards_to_add) = &request.cards_to_add {
            for card in cards_to_add {
                tx.execute(
                    queries::INSERT_FLASHCARD,
                    params![request.deck_id, card.front, card.back, card.position],
                )
                .map_err(|e| AppError::DbError(format!("Failed to add new flashcard: {}", e)))?;
            }
        }

        // Update the review material's updated_at timestamp
        self.update_review_material_tx(&tx, &request.deck_id)?;

        tx.commit()
            .map_err(|e| AppError::DbError(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }
}

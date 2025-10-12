use crate::constants;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Main flashcard struct representing a stored flashcard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flashcard {
    pub id: Option<i64>, // Database ID. None for just new cards, Some(id) for existing.
    pub deck_id: String, // UUID as string
    pub front: String,   // Question
    pub back: String,    // Answer
    pub position: i32,   // Position in deck

    // Spaced repetition fields
    pub due_date: DateTime<Utc>, // Time in which switches to "Pending" again
    pub interval_days: i32,      // Waiting period before seeing cards in the "Review" state again
    pub ease_factor: f64,        // Multiplier that decides how fast interval_days grows
    pub repetitions: i32,        // How many times user has successfully reviewed a card in a row
    pub status: CardStatus,      // Review status of the card
    pub last_reviewed: Option<DateTime<Utc>>, // Last time reviewed
    pub learning_step: i32,      // Tracks which sort-term minute-based interval the card is on

    // Metadata
    pub created_at: DateTime<Utc>, // Date of creation
    pub updated_at: DateTime<Utc>, // Date of last update
}

/// Internal card creation struct for new flashcards (without database fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFlashcard {
    pub front: String,
    pub back: String,
    pub position: i32,
    pub deck_id: Option<String>,
}

/// Card update struct for modifying existing flashcards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFlashcard {
    pub id: i64,
    pub front: String,
    pub back: String,
    pub position: i32,
}

/// Card status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CardStatus {
    #[serde(rename = "new")]
    New,
    #[serde(rename = "learning")]
    Learning,
    #[serde(rename = "review")]
    Review,
    #[serde(rename = "suspended")]
    Suspended,
}

/// Flashcard counts for deck statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashcardCounts {
    pub new_count: i32,
    pub learning_count: i32,
    pub review_count: i32,
    pub pending_count: i32,
    pub total_count: i32,
}

impl From<String> for CardStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "new" => CardStatus::New,
            "learning" => CardStatus::Learning,
            "review" => CardStatus::Review,
            "suspended" => CardStatus::Suspended,
            _ => CardStatus::New,
        }
    }
}

impl From<CardStatus> for String {
    fn from(status: CardStatus) -> Self {
        match status {
            CardStatus::New => "new".to_string(),
            CardStatus::Learning => "learning".to_string(),
            CardStatus::Review => "review".to_string(),
            CardStatus::Suspended => "suspended".to_string(),
        }
    }
}

impl Default for FlashcardCounts {
    fn default() -> Self {
        Self {
            new_count: 0,
            learning_count: 0,
            review_count: 0,
            pending_count: 0,
            total_count: 0,
        }
    }
}

impl Flashcard {
    pub fn calculate_next_review(&self, difficulty: &str) -> Flashcard {
        let now = Utc::now();
        let mut updated_card = self.clone();
        updated_card.last_reviewed = Some(now);
        updated_card.updated_at = now;

        // Anki learning steps in minutes: [1, 10]
        // TODO - Let user modify all review calculation parameters
        let learning_steps = vec![1, 10];

        match difficulty {
            constants::AGAIN => {
                // ALWAYS goes back to first learning step
                updated_card.repetitions = 0;
                updated_card.learning_step = 0;
                updated_card.status = CardStatus::Learning;

                // Reduce ease factor (only for graduated cards)
                if self.status == CardStatus::Review {
                    updated_card.ease_factor = f64::max(1.3, updated_card.ease_factor - 0.2);
                }

                // Back to first learning step (1 minute)
                updated_card.due_date = now + Duration::minutes(learning_steps[0]);
                updated_card.interval_days = 0;
            }

            constants::HARD => {
                match updated_card.status {
                    CardStatus::New => {
                        // New card answered "hard": goes to learning with special timing
                        updated_card.status = CardStatus::Learning;
                        updated_card.learning_step = 0;

                        let again_minutes = learning_steps[0] as f64;
                        let good_minutes = learning_steps[0] as f64; // First step to next step
                        let hard_minutes = ((again_minutes + good_minutes) / 2.0).round() as i64;

                        updated_card.due_date = now + Duration::minutes(hard_minutes.max(1));
                        updated_card.interval_days = 0;
                        // Slight ease reduction for hard on new cards
                        updated_card.ease_factor = f64::max(1.3, updated_card.ease_factor - 0.15);
                    }

                    CardStatus::Learning => {
                        let current_step = updated_card.learning_step as usize;

                        if current_step == 0 {
                            // First learning step: Hard = average of Again and Good
                            let again_minutes = learning_steps[0] as f64;
                            let good_minutes = if learning_steps.len() > 1 {
                                learning_steps[1] as f64
                            } else {
                                // If only one step, Good would graduate, so use a reasonable middle ground
                                ((learning_steps[0] as f64) * 2.0).min(1440.0) // Max 1 day
                            };
                            let hard_minutes =
                                ((again_minutes + good_minutes) / 2.0).round() as i64;

                            updated_card.due_date = now + Duration::minutes(hard_minutes.max(1));
                        } else {
                            // On any step after the first: Hard REPEATS the current step
                            let current_step_minutes = learning_steps[current_step];
                            updated_card.due_date = now + Duration::minutes(current_step_minutes);
                            // learning_step stays the same (repeats current step)
                        }

                        updated_card.ease_factor = f64::max(1.3, updated_card.ease_factor - 0.15);
                    }

                    CardStatus::Review => {
                        // Review card answered "hard"
                        updated_card.repetitions += 1;

                        // For hard, uses interval * 1.2 (Anki standard)
                        let new_interval = f64::max(1.0, (updated_card.interval_days as f64) * 1.2);
                        updated_card.interval_days = new_interval as i32;
                        updated_card.due_date =
                            now + Duration::days(updated_card.interval_days as i64);

                        // Reduce ease factor
                        updated_card.ease_factor = f64::max(1.3, updated_card.ease_factor - 0.15);
                    }

                    _ => {}
                }
            }

            constants::GOOD => {
                match updated_card.status {
                    CardStatus::New => {
                        // New card graduates to learning, goes to first step
                        updated_card.status = CardStatus::Learning;
                        updated_card.learning_step = 0;
                        updated_card.due_date = now + Duration::minutes(learning_steps[0]);
                        updated_card.interval_days = 0;
                    }

                    CardStatus::Learning => {
                        let current_step = updated_card.learning_step as usize;
                        if current_step < learning_steps.len() - 1 {
                            // Move to next learning step
                            updated_card.learning_step += 1;
                            let next_step_minutes = learning_steps[current_step + 1];
                            updated_card.due_date = now + Duration::minutes(next_step_minutes);
                        } else {
                            // Graduate from learning to review with 1 day interval
                            updated_card.status = CardStatus::Review;
                            updated_card.interval_days = 1;
                            updated_card.due_date = now + Duration::days(1);
                            updated_card.repetitions = 1;
                            updated_card.learning_step = 0;
                        }
                    }

                    CardStatus::Review => {
                        // Standard Anki scheduling for review cards
                        updated_card.repetitions += 1;

                        if updated_card.repetitions == 1 {
                            updated_card.interval_days = 1;
                        } else if updated_card.repetitions == 2 {
                            updated_card.interval_days = 6;
                        } else {
                            // Subsequent reviews - use ease factor
                            let new_interval: f64 =
                                (updated_card.interval_days as f64) * updated_card.ease_factor;
                            updated_card.interval_days = new_interval.round() as i32;
                            // Cap at reasonable maximum, 1 year
                            updated_card.interval_days = i32::min(updated_card.interval_days, 365);
                        }

                        updated_card.due_date =
                            now + Duration::days(updated_card.interval_days as i64);
                    }

                    _ => {}
                }
            }

            constants::EASY => {
                match updated_card.status {
                    CardStatus::New => {
                        // skip learning entirely
                        updated_card.status = CardStatus::Review;
                        updated_card.interval_days = 4; // Anki default easy interval
                        updated_card.due_date = now + Duration::days(4);
                        updated_card.repetitions = 1;
                        updated_card.learning_step = 0;
                        // Bonus to ease factor for easy
                        updated_card.ease_factor = f64::min(2.5, updated_card.ease_factor + 0.15);
                    }

                    CardStatus::Learning => {
                        // graduate immediately with easy interval
                        updated_card.status = CardStatus::Review;
                        updated_card.interval_days = 4;
                        updated_card.due_date = now + Duration::days(4);
                        updated_card.repetitions = 1;
                        updated_card.learning_step = 0;
                        updated_card.ease_factor = f64::min(2.5, updated_card.ease_factor + 0.15);
                    }

                    CardStatus::Review => {
                        updated_card.repetitions += 1;

                        if updated_card.repetitions == 1 {
                            // First review as easy
                            updated_card.interval_days = 4;
                        } else if updated_card.repetitions == 2 {
                            // Second review as easy, uses good interval + easy bonus
                            let base_interval: f64 = 6.0;
                            updated_card.interval_days = (base_interval * 1.3).round() as i32;
                        // Easy bonus
                        } else {
                            // Use ease factor with easy bonus (1.3x multiplier)
                            let new_interval: f64 = (updated_card.interval_days as f64)
                                * updated_card.ease_factor
                                * 1.3;
                            updated_card.interval_days = new_interval.round() as i32;
                            updated_card.interval_days =
                                i32::min(updated_card.interval_days, 36500);
                        }

                        updated_card.due_date =
                            now + Duration::days(updated_card.interval_days as i64);

                        // Increase ease factor (Anki increases by 0.15, max 2.5)
                        updated_card.ease_factor = f64::min(2.5, updated_card.ease_factor + 0.15);
                    }

                    _ => {}
                }
            }

            _ => {
                // Invalid difficulties are treated as "good" (program should not get here)
                return self.calculate_next_review(constants::GOOD);
            }
        }

        updated_card
    }
}

pub fn export_to_anki(flashcards: Vec<Flashcard>) -> String {
    flashcards
        .iter()
        .map(|card| {
            let front = escape_anki_field(&card.front);
            let back = escape_anki_field(&card.back);

            format!("{}\t{}", front, back)
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn escape_anki_field(text: &str) -> String {
    text.replace('\t', "    ")  
        .replace('\n', "<br>")   
        .replace('\r', "")      
}